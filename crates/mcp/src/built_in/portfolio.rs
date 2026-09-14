//! Built-in portfolio tools — single family `portfolio.*`, integer ids.

use std::sync::Arc;

use crate::handler::portfolio::{PortfolioOp, PortfolioToolHandler};
use crate::handler::SharedProxy;
use crate::tools::{ToolBuilder, ToolRegistry};

pub fn register(registry: &ToolRegistry, proxy: SharedProxy) -> anyhow::Result<()> {
    let r = |op: PortfolioOp, name: &str, desc: &str, params: Vec<(&str,&str,&str,bool)>| {
        let mut b = ToolBuilder::new(name).description(desc);
        for (k,ty,d,req) in params { b = b.with_param(k, ty, d, req); }
        registry.register(Arc::new(b.build(Arc::new(PortfolioToolHandler::new(op, Arc::clone(&proxy))))))
    };

    r(PortfolioOp::Show, "portfolio.show", "Show portfolio summary (profile, goals, tasks, skills, recent attempts, timeline)", vec![])?;
    r(PortfolioOp::Update, "portfolio.update", "Update portfolio profile (agent_name, nickname, track, language, timezone)", vec![("agent_name","string","Agent name",false),("nickname","string","Nickname",false),("track","string","Track",false),("language","string","Language",false),("timezone","string","Timezone",false)])?;

    r(PortfolioOp::CreateGoal, "portfolio.create_goal", "Create a goal", vec![("title","string","Goal title",true),("track","string","Track",false),("target_date","string","Target date YYYY-MM-DD",false)])?;
    r(PortfolioOp::UpdateGoal, "portfolio.update_goal", "Update a goal", vec![("id","number","Goal id",true),("title","string","New title",false),("status","string","active|done|paused",false),("target_date","string","New target date",false)])?;
    r(PortfolioOp::ListGoals, "portfolio.list_goals", "List goals", vec![("status","string","Status filter",false),("limit","number","Max results",false)])?;
    r(PortfolioOp::GetGoal, "portfolio.get_goal", "Get a goal by id", vec![("id","number","Goal id",true)])?;

    r(PortfolioOp::AddMilestone, "portfolio.add_milestone", "Add milestone under a goal", vec![("goal_id","number","Goal id",true),("title","string","Milestone title",true),("order_index","number","Order",false)])?;
    r(PortfolioOp::ListMilestones, "portfolio.list_milestones", "List milestones for a goal", vec![("goal_id","number","Goal id",true)])?;
    r(PortfolioOp::UpdateMilestone, "portfolio.update_milestone", "Update milestone", vec![("id","number","Milestone id",true),("title","string","New title",false),("status","string","open|done",false)])?;

    r(PortfolioOp::CreateTask, "portfolio.create_task", "Create task under a milestone", vec![("milestone_id","number","Milestone id",true),("title","string","Task title",true),("kind","string","task|question|exercise",true),("description","string","Description",false),("difficulty","string","easy|medium|hard",false)])?;
    r(PortfolioOp::UpdateTask, "portfolio.update_task", "Update task", vec![("id","number","Task id",true),("title","string","New title",false),("status","string","open|done|blocked",false),("description","string","New description",false)])?;
    r(PortfolioOp::ListTasks, "portfolio.list_tasks", "List tasks", vec![("milestone_id","number","Milestone filter",false),("status","string","Status filter",false),("kind","string","Kind filter",false)])?;
    r(PortfolioOp::GetTask, "portfolio.get_task", "Get task by id", vec![("id","number","Task id",true)])?;

    r(PortfolioOp::SubmitAttempt, "portfolio.submit_attempt", "Submit attempt for a task", vec![("task_id","number","Task id",true),("result","string","pass|fail|partial",true),("output","string","Student output",true),("feedback","string","Feedback",false)])?;
    r(PortfolioOp::ResolveAttempt, "portfolio.resolve_attempt", "Resolve attempt with example solution", vec![("id","number","Attempt id",true),("example","string","Example solution",true),("note","string","Note",false)])?;
    r(PortfolioOp::ListAttempts, "portfolio.list_attempts", "List attempts for a task", vec![("task_id","number","Task id",true),("limit","number","Max results",false)])?;
    r(PortfolioOp::GetAttempt, "portfolio.get_attempt", "Get attempt by id", vec![("id","number","Attempt id",true)])?;

    r(PortfolioOp::RecordSkill, "portfolio.record_skill", "Record a skill", vec![("name","string","Skill name",true),("domain","string","Domain",false),("level","string","novice|practicing|skilled|master",false),("evidence","number","Attempt id evidence",false),("confidence","number","Confidence 0..1",false)])?;
    r(PortfolioOp::UpdateSkill, "portfolio.update_skill", "Update skill", vec![("id","number","Skill id",true),("level","string","New level",false),("evidence","number","New evidence attempt id",false)])?;
    r(PortfolioOp::ListSkills, "portfolio.list_skills", "List skills", vec![("domain","string","Domain filter",false)])?;
    r(PortfolioOp::GetSkill, "portfolio.get_skill", "Get skill by id", vec![("id","number","Skill id",true)])?;

    r(PortfolioOp::AwardAchievement, "portfolio.award_achievement", "Award achievement", vec![("title","string","Title",true),("skill_id","number","Skill id",false),("task_id","number","Task id",false)])?;
    r(PortfolioOp::ListAchievements, "portfolio.list_achievements", "List achievements", vec![("limit","number","Max results",false)])?;
    r(PortfolioOp::GetAchievement, "portfolio.get_achievement", "Get achievement by id", vec![("id","number","Achievement id",true)])?;

    r(PortfolioOp::GenerateReport, "portfolio.generate_report", "Generate portfolio report", vec![("period","string","weekly|monthly",true),("content","string","Content",false)])?;
    r(PortfolioOp::ListReports, "portfolio.list_reports", "List reports", vec![("limit","number","Max results",false)])?;
    r(PortfolioOp::GetReport, "portfolio.get_report", "Get report by id", vec![("id","number","Report id",true)])?;

    r(PortfolioOp::ListTimeline, "portfolio.list_timeline", "List timeline diary", vec![("kind","string","Kind filter",false),("since","string","Since RFC3339",false),("limit","number","Max results",false)])?;
    r(PortfolioOp::GetTimeline, "portfolio.get_timeline", "Get timeline entry by id", vec![("id","number","Timeline id",true)])?;
    r(PortfolioOp::AddTimeline, "portfolio.add_timeline", "Add timeline note", vec![("summary","string","Summary",true),("kind","string","Kind",false),("ref_id","number","Ref id",false),("ref_type","string","Ref type",false)])?;
    Ok(())
}
