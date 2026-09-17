//! hubportfolio CLI embedded in supervisor binary.
//! Cross-tenant portfolio access — CLI-only, NEVER registered as MCP tools.
//! Invocation: `hubportfolio ...` (symlink hubportfolio -> supervisor) or
//! `supervisor hubportfolio ...`. Talks to wrapper over the same TCP channel
//! as `portfolio`, with `hub.*` ops. The wrapper identifies the caller by
//! container source IP; the owner needs no token for their own data.
//!
//! `hubportfolio use <tenant-id> [code]` loads one foreign portfolio at a
//! time (stored in ~/.hubportfolio.json); every portfolio command then acts
//! on that target until `hubportfolio leave`.

use clap::{Parser, Subcommand};
use serde_json::{json, Value};

use crate::cli::{
    self, AchievementCmd, AttemptCmd, GoalCmd, MilestoneCmd, ReportCmd, SkillCmd, TaskCmd,
    TimelineCmd,
};

#[derive(Parser, Debug)]
#[command(name = "hubportfolio", about = "Cross-tenant portfolio hub — CLI only, talks to wrapper via supervisor proxy")]
pub struct HubCli {
    #[arg(long, env = "WRAPPER_ADDRESS", default_value = "172.16.0.1")]
    pub wrapper_address: String,
    #[arg(long, env = "WRAPPER_PORT", default_value = "9090")]
    pub wrapper_port: u16,
    #[arg(long, env = "WRAPPER_TOKEN", default_value = "")]
    pub token: String,
    #[arg(long, env = "TENANT_ID")]
    pub tenant_id: Option<String>,
    /// Target tenant override (else the loaded `use` target).
    #[arg(long)]
    pub to: Option<String>,
    /// Share-code override (else the loaded `use` code).
    #[arg(long)]
    pub code: Option<String>,
    #[command(subcommand)]
    pub command: HubCommand,
}

