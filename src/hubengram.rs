//! hubengram CLI embedded in supervisor binary.
//! Cross-tenant engram access — CLI-only, NEVER registered as MCP tools.
//! Invocation: `hubengram ...` (symlink hubengram -> supervisor) or
//! `supervisor hubengram ...`. Shares the hub target state
//! (~/.hubportfolio.json) with hubportfolio: `hubportfolio use <id> [code]`
//! then `hubengram recall --query ...`. Talks to wrapper over TCP as
//! `hub.engram` / `hub.activate`; the wrapper forwards to the target's live
//! engram HTTP API (see engramapi.txt, 1:1 CLI mirror).

use clap::{Parser, Subcommand};
use serde_json::{json, Value};

use crate::hub::{hcall, load_state, own_id, resolve_target, save_state, HubState};

#[derive(Parser, Debug)]
#[command(name = "hubengram", about = "Cross-tenant engram memory — CLI only, talks to wrapper via supervisor proxy")]
pub struct HubEngramCli {
    #[arg(long, env = "WRAPPER_ADDRESS", default_value = "172.16.0.1")]
    pub wrapper_address: String,
    #[arg(long, env = "WRAPPER_PORT", default_value = "9090")]
    pub wrapper_port: u16,
    #[arg(long, env = "WRAPPER_TOKEN", default_value = "")]
    pub token: String,
    #[arg(long, env = "TENANT_ID")]
    pub tenant_id: Option<String>,
    /// Target tenant override (else the loaded `use` target, else self).
    #[arg(long)]
    pub to: Option<String>,
    /// Share-code override (else the loaded `use` code).
    #[arg(long)]
    pub code: Option<String>,
    #[command(subcommand)]
    pub command: HubEngramCommand,
}

#[derive(Subcommand, Debug)]
pub enum HubEngramCommand {
    /// Load someone's engram (needs their share code unless an admin or
    /// owner grant covers you). One at a time. Shared with hubportfolio.
    Use { target: String, code: Option<String> },
    /// Drop the loaded target.
    Leave,
    /// Show owner, loaded target and wrapper reachability.
    Info,
    /// Wake the target container (needed before engram access).
    Activate { target: Option<String> },
    /// recall memories: query text, or chain-walk with --follow-seq.
    Recall {
        query: Option<String>,
        #[arg(long)] limit: Option<u64>,
        #[arg(long)] follow_seq: Option<u64>,
        #[arg(long)] depth: Option<u64>,
        #[arg(long)] dir: Option<String>,
    },
    /// reward a recall so the memory learns (score -1..1).
    Reward {
        score: f64,
        #[arg(long)] recall_id: Option<u64>,
        #[arg(long)] attributions: Option<String>,
    },
    /// store one episodic memory, optionally chained after --seq.
    Ingest {
        text: String,
        #[arg(long)] seq: Option<u64>,
        #[arg(long)] seq_pos: Option<i64>,
    },
    Stats,
    /// engram daemon status (memory stats, learning state, reachability).
    Status,
    /// daemon liveness (DB-free wake check).
    Health,
    Setup,
    /// run one consolidation cycle.
    Consolidate {
        #[arg(long)] aggressive: bool,
    },
    TrainReranker,
    Help,
    /// escape hatch: forward any path with optional method + JSON body.
    Raw {
        path: String,
        #[arg(long)] method: Option<String>,
        #[arg(long)] data: Option<String>,
    },
}

fn as_hub(cli: &HubEngramCli) -> crate::hub::HubCli {
    crate::hub::HubCli {
        wrapper_address: cli.wrapper_address.clone(),
        wrapper_port: cli.wrapper_port,
        token: cli.token.clone(),
        tenant_id: cli.tenant_id.clone(),
        to: cli.to.clone(),
        code: cli.code.clone(),
        command: crate::hub::HubCommand::Status,
    }
}

