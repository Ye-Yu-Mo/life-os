use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::ai::{
    AiDecisionOutput, AiExecutionOutcome, AiExecutionStatus, AiIntent, AiRunContext,
    AiRunRecord, AiRunResult, ValidationOutcome,
};
use crate::error::AppError;

#[async_trait]
pub trait AiUnderstander: Send + Sync {
    async fn understand(&self, context: &AiRunContext) -> Result<AiIntent, AppError>;
}

#[async_trait]
pub trait AiDecisionEngine: Send + Sync {
    async fn decide(
        &self,
        intent: AiIntent,
        context: &AiRunContext,
    ) -> Result<AiDecisionOutput, AppError>;
}

#[async_trait]
pub trait AiValidator: Send + Sync {
    async fn validate(&self, decision: &AiDecisionOutput) -> Result<ValidationOutcome, AppError>;
}

#[async_trait]
pub trait AiExecutor: Send + Sync {
    async fn execute(
        &self,
        intent: AiIntent,
        decision: AiDecisionOutput,
        context: &AiRunContext,
    ) -> Result<AiExecutionOutcome, AppError>;
}

pub trait AiExecutionOrchestrator {
    fn runner_name(&self) -> &'static str;
}

pub struct AiRunner {
    understander: Arc<dyn AiUnderstander>,
    decision_engine: Arc<dyn AiDecisionEngine>,
    validator: Arc<dyn AiValidator>,
    executor: Arc<dyn AiExecutor>,
}

impl AiRunner {
    pub fn new(
        understander: Arc<dyn AiUnderstander>,
        decision_engine: Arc<dyn AiDecisionEngine>,
        validator: Arc<dyn AiValidator>,
        executor: Arc<dyn AiExecutor>,
    ) -> Self {
        Self {
            understander,
            decision_engine,
            validator,
            executor,
        }
    }

    pub async fn run(&self, context: AiRunContext) -> Result<AiRunResult, AppError> {
        let mut stage_trace = Vec::with_capacity(4);

        stage_trace.push("understand".to_string());
        let intent = self.understander.understand(&context).await?;

        stage_trace.push("decide".to_string());
        let decision = self.decision_engine.decide(intent, &context).await?;

        stage_trace.push("validate".to_string());
        match self.validator.validate(&decision).await? {
            ValidationOutcome::Accepted => {
                stage_trace.push("execute".to_string());
                let outcome = self.executor.execute(intent, decision, &context).await?;

                Ok(AiRunResult {
                    record: AiRunRecord {
                        run_id: format!("run:{}", context.raw_log_id),
                        raw_log_id: context.raw_log_id,
                        status: AiExecutionStatus::Completed,
                        attempts: 1,
                        stage_trace,
                    },
                    outcome,
                })
            }
            ValidationOutcome::Rejected { reason } => Ok(AiRunResult {
                record: AiRunRecord {
                    run_id: format!("run:{}", context.raw_log_id),
                    raw_log_id: context.raw_log_id,
                    status: AiExecutionStatus::Rejected,
                    attempts: 1,
                    stage_trace,
                },
                outcome: AiExecutionOutcome::Rejected { reason },
            }),
        }
    }
}

impl AiExecutionOrchestrator for AiRunner {
    fn runner_name(&self) -> &'static str {
        "ai_runner"
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use crate::domain::ai::{
        AiDecisionOutput, AiExecutionOutcome, AiExecutionStatus, AiIntent, AiRunContext,
        AiRunRecord, AiRunResult, ValidationOutcome,
    };
    use crate::error::AppError;
    use crate::service::ai::{
        AiDecisionEngine, AiExecutor, AiRunner, AiUnderstander, AiValidator,
    };

    #[derive(Default)]
    struct TraceLog {
        entries: Mutex<Vec<String>>,
    }

    struct FakeUnderstander {
        trace: Arc<TraceLog>,
    }

    #[async_trait]
    impl AiUnderstander for FakeUnderstander {
        async fn understand(&self, _context: &AiRunContext) -> Result<AiIntent, AppError> {
            self.trace
                .entries
                .lock()
                .expect("mutex should not be poisoned")
                .push("understand".to_string());

            Ok(AiIntent::Record)
        }
    }

    struct FakeDecisionEngine {
        trace: Arc<TraceLog>,
    }

    #[async_trait]
    impl AiDecisionEngine for FakeDecisionEngine {
        async fn decide(
            &self,
            intent: AiIntent,
            _context: &AiRunContext,
        ) -> Result<AiDecisionOutput, AppError> {
            self.trace
                .entries
                .lock()
                .expect("mutex should not be poisoned")
                .push("decide".to_string());

            assert_eq!(intent, AiIntent::Record);

            Ok(AiDecisionOutput {
                decision_type: "apply_mutation".to_string(),
                module: "routine".to_string(),
                action_count: 1,
            })
        }
    }

    struct FakeValidator {
        trace: Arc<TraceLog>,
    }