#[derive(Subcommand, Debug)]
pub enum HubCommand {
    /// Load someone's portfolio (needs their share code unless an admin or
    /// owner grant covers you). One at a time.
    Use { target: String, code: Option<String> },
    /// Drop the loaded target.
    Leave,
    /// Show owner, loaded target and wrapper reachability.
    Status,
    /// Wake the target container (needed before engram access).
    Activate { target: Option<String> },
    /// Create a short share code for YOUR portfolio (default: never expires).
    Share {
        #[command(subcommand)] cmd: ShareCmd,
    },
    /// Who can reach YOUR portfolio (grants with names, active codes, blocks).
    Access,
    /// Block a tenant from your portfolio (overrides grants AND tokens).
    Block { tenant: String },
    /// Unblock a tenant.
    Unblock { tenant: String },
    /// Revoke a grantee's access to your portfolio.
    GrantRevoke { tenant: String },
    // ---- portfolio mirror on the loaded target (full edit) ----
    Show { #[arg(long)] json: bool },
    Update {
        #[arg(long)] name: Option<String>,
        #[arg(long)] nickname: Option<String>,
        #[arg(long)] track: Option<String>,
        #[arg(long)] language: Option<String>,
        #[arg(long)] timezone: Option<String>,
    },
    Goal { #[command(subcommand)] cmd: GoalCmd },
    Milestone { #[command(subcommand)] cmd: MilestoneCmd },
    Task { #[command(subcommand)] cmd: TaskCmd },
    Attempt { #[command(subcommand)] cmd: AttemptCmd },
    Skill { #[command(subcommand)] cmd: SkillCmd },
    Achievement { #[command(subcommand)] cmd: AchievementCmd },
    Report { #[command(subcommand)] cmd: ReportCmd },
    Timeline { #[command(subcommand)] cmd: TimelineCmd },
}

#[derive(Subcommand, Debug)]
pub enum ShareCmd {
    Create { #[arg(long)] expire: Option<String> },
    List,
    Cancel { code: String },
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub(crate) struct HubState {
    pub(crate) target_tenant: String,
    pub(crate) code: Option<String>,
}

pub(crate) fn state_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::Path::new(&h).join(".hubportfolio.json")
    })
}

pub(crate) fn load_state() -> HubState {
    state_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub(crate) fn save_state(st: &HubState) -> anyhow::Result<()> {
    let p = state_path().ok_or_else(|| anyhow::anyhow!("HOME not set"))?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(st)?)?;
    Ok(())
}

pub(crate) fn own_id(hub: &HubCli) -> anyhow::Result<String> {
    let cli = crate::cli::Cli {
        wrapper_address: hub.wrapper_address.clone(),
        wrapper_port: hub.wrapper_port,
        token: hub.token.clone(),
        tenant_id: hub.tenant_id.clone(),
        command: crate::cli::Command::Show { json: false },
    };
    crate::cli::tenant_id_or_fail(&cli)
}

pub(crate) async fn hcall(
    hub: &HubCli,
    owner: &str,
    op: &str,
    rtype: &str,
    data: Value,
) -> anyhow::Result<Value> {
    let cli = crate::cli::Cli {
        wrapper_address: hub.wrapper_address.clone(),
        wrapper_port: hub.wrapper_port,
        token: hub.token.clone(),
        tenant_id: hub.tenant_id.clone(),
        command: crate::cli::Command::Show { json: false },
    };
    crate::cli::call(owner, op, rtype, data, &cli).await
}

/// Resolve (target, code): explicit flags win, else loaded state, else self
/// (the owner always works on their own portfolio without `use`).
pub(crate) fn resolve_target(hub: &HubCli, me: &str, arg_target: Option<String>) -> anyhow::Result<(String, Option<String>)> {
    let st = load_state();
    let target = hub
        .to
        .clone()
        .or(arg_target)
        .or_else(|| if st.target_tenant.is_empty() { None } else { Some(st.target_tenant.clone()) })
        .unwrap_or_else(|| me.to_string());
    let code = hub.code.clone().or(st.code.clone()).filter(|c| !c.is_empty());
    Ok((target, code))
}

pub async fn run_from_args(args: Vec<String>) -> anyhow::Result<()> {
    let hub = HubCli::parse_from(args);
    let me = own_id(&hub)?;
    match &hub.command {
        HubCommand::Use { target, code } => {
            // Verify before saving: owner always passes; foreign needs grant/code.
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
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        HubCommand::Leave => {
            save_state(&HubState::default())?;
            println!("left");
            Ok(())
        }
        HubCommand::Status => {
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
        HubCommand::Activate { target } => {
            let (t, code) = resolve_target(&hub, &me, target.clone())?;
            eprintln!("hub -> {t}");
            let resp = hcall(&hub, &me, "hub.activate", "db_write",
                json!({"target_tenant": t, "code": code})).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        HubCommand::Share { cmd } => {
            let resp = match cmd {
                ShareCmd::Create { expire } => {
                    hcall(&hub, &me, "hub.share_create", "db_write", json!({"expires_at": expire})).await?
                }
                ShareCmd::List => hcall(&hub, &me, "hub.share_list", "db_read", json!({})).await?,
                ShareCmd::Cancel { code } => {
                    hcall(&hub, &me, "hub.share_cancel", "db_write", json!({"code": code})).await?
                }
            };
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        HubCommand::Access => {
            let resp = hcall(&hub, &me, "hub.access", "db_read", json!({})).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        HubCommand::Block { tenant } => {
            let resp = hcall(&hub, &me, "hub.block", "db_write", json!({"tenant": tenant})).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        HubCommand::Unblock { tenant } => {
            let resp = hcall(&hub, &me, "hub.unblock", "db_write", json!({"tenant": tenant})).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        HubCommand::GrantRevoke { tenant } => {
            let resp = hcall(&hub, &me, "hub.grant_revoke", "db_write", json!({"tenant": tenant})).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        // ---- portfolio mirror ----
        HubCommand::Show { json: _ } => hub_portfolio(&hub, &me, cli::Command::Show { json: false }).await,
        HubCommand::Update { name, nickname, track, language, timezone } => {
            hub_portfolio(&hub, &me, cli::Command::Update {
                name: name.clone(), nickname: nickname.clone(), track: track.clone(),
                language: language.clone(), timezone: timezone.clone(),
            }).await
        }
        HubCommand::Goal { cmd } => hub_portfolio(&hub, &me, cli::Command::Goal { cmd: cmd.clone() }).await,
        HubCommand::Milestone { cmd } => hub_portfolio(&hub, &me, cli::Command::Milestone { cmd: cmd.clone() }).await,
        HubCommand::Task { cmd } => hub_portfolio(&hub, &me, cli::Command::Task { cmd: cmd.clone() }).await,
        HubCommand::Attempt { cmd } => hub_portfolio(&hub, &me, cli::Command::Attempt { cmd: cmd.clone() }).await,
        HubCommand::Skill { cmd } => hub_portfolio(&hub, &me, cli::Command::Skill { cmd: cmd.clone() }).await,
        HubCommand::Achievement { cmd } => hub_portfolio(&hub, &me, cli::Command::Achievement { cmd: cmd.clone() }).await,
        HubCommand::Report { cmd } => hub_portfolio(&hub, &me, cli::Command::Report { cmd: cmd.clone() }).await,
        HubCommand::Timeline { cmd } => hub_portfolio(&hub, &me, cli::Command::Timeline { cmd: cmd.clone() }).await,
    }
}

async fn hub_portfolio(hub: &HubCli, me: &str, cmd: cli::Command) -> anyhow::Result<()> {
    let (op, rtype, mut data) = cli::portfolio_op(&cmd);
    let hub_op = op.replacen("portfolio.", "hub.", 1);
    let (target, code) = resolve_target(hub, me, None)?;
    eprintln!("hub -> {target}");
    if let Some(obj) = data.as_object_mut() {
        obj.insert("target_tenant".to_string(), Value::String(target));
        obj.insert("code".to_string(), code.map(Value::String).unwrap_or(Value::Null));
    }
    let resp = hcall(hub, me, &hub_op, rtype, data).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

/// True when argv[0] itself is the hubportfolio binary (symlink). Checked
/// before portfolio detection: `hubportfolio --help` must show hub help.
pub fn is_hub_binary(args: &[String]) -> bool {
    let prog = std::path::Path::new(args.first().map(|s| s.as_str()).unwrap_or(""))
        .file_name().and_then(|s| s.to_str()).unwrap_or("");
    prog == "hubportfolio"
}

/// Mirror of portfolio detection: hubportfolio binary, `supervisor
/// hubportfolio ...`, or a bare hub subcommand after `supervisor`.
pub fn is_hubportfolio_invocation(args: &[String]) -> bool {
    let prog = std::path::Path::new(args.first().map(|s| s.as_str()).unwrap_or(""))
        .file_name().and_then(|s| s.to_str()).unwrap_or("");
    if prog == "hubportfolio" {
        return true;
    }
    if args.len() >= 2 && args[1] == "hubportfolio" {
        return true;
    }
    const SUBS: &[&str] = &[
        "use", "leave", "status", "share", "access", "block",
        "unblock", "grantrevoke", "grant-revoke",
    ];
    if args.len() >= 2 && SUBS.contains(&args[1].as_str()) {
        return true;
    }
    false
}

pub fn normalize_hubportfolio_args(args: Vec<String>) -> Vec<String> {
    // `supervisor hubportfolio use ...` -> `hubportfolio use ...`
    if args.len() >= 2 && args[1] == "hubportfolio" {
        let mut v = vec![args[0].clone()];
        v.extend(args[2..].iter().cloned());
        if v[0].contains("supervisor") {
            v[0] = "hubportfolio".to_string();
        }
        return v;
    }
    // invoked as `supervisor use ...` (hub-only word) -> inject prog name.
    // Portfolio-mirror words stay with the portfolio path (checked first).
    const SUBS: &[&str] = &[
        "use", "leave", "status", "share", "access", "block",
        "unblock", "grantrevoke", "grant-revoke",
    ];
    let prog = std::path::Path::new(&args[0]).file_name().and_then(|s| s.to_str()).unwrap_or("");
    if prog != "hubportfolio" && args.len() >= 2 && SUBS.contains(&args[1].as_str()) {
        let mut v = vec!["hubportfolio".to_string()];
        v.extend(args[1..].iter().cloned());
        return v;
    }
    args
}
