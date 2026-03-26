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

impl AiIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Record => "record",
            Self::Update => "update",
            Self::Query => "query",
            Self::Suggest => "suggest",
            Self::Command => "command",
            Self::Chat => "chat",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiUnderstandingInput {
    pub raw_log_id: String,
    pub user_id: String,
    pub message_text: String,
    pub channel: String,
    pub context_date: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiUnderstandingOutput {
    pub intent: AiIntent,
    pub target_module: String,
    pub references: Vec<String>,
    pub extracted_entities: Vec<String>,
    pub confidence: u8,
    pub needs_clarification: bool,
    pub clarification_question: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiDecisionInput {
    pub understanding: AiUnderstandingOutput,
    pub state_summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiToolKind {
    FoodCategory,
    BillCategory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiToolInput {
    pub kind: AiToolKind,
    pub payload: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiToolOutput {
    pub kind: AiToolKind,
    pub normalized_value: String,
    pub confidence: u8,
    pub cache_key: String,
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
        AiDecisionInput, AiDecisionOutput, AiExecutionOutcome, AiExecutionStatus, AiIntent,
        AiToolInput, AiToolKind, AiToolOutput, AiUnderstandingInput, AiUnderstandingOutput,
        AiRunContext, AiRunRecord, AiRunResult,
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

    #[test]
    fn understanding_output_supports_all_supported_intents() {
        let expectations = [
            (AiIntent::Record, "record"),
            (AiIntent::Update, "update"),
            (AiIntent::Query, "query"),
            (AiIntent::Suggest, "suggest"),
            (AiIntent::Command, "command"),
            (AiIntent::Chat, "chat"),
        ];

        for (intent, expected_label) in expectations {
            let output = AiUnderstandingOutput {
                intent,
                target_module: "general".to_string(),
                references: vec![],
                extracted_entities: vec![],
                confidence: 95,
                needs_clarification: false,
                clarification_question: None,
            };

            assert_eq!(output.intent.as_str(), expected_label);
        }
    }

    #[test]
    fn understanding_input_contains_message_and_context_fields() {
        let input = AiUnderstandingInput {
            raw_log_id: "log-1".to_string(),
            user_id: "user-1".to_string(),
            message_text: "把刚才那条晚饭改成午饭".to_string(),
            channel: "telegram".to_string(),
            context_date: Some("2026-03-26".to_string()),
            timezone: Some("Asia/Shanghai".to_string()),
        };

        assert_eq!(input.raw_log_id, "log-1");
        assert_eq!(input.channel, "telegram");
        assert_eq!(input.context_date.as_deref(), Some("2026-03-26"));
    }

    #[test]
    fn decision_input_is_derived_from_understanding_output_and_state_summary() {
        let input = AiDecisionInput {
            understanding: AiUnderstandingOutput {
                intent: AiIntent::Query,
                target_module: "ledger".to_string(),
                references: vec!["this_month".to_string()],
                extracted_entities: vec!["metric:expense_total".to_string()],
                confidence: 93,
                needs_clarification: false,
                clarification_question: None,
            },
            state_summary: "current month expenses exist".to_string(),
        };

        assert_eq!(input.understanding.intent, AiIntent::Query);
        assert_eq!(input.state_summary, "current month expenses exist");
    }

    #[test]
    fn decision_output_is_structured_action_plan_instead_of_free_text() {
        let decision = AiDecisionOutput {
            decision_type: "query_only".to_string(),
            module: "ledger".to_string(),
            action_count: 1,
        };

        assert_eq!(decision.decision_type, "query_only");
        assert_eq!(decision.module, "ledger");
        assert_eq!(decision.action_count, 1);
    }

    #[test]
    fn tool_input_keeps_narrow_kind_and_payload() {
        let input = AiToolInput {
            kind: AiToolKind::FoodCategory,
            payload: "鸡胸肉便当".to_string(),
        };

        assert_eq!(input.kind, AiToolKind::FoodCategory);
        assert_eq!(input.payload, "鸡胸肉便当");
    }

    #[test]
    fn tool_output_is_structured_and_not_free_text() {
        let output = AiToolOutput {
            kind: AiToolKind::BillCategory,
            normalized_value: "food.drink".to_string(),
            confidence: 94,
            cache_key: "bill:luckin".to_string(),
        };

        assert_eq!(output.kind, AiToolKind::BillCategory);
        assert_eq!(output.normalized_value, "food.drink");
        assert_eq!(output.confidence, 94);
    }
}
