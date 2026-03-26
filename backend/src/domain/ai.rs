#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiRunContext {
    pub raw_log_id: String,
    pub user_id: String,
    pub message_text: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiIntent {
    Record,
    Update,
    Query,
    Suggest,
    Command,
    Chat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiExecutionStatus {
    Completed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiDecisionOutput {
    pub decision_type: String,
    pub module: String,
    pub action_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationOutcome {
    Accepted,
    Rejected { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiExecutionOutcome {
    Applied {
        intent: AiIntent,
        decision: AiDecisionOutput,
    },
    Rejected {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiRunRecord {
    pub run_id: String,
    pub raw_log_id: String,
    pub status: AiExecutionStatus,
    pub attempts: usize,
    pub stage_trace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiRunResult {
    pub record: AiRunRecord,
    pub outcome: AiExecutionOutcome,
}

#[cfg(test)]
mod tests {
    use crate::domain::ai::{
        AiDecisionOutput, AiExecutionOutcome, AiExecutionStatus, AiIntent, AiRunContext,
        AiRunRecord, AiRunResult,
    };

    #[test]
    fn ai_run_context_keeps_message_identity_and_transport_encoding() {
        let context = AiRunContext {
            raw_log_id: "log-1".to_string(),
            user_id: "user-1".to_string(),
            message_text: "今天 9:40 起床".to_string(),
            encoding: "toon".to_string(),
        };

        assert_eq!(context.raw_log_id, "log-1");
        assert_eq!(context.user_id, "user-1");
        assert_eq!(context.encoding, "toon");
    }

    #[test]
    fn ai_run_record_tracks_stages_and_retry_count() {
        let record = AiRunRecord {
            run_id: "run-1".to_string(),
            raw_log_id: "log-1".to_string(),
            status: AiExecutionStatus::Completed,
            attempts: 2,
            stage_trace: vec![
                "understand".to_string(),
                "decide".to_string(),
                "validate".to_string(),
                "execute".to_string(),
            ],
        };

        assert_eq!(record.attempts, 2);
        assert_eq!(record.stage_trace.len(), 4);
    }

    #[test]
    fn ai_run_result_has_single_success_and_failure_shape() {
        let success = AiRunResult {
            record: AiRunRecord {
                run_id: "run-1".to_string(),
                raw_log_id: "log-1".to_string(),
                status: AiExecutionStatus::Completed,
                attempts: 1,
                stage_trace: vec![
                    "understand".to_string(),
                    "decide".to_string(),
                    "validate".to_string(),
                    "execute".to_string(),
                ],
            },
            outcome: AiExecutionOutcome::Applied {
                intent: AiIntent::Record,
                decision: AiDecisionOutput {
                    decision_type: "apply_mutation".to_string(),
                    module: "routine".to_string(),
                    action_count: 1,
                },
            },
        };

        match success.outcome {
            AiExecutionOutcome::Applied { intent, decision } => {
                assert_eq!(intent, AiIntent::Record);
                assert_eq!(decision.module, "routine");
                assert_eq!(decision.action_count, 1);
            }
            other => panic!("expected applied outcome, got {other:?}"),
        }
    }
}