pub async fn run_from_args(args: Vec<String>) -> anyhow::Result<()> {
    let cli = HubEngramCli::parse_from(args);
    let hub = as_hub(&cli);
    let me = own_id(&hub)?;
    match &cli.command {
        HubEngramCommand::Use { target, code } => {
            let data = json!({"target_tenant": target, "code": code});
            let resp = hcall(&hub, &me, "hub.show", "db_read", data).await.map_err(|e| {
                anyhow::anyhow!("cannot load tenant {target}: {e} (wrong/expired code, or no grant)")
            })?;
            if resp.get("error").is_some() {
                println!("{}", serde_json::to_string_pretty(&resp)?);
                anyhow::bail!("cannot load tenant {target}");
            }
            save_state(&HubState { target_tenant: target.clone(), code: code.clone() })?;
            println!("loaded {target}");
            Ok(())
        }
        HubEngramCommand::Leave => {
            save_state(&HubState::default())?;
            println!("left");
            Ok(())
        }
        HubEngramCommand::Info => {
            let st = load_state();
            let resp = hcall(&hub, &me, "hub.show", "db_read", json!({"target_tenant": me})).await;
            println!("{}", serde_json::to_string_pretty(&json!({
                "owner": me,
                "target": if st.target_tenant.is_empty() { Value::Null } else { Value::String(st.target_tenant) },
                "has_code": st.code.is_some(),
                "self_reachable": resp.is_ok(),
            }))?);
            Ok(())
        }
        HubEngramCommand::Activate { target } => {
            let (t, code) = resolve_target(&hub, &me, target.clone())?;
            eprintln!("hub -> {t}");
            let resp = hcall(&hub, &me, "hub.activate", "db_write",
                json!({"target_tenant": t, "code": code})).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        cmd => {
            let (t, code) = resolve_target(&hub, &me, None)?;
            eprintln!("hub -> {t}");
            let (path, method, body): (String, Option<String>, Option<Value>) = match cmd {
                HubEngramCommand::Recall { query, limit, follow_seq, depth, dir } => {
                    let mut m = serde_json::Map::new();
                    if let Some(q) = query { m.insert("query".to_string(), Value::String(q.clone())); }
                    if let Some(n) = limit { m.insert("limit".to_string(), json!(n)); }
                    if let Some(f) = follow_seq { m.insert("follow_seq".to_string(), json!(f)); }
                    if let Some(d) = depth { m.insert("depth".to_string(), json!(d)); }
                    if let Some(d) = dir { m.insert("dir".to_string(), Value::String(d.clone())); }
                    ("recall".to_string(), None, Some(Value::Object(m)))
                }
                HubEngramCommand::Reward { score, recall_id, attributions } => {
                    let mut m = serde_json::Map::new();
                    m.insert("score".to_string(), json!(score));
                    if let Some(r) = recall_id { m.insert("recall_id".to_string(), json!(r)); }
                    if let Some(a) = attributions { m.insert("attributions".to_string(), Value::String(a.clone())); }
                    ("reward".to_string(), None, Some(Value::Object(m)))
                }
                HubEngramCommand::Ingest { text, seq, seq_pos } => {
                    let mut m = serde_json::Map::new();
                    m.insert("text".to_string(), Value::String(text.clone()));
                    if let Some(p) = seq { m.insert("prev_id".to_string(), json!(p)); }
                    if let Some(q) = seq_pos { m.insert("seq_pos".to_string(), json!(q)); }
                    ("ingest".to_string(), None, Some(Value::Object(m)))
                }
                HubEngramCommand::Stats => ("stats".to_string(), None, None),
                HubEngramCommand::Status => ("status".to_string(), None, None),
                HubEngramCommand::Health => ("health".to_string(), None, None),
                HubEngramCommand::Setup => ("setup".to_string(), None, None),
                HubEngramCommand::Consolidate { aggressive } => (
                    "consolidate".to_string(), None,
                    if *aggressive { Some(json!({"aggressive": true})) } else { None },
                ),
                HubEngramCommand::TrainReranker => ("train-reranker".to_string(), None, None),
                HubEngramCommand::Help => ("help".to_string(), None, None),
                HubEngramCommand::Raw { path, method, data } => {
                    let body: Option<Value> = data.as_deref().map(|d| {
                        serde_json::from_str(d).map_err(|e| anyhow::anyhow!("bad --data JSON: {e}"))
                    }).transpose()?;
                    (path.clone(), method.clone(), body)
                }
                _ => anyhow::bail!("unreachable"),
            };
            let resp = hcall(&hub, &me, "hub.engram", "db_read",
                json!({"target_tenant": t, "code": code, "path": path, "method": method, "body": body})).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
    }
}

/// True for the hubengram binary, `supervisor hubengram ...`, or a bare
/// engram subcommand after `supervisor`.
pub fn is_hubengram_binary(args: &[String]) -> bool {
    let prog = std::path::Path::new(args.first().map(|s| s.as_str()).unwrap_or(""))
        .file_name().and_then(|s| s.to_str()).unwrap_or("");
    prog == "hubengram"
}

pub fn is_hubengram_invocation(args: &[String]) -> bool {
    if is_hubengram_binary(args) {
        return true;
    }
    if args.len() >= 2 && args[1] == "hubengram" {
        return true;
    }
    const SUBS: &[&str] = &[
        "recall", "reward", "ingest", "stats", "health", "setup", "consolidate",
        "train-reranker", "trainreranker", "raw", "info",
    ];
    if args.len() >= 2 && SUBS.contains(&args[1].as_str()) {
        return true;
    }
    false
}

pub fn normalize_hubengram_args(args: Vec<String>) -> Vec<String> {
    // `supervisor hubengram recall ...` -> `hubengram recall ...`
    if args.len() >= 2 && args[1] == "hubengram" {
        let mut v = vec![args[0].clone()];
        v.extend(args[2..].iter().cloned());
        if v[0].contains("supervisor") {
            v[0] = "hubengram".to_string();
        }
        return v;
    }
    const SUBS: &[&str] = &[
        "recall", "reward", "ingest", "stats", "health", "setup", "consolidate",
        "train-reranker", "trainreranker", "raw", "info",
    ];
    let prog = std::path::Path::new(&args[0]).file_name().and_then(|s| s.to_str()).unwrap_or("");
    if prog != "hubengram" && args.len() >= 2 && SUBS.contains(&args[1].as_str()) {
        let mut v = vec!["hubengram".to_string()];
        v.extend(args[1..].iter().cloned());
        return v;
    }
    args
}
