//! Portfolio tool handlers — all `portfolio.*` ops via wrapper proxy.

use std::sync::Arc;

use serde_json::Value;

use crate::tools::handler::{BoxFuture, ToolHandler};

use super::SharedProxy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortfolioOp {
    Show,
    Update,
    CreateGoal,
    UpdateGoal,
    ListGoals,
    GetGoal,
    AddMilestone,
    ListMilestones,
    UpdateMilestone,
    CreateTask,
    UpdateTask,
    ListTasks,
    GetTask,
    SubmitAttempt,
    ResolveAttempt,
    ListAttempts,
    GetAttempt,
    RecordSkill,
    UpdateSkill,
    ListSkills,
    GetSkill,
    AwardAchievement,
    ListAchievements,
    GetAchievement,
    GenerateReport,
    ListReports,
    GetReport,
    ListTimeline,
    GetTimeline,
    AddTimeline,
}

impl PortfolioOp {
    pub fn method(self) -> &'static str {
        match self {
            PortfolioOp::Show => "portfolio.show",
            PortfolioOp::Update => "portfolio.update",
            PortfolioOp::CreateGoal => "portfolio.create_goal",
            PortfolioOp::UpdateGoal => "portfolio.update_goal",
            PortfolioOp::ListGoals => "portfolio.list_goals",
            PortfolioOp::GetGoal => "portfolio.get_goal",
            PortfolioOp::AddMilestone => "portfolio.add_milestone",
            PortfolioOp::ListMilestones => "portfolio.list_milestones",
            PortfolioOp::UpdateMilestone => "portfolio.update_milestone",
            PortfolioOp::CreateTask => "portfolio.create_task",
            PortfolioOp::UpdateTask => "portfolio.update_task",
            PortfolioOp::ListTasks => "portfolio.list_tasks",
            PortfolioOp::GetTask => "portfolio.get_task",
            PortfolioOp::SubmitAttempt => "portfolio.submit_attempt",
            PortfolioOp::ResolveAttempt => "portfolio.resolve_attempt",
            PortfolioOp::ListAttempts => "portfolio.list_attempts",
            PortfolioOp::GetAttempt => "portfolio.get_attempt",
            PortfolioOp::RecordSkill => "portfolio.record_skill",
            PortfolioOp::UpdateSkill => "portfolio.update_skill",
            PortfolioOp::ListSkills => "portfolio.list_skills",
            PortfolioOp::GetSkill => "portfolio.get_skill",
            PortfolioOp::AwardAchievement => "portfolio.award_achievement",
            PortfolioOp::ListAchievements => "portfolio.list_achievements",
            PortfolioOp::GetAchievement => "portfolio.get_achievement",
            PortfolioOp::GenerateReport => "portfolio.generate_report",
            PortfolioOp::ListReports => "portfolio.list_reports",
            PortfolioOp::GetReport => "portfolio.get_report",
            PortfolioOp::ListTimeline => "portfolio.list_timeline",
            PortfolioOp::GetTimeline => "portfolio.get_timeline",
            PortfolioOp::AddTimeline => "portfolio.add_timeline",
        }
    }

    pub fn request_type(self) -> &'static str {
        match self {
            PortfolioOp::Show | PortfolioOp::ListGoals | PortfolioOp::GetGoal | PortfolioOp::ListMilestones | PortfolioOp::ListTasks | PortfolioOp::GetTask | PortfolioOp::ListAttempts | PortfolioOp::GetAttempt | PortfolioOp::ListSkills | PortfolioOp::GetSkill | PortfolioOp::ListAchievements | PortfolioOp::GetAchievement | PortfolioOp::ListReports | PortfolioOp::GetReport | PortfolioOp::ListTimeline | PortfolioOp::GetTimeline => "db_read",
            _ => "db_write",
        }
    }
}

pub struct PortfolioToolHandler {
    pub op: PortfolioOp,
    pub proxy: SharedProxy,
}

impl PortfolioToolHandler {
    pub fn new(op: PortfolioOp, proxy: SharedProxy) -> Self {
        Self { op, proxy }
    }
}

impl ToolHandler for PortfolioToolHandler {
    fn call(&self, params: Value) -> BoxFuture<'static, anyhow::Result<Value>> {
        let op = self.op;
        let method = self.op.method().to_string();
        let proxy = Arc::clone(&self.proxy);
        Box::pin(async move {
            // no strict validation — wrapper validates required fields
            let tenant_id = super::tenant_from_params(&params);
            super::proxy_call(proxy.as_ref(), op.request_type(), &tenant_id, &method, params).await
        })
    }
}
