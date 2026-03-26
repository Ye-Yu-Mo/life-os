use async_trait::async_trait;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq)]
pub struct CreateAiRunRecord {
    pub raw_log_id: String,
    pub user_id: String,
    pub status: String,
    pub attempts: i32,
    pub stage_trace: Value,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateAiActionPlanRecord {
    pub plan_kind: String,
    pub module: String,
    pub action_count: i32,
    pub summary: String,
    pub snapshot: Value,
}

#[async_trait]
pub trait AiRunRepository: Send + Sync {
    async fn record_run(
        &self,
        run: CreateAiRunRecord,
        action_plan: Option<CreateAiActionPlanRecord>,
    ) -> Result<String, AppError>;
}

pub struct PgAiRunRepository {
    pool: PgPool,
}

impl PgAiRunRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AiRunRepository for PgAiRunRepository {
    async fn record_run(
        &self,
        run: CreateAiRunRecord,
        action_plan: Option<CreateAiActionPlanRecord>,
    ) -> Result<String, AppError> {
        let mut transaction = self.pool.begin().await?;
        let run_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO ai_runs (
                id,
                raw_log_id,
                user_id,
                status,
                attempts,
                stage_trace,
                error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(run_id)
        .bind(parse_uuid(&run.raw_log_id, "raw_log_id")?)
        .bind(parse_uuid(&run.user_id, "user_id")?)
        .bind(run.status)
        .bind(run.attempts)
        .bind(run.stage_trace)
        .bind(run.error_message)
        .execute(transaction.as_mut())
        .await?;

        if let Some(action_plan) = action_plan {
            sqlx::query(
                r#"
                INSERT INTO ai_action_plans (
                    id,
                    ai_run_id,
                    plan_kind,
                    module,
                    action_count,
                    summary,
                    snapshot
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(run_id)
            .bind(action_plan.plan_kind)
            .bind(action_plan.module)
            .bind(action_plan.action_count)
            .bind(action_plan.summary)
            .bind(action_plan.snapshot)
            .execute(transaction.as_mut())
            .await?;
        }

        transaction.commit().await?;
        Ok(run_id.to_string())
    }
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value)
        .map_err(|error| AppError::Validation(format!("invalid {field}: {error}")))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CreateAiActionPlanRecord, CreateAiRunRecord};

    #[test]
    fn ai_run_snapshot_structs_keep_required_fields() {
        let run = CreateAiRunRecord {
            raw_log_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            status: "failed".to_string(),
            attempts: 1,
            stage_trace: json!(["understand"]),
            error_message: Some("provider unavailable".to_string()),
        };

        let plan = CreateAiActionPlanRecord {
            plan_kind: "apply_mutation".to_string(),
            module: "routine".to_string(),
            action_count: 1,
            summary: "write one routine record".to_string(),
            snapshot: json!({
                "kind": "apply_mutation",
                "module": "routine",
                "action_count": 1,
                "summary": "write one routine record"
            }),
        };

        assert_eq!(run.status, "failed");
        assert_eq!(plan.module, "routine");
    }
}