    #[async_trait]
    impl AiValidator for FakeValidator {
        async fn validate(
            &self,
            decision: &AiDecisionOutput,
        ) -> Result<ValidationOutcome, AppError> {
            self.trace
                .entries
                .lock()
                .expect("mutex should not be poisoned")
                .push("validate".to_string());

            assert_eq!(decision.module, "routine");

            Ok(ValidationOutcome::Accepted)
        }
    }

    struct FakeExecutor {
        trace: Arc<TraceLog>,
    }

    #[async_trait]
    impl AiExecutor for FakeExecutor {
        async fn execute(
            &self,
            intent: AiIntent,
            decision: AiDecisionOutput,
            _context: &AiRunContext,
        ) -> Result<AiExecutionOutcome, AppError> {
            self.trace
                .entries
                .lock()
                .expect("mutex should not be poisoned")
                .push("execute".to_string());

            Ok(AiExecutionOutcome::Applied { intent, decision })
        }
    }

    #[tokio::test]
    async fn orchestrator_runs_understand_decide_validate_execute_in_order() {
        let trace = Arc::new(TraceLog::default());
        let runner = AiRunner::new(
            Arc::new(FakeUnderstander {
                trace: trace.clone(),
            }),
            Arc::new(FakeDecisionEngine {
                trace: trace.clone(),
            }),
            Arc::new(FakeValidator {
                trace: trace.clone(),
            }),
            Arc::new(FakeExecutor {
                trace: trace.clone(),
            }),
        );

        let result = runner
            .run(AiRunContext {
                raw_log_id: "log-1".to_string(),
                user_id: "user-1".to_string(),
                message_text: "今天 9:40 起床".to_string(),
                encoding: "json".to_string(),
            })
            .await
            .expect("run should succeed");

        assert_eq!(
            trace.entries.lock().expect("mutex should not be poisoned").clone(),
            vec!["understand", "decide", "validate", "execute"]
        );
        assert_eq!(result.record.status, AiExecutionStatus::Completed);
    }

    #[tokio::test]
    async fn orchestrator_returns_single_run_record_and_outcome() {
        let trace = Arc::new(TraceLog::default());
        let runner = AiRunner::new(
            Arc::new(FakeUnderstander {
                trace: trace.clone(),
            }),
            Arc::new(FakeDecisionEngine {
                trace: trace.clone(),
            }),
            Arc::new(FakeValidator {
                trace: trace.clone(),
            }),
            Arc::new(FakeExecutor { trace }),
        );

        let result = runner
            .run(AiRunContext {
                raw_log_id: "log-1".to_string(),
                user_id: "user-1".to_string(),
                message_text: "今天 9:40 起床".to_string(),
                encoding: "json".to_string(),
            })
            .await
            .expect("run should succeed");

        assert_eq!(result.record.raw_log_id, "log-1");
        assert_eq!(result.record.attempts, 1);

        match result.outcome {
            AiExecutionOutcome::Applied { intent, decision } => {
                assert_eq!(intent, AiIntent::Record);
                assert_eq!(decision.decision_type, "apply_mutation");
            }
            other => panic!("expected applied outcome, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn orchestrator_stops_before_execution_when_validation_rejects() {
        struct RejectingValidator {
            trace: Arc<TraceLog>,
        }

        #[async_trait]
        impl AiValidator for RejectingValidator {
            async fn validate(
                &self,
                _decision: &AiDecisionOutput,
            ) -> Result<ValidationOutcome, AppError> {
                self.trace
                    .entries
                    .lock()
                    .expect("mutex should not be poisoned")
                    .push("validate".to_string());

                Ok(ValidationOutcome::Rejected {
                    reason: "missing required field".to_string(),
                })
            }
        }

        let trace = Arc::new(TraceLog::default());
        let runner = AiRunner::new(
            Arc::new(FakeUnderstander {
                trace: trace.clone(),
            }),
            Arc::new(FakeDecisionEngine {
                trace: trace.clone(),
            }),
            Arc::new(RejectingValidator {
                trace: trace.clone(),
            }),
            Arc::new(FakeExecutor {
                trace: trace.clone(),
            }),
        );

        let result = runner
            .run(AiRunContext {
                raw_log_id: "log-1".to_string(),
                user_id: "user-1".to_string(),
                message_text: "今天 9:40 起床".to_string(),
                encoding: "json".to_string(),
            })
            .await
            .expect("run should succeed");

        assert_eq!(
            trace.entries.lock().expect("mutex should not be poisoned").clone(),
            vec!["understand", "decide", "validate"]
        );

        match result {
            AiRunResult {
                record:
                    AiRunRecord {
                        status: AiExecutionStatus::Rejected,
                        ..
                    },
                outcome: AiExecutionOutcome::Rejected { reason },
            } => {
                assert!(reason.contains("missing required field"));
            }
            other => panic!("expected rejected run result, got {other:?}"),
        }
    }
}
