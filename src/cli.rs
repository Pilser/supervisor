//! Portfolio CLI embedded in supervisor binary.
//! Invocation: `portfolio show ...` (symlink portfolio -> supervisor) or
//! `supervisor portfolio show ...`. Shares wrapper proxy with daemon.

use clap::{Parser, Subcommand};
use serde_json::{json, Value};

#[derive(Parser, Debug)]
#[command(name = "portfolio", about = "Portfolio ledger CLI — talks to wrapper via supervisor proxy")]
pub struct Cli {
    #[arg(long, env = "WRAPPER_ADDRESS", default_value = "172.16.0.1")]
    pub wrapper_address: String,
    #[arg(long, env = "WRAPPER_PORT", default_value = "9090")]
    pub wrapper_port: u16,
    #[arg(long, env = "WRAPPER_TOKEN", default_value = "")]
    pub token: String,
    #[arg(long, env = "TENANT_ID")]
    pub tenant_id: Option<String>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
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

#[derive(Subcommand, Debug, Clone)]
pub enum GoalCmd {
    Create { #[arg(long)] title: String, #[arg(long)] track: Option<String>, #[arg(long)] target_date: Option<String> },
    Update { #[arg(long)] id: i64, #[arg(long)] title: Option<String>, #[arg(long)] status: Option<String>, #[arg(long)] target_date: Option<String> },
    List { #[arg(long)] status: Option<String>, #[arg(long)] limit: Option<i64> },
    Get { #[arg(long)] id: i64 },
}
#[derive(Subcommand, Debug, Clone)]
pub enum MilestoneCmd {
    Add { #[arg(long)] goal: i64, #[arg(long)] title: String, #[arg(long)] order: Option<i64> },
    List { #[arg(long)] goal: i64 },
    Update { #[arg(long)] id: i64, #[arg(long)] title: Option<String>, #[arg(long)] status: Option<String> },
}
#[derive(Subcommand, Debug, Clone)]
pub enum TaskCmd {
    Create { #[arg(long)] milestone: i64, #[arg(long)] title: String, #[arg(long)] kind: String, #[arg(long)] description: Option<String>, #[arg(long)] difficulty: Option<String> },
    Update { #[arg(long)] id: i64, #[arg(long)] title: Option<String>, #[arg(long)] status: Option<String>, #[arg(long)] description: Option<String> },
    List { #[arg(long)] milestone: Option<i64>, #[arg(long)] status: Option<String>, #[arg(long)] kind: Option<String> },
    Get { #[arg(long)] id: i64 },
}
#[derive(Subcommand, Debug, Clone)]
pub enum AttemptCmd {
    Submit { #[arg(long)] task: i64, #[arg(long)] result: String, #[arg(long)] output: String, #[arg(long)] feedback: Option<String> },
    Resolve { #[arg(long)] id: i64, #[arg(long)] example: String, #[arg(long)] note: Option<String> },
    List { #[arg(long)] task: i64, #[arg(long)] limit: Option<i64> },
    Get { #[arg(long)] id: i64 },
}
#[derive(Subcommand, Debug, Clone)]
pub enum SkillCmd {
    Record { #[arg(long)] name: String, #[arg(long)] domain: Option<String>, #[arg(long)] level: Option<String>, #[arg(long)] evidence: Option<i64>, #[arg(long)] confidence: Option<f64> },
    Update { #[arg(long)] id: i64, #[arg(long)] level: Option<String>, #[arg(long)] evidence: Option<i64> },
    List { #[arg(long)] domain: Option<String> },
    Get { #[arg(long)] id: i64 },
}
#[derive(Subcommand, Debug, Clone)]
pub enum AchievementCmd {
    Award { #[arg(long)] title: String, #[arg(long)] skill: Option<i64>, #[arg(long)] task: Option<i64> },
    List { #[arg(long)] limit: Option<i64> },
    Get { #[arg(long)] id: i64 },
}
#[derive(Subcommand, Debug, Clone)]
pub enum ReportCmd {
    Generate { #[arg(long)] period: String, #[arg(long)] content: Option<String> },
    List { #[arg(long)] limit: Option<i64> },
    Get { #[arg(long)] id: i64 },
}
#[derive(Subcommand, Debug, Clone)]
pub enum TimelineCmd {
    List { #[arg(long)] kind: Option<String>, #[arg(long)] since: Option<String>, #[arg(long)] limit: Option<i64> },
    Get { #[arg(long)] id: i64 },
    Add { #[arg(long)] summary: String, #[arg(long)] kind: Option<String>, #[arg(long, alias="ref-id")] ref_id: Option<i64>, #[arg(long, alias="ref-type")] ref_type: Option<String> },
}

pub(crate) fn tenant_id_or_fail(cli: &Cli) -> anyhow::Result<String> {
    if let Some(t) = cli.tenant_id.clone() { if !t.trim().is_empty() { return Ok(t); } }
    if let Ok(t) = std::env::var("TENANT_ID") { if !t.trim().is_empty() { return Ok(t); } }
    if let Ok(s) = std::fs::read_to_string("config.toml") { for line in s.lines() { if line.trim().starts_with("tenant_id") { if let Some(v) = line.split('=').nth(1) { let v = v.trim().trim_matches('"').trim_matches('\''); if !v.is_empty() { return Ok(v.to_string()); } } } } }
    anyhow::bail!("TENANT_ID not set (env or config.toml)")
}

pub(crate) async fn call(tenant_id: &str, op: &str, rtype: &str, data: Value, cli: &Cli) -> anyhow::Result<Value> {
    let tenant = supervisor_core::TenantContext::new(tenant_id, "", "", "");
    let rtype = proxy::RequestType::from_str(rtype).ok_or_else(|| anyhow::anyhow!("unknown request type {rtype}"))?;
    let mut envelope = proxy::RequestBuilder::new(rtype).operation(op).tenant(tenant).data(data).build()?;
    if let Some(obj) = envelope.as_object_mut() { if let Some(p) = obj.get_mut("payload").and_then(|p| p.as_object_mut()) { p.entry("token".to_string()).or_insert(Value::String(cli.token.clone())); } }
    let client = proxy::ProxyClient::new(&cli.wrapper_address, cli.wrapper_port, &cli.token)?;
    let resp = client.send_json(envelope).await?;
    Ok(resp.get("payload").cloned().unwrap_or(resp))
}

/// Map a portfolio subcommand to (dotted op, request type, data).
/// Hub reuses this with the `portfolio.` prefix swapped to `hub.`.
pub fn portfolio_op(cmd: &Command) -> (&'static str, &'static str, Value) {
    match cmd {
        Command::Show { .. } => ("portfolio.show", "db_read", json!({})),
        Command::Update { name, nickname, track, language, timezone } => ("portfolio.update", "db_write", json!({ "agent_name": name, "nickname": nickname, "track": track, "language": language, "timezone": timezone })),
        Command::Goal { cmd } => match cmd {
            GoalCmd::Create { title, track, target_date } => ("portfolio.create_goal", "db_write", json!({"title": title, "track": track, "target_date": target_date})),
            GoalCmd::Update { id, title, status, target_date } => ("portfolio.update_goal", "db_write", json!({"id": id, "title": title, "status": status, "target_date": target_date})),
            GoalCmd::List { status, limit } => ("portfolio.list_goals", "db_read", json!({"status": status, "limit": limit})),
            GoalCmd::Get { id } => ("portfolio.get_goal", "db_read", json!({"id": id})),
        },
        Command::Milestone { cmd } => match cmd {
            MilestoneCmd::Add { goal, title, order } => ("portfolio.add_milestone", "db_write", json!({"goal_id": goal, "title": title, "order_index": order})),
            MilestoneCmd::List { goal } => ("portfolio.list_milestones", "db_read", json!({"goal_id": goal})),
            MilestoneCmd::Update { id, title, status } => ("portfolio.update_milestone", "db_write", json!({"id": id, "title": title, "status": status})),
        },
        Command::Task { cmd } => match cmd {
            TaskCmd::Create { milestone, title, kind, description, difficulty } => ("portfolio.create_task", "db_write", json!({"milestone_id": milestone, "title": title, "kind": kind, "description": description, "difficulty": difficulty})),
            TaskCmd::Update { id, title, status, description } => ("portfolio.update_task", "db_write", json!({"id": id, "title": title, "status": status, "description": description})),
            TaskCmd::List { milestone, status, kind } => ("portfolio.list_tasks", "db_read", json!({"milestone_id": milestone, "status": status, "kind": kind})),
            TaskCmd::Get { id } => ("portfolio.get_task", "db_read", json!({"id": id})),
        },
        Command::Attempt { cmd } => match cmd {
            AttemptCmd::Submit { task, result, output, feedback } => ("portfolio.submit_attempt", "db_write", json!({"task_id": task, "result": result, "output": output, "feedback": feedback})),
            AttemptCmd::Resolve { id, example, note } => ("portfolio.resolve_attempt", "db_write", json!({"id": id, "example": example, "note": note})),
            AttemptCmd::List { task, limit } => ("portfolio.list_attempts", "db_read", json!({"task_id": task, "limit": limit})),
            AttemptCmd::Get { id } => ("portfolio.get_attempt", "db_read", json!({"id": id})),
        },
        Command::Skill { cmd } => match cmd {
            SkillCmd::Record { name, domain, level, evidence, confidence } => ("portfolio.record_skill", "db_write", json!({"name": name, "domain": domain, "level": level, "evidence": evidence, "confidence": confidence})),
            SkillCmd::Update { id, level, evidence } => ("portfolio.update_skill", "db_write", json!({"id": id, "level": level, "evidence": evidence})),
            SkillCmd::List { domain } => ("portfolio.list_skills", "db_read", json!({"domain": domain})),
            SkillCmd::Get { id } => ("portfolio.get_skill", "db_read", json!({"id": id})),
        },
        Command::Achievement { cmd } => match cmd {
            AchievementCmd::Award { title, skill, task } => ("portfolio.award_achievement", "db_write", json!({"title": title, "skill_id": skill, "task_id": task})),
            AchievementCmd::List { limit } => ("portfolio.list_achievements", "db_read", json!({"limit": limit})),
            AchievementCmd::Get { id } => ("portfolio.get_achievement", "db_read", json!({"id": id})),
        },
        Command::Report { cmd } => match cmd {
            ReportCmd::Generate { period, content } => ("portfolio.generate_report", "db_write", json!({"period": period, "content": content})),
            ReportCmd::List { limit } => ("portfolio.list_reports", "db_read", json!({"limit": limit})),
            ReportCmd::Get { id } => ("portfolio.get_report", "db_read", json!({"id": id})),
        },
        Command::Timeline { cmd } => match cmd {
            TimelineCmd::List { kind, since, limit } => ("portfolio.list_timeline", "db_read", json!({"kind": kind, "since": since, "limit": limit})),
            TimelineCmd::Get { id } => ("portfolio.get_timeline", "db_read", json!({"id": id})),
            TimelineCmd::Add { summary, kind, ref_id, ref_type } => ("portfolio.add_timeline", "db_write", json!({"summary": summary, "kind": kind, "ref_id": ref_id, "ref_type": ref_type})),
        },
    }
}

pub async fn run_from_args(args: Vec<String>) -> anyhow::Result<()> {
    // args[0] is prog, rest are cli args (already stripped of "portfolio" prefix if needed)
    let cli = Cli::parse_from(args);
    let tenant_id = tenant_id_or_fail(&cli)?;
    let (op, rtype, data) = portfolio_op(&cli.command);
    let resp = call(&tenant_id, op, rtype, data, &cli).await?;
    println!("{}", serde_json::to_string_pretty(&resp)?);
    Ok(())
}

/// True when this invocation should act as `portfolio` CLI rather than daemon.
/// Cases: binary name is `portfolio` (symlink), or first arg is portfolio subcommand,
/// or `supervisor portfolio ...` form.
pub fn is_portfolio_invocation(args: &[String]) -> bool {
    let prog = std::path::Path::new(args.first().map(|s| s.as_str()).unwrap_or(""))
        .file_name().and_then(|s| s.to_str()).unwrap_or("");
    if prog == "portfolio" { return true; }
    const SUBS: &[&str] = &["show","update","goal","milestone","task","attempt","skill","achievement","report","timeline","--help","-h","--version"];
    if args.len() >= 2 && SUBS.contains(&args[1].as_str()) { return true; }
    if args.len() >= 2 && args[1] == "portfolio" { return true; }
    false
}

pub fn normalize_portfolio_args(mut args: Vec<String>) -> Vec<String> {
    // `supervisor portfolio show` -> `portfolio show`
    // `portfolio show` already ok, `portfolio` binary already has prog portfolio
    if args.len() >= 2 && args[1] == "portfolio" {
        let mut v = vec![args[0].clone()];
        v.extend(args[2..].iter().cloned());
        if v[0].contains("supervisor") { v[0] = "portfolio".to_string(); }
        return v;
    }
    // invoked as `supervisor show` (no portfolio word) -> inject portfolio prog name for clap
    let prog = std::path::Path::new(&args[0]).file_name().and_then(|s| s.to_str()).unwrap_or("");
    if prog != "portfolio" && args.len() >= 2 && ["show","update","goal","milestone","task","attempt","skill","achievement","report","timeline"].contains(&args[1].as_str()) {
        let mut v = vec!["portfolio".to_string()];
        v.extend(args[1..].iter().cloned());
        return v;
    }
    args
}
