use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use toon_format::{decode_default, encode_default};

use crate::domain::ai::{
    AiDecisionInput, AiDecisionOutput, AiExecutionOutcome, AiExecutionStatus, AiIntent,
    AiToolInput, AiToolOutput, AiUnderstandingInput, AiUnderstandingOutput, AiRunContext,
    AiRunRecord, AiRunResult, ValidationOutcome,
};
pub use crate::config::ModelPayloadEncoding;
use crate::error::AppError;

#[async_trait]
pub trait AiUnderstandingProvider: Send + Sync {
    async fn understand_input(
        &self,
        input: AiUnderstandingInput,
    ) -> Result<AiUnderstandingOutput, AppError>;
}

#[async_trait]
pub trait AiDecisionProvider: Send + Sync {
    async fn decide_input(&self, input: AiDecisionInput) -> Result<AiDecisionOutput, AppError>;
}

#[async_trait]
pub trait AiToolProvider: Send + Sync {
    async fn call(&self, input: AiToolInput) -> Result<AiToolOutput, AppError>;
}

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

pub fn encode_model_payload(
    encoding: ModelPayloadEncoding,
    payload: &str,
) -> Result<String, AppError> {
    match encoding {
        ModelPayloadEncoding::Json => {
            serde_json::from_str::<Value>(payload).map_err(|error| AppError::AiDecode {
                stage: "encode",
                encoding: "json",
                message: format!("invalid json payload: {error}"),
            })?;
            Ok(payload.to_string())
        }
        ModelPayloadEncoding::Toon => {
            let trimmed = payload.trim();
            if trimmed.is_empty() {
                return Err(AppError::AiDecode {
                    stage: "encode",
                    encoding: "toon",
                    message: "toon payload cannot be empty".to_string(),
                });
            }

            if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
                encode_default(&value).map_err(|error| AppError::AiDecode {
                    stage: "encode",
                    encoding: "toon",
                    message: format!("invalid toon payload: {error}"),
                })
            } else {
                decode_default::<Value>(trimmed).map_err(|error| AppError::AiDecode {
                    stage: "encode",
                    encoding: "toon",
                    message: format!("invalid toon payload: {error}"),
                })?;
                Ok(trimmed.to_string())
            }
        }
    }
}

pub fn decode_model_payload(
    encoding: ModelPayloadEncoding,
    payload: &str,
) -> Result<String, AppError> {
    match encoding {
        ModelPayloadEncoding::Json => {
            serde_json::from_str::<Value>(payload).map_err(|error| AppError::AiDecode {
                stage: "decode",
                encoding: "json",
                message: format!("invalid json payload: {error}"),
            })?;
            Ok(payload.to_string())
        }
        ModelPayloadEncoding::Toon => {
            let trimmed = payload.trim();
            if trimmed.is_empty() {
                return Err(AppError::AiDecode {
                    stage: "decode",
                    encoding: "toon",
                    message: "toon payload cannot be empty".to_string(),
                });
            }

            decode_default::<Value>(trimmed).map_err(|error| AppError::AiDecode {
                stage: "decode",
                encoding: "toon",
                message: format!("invalid toon payload: {error}"),
            })?;
            Ok(trimmed.to_string())
        }
    }
}

pub fn validate_understanding_output(
    output: &AiUnderstandingOutput,
) -> Result<(), AppError> {
    if output.target_module.trim().is_empty() {
        return Err(AppError::AiSchema {
            stage: "understanding",
            schema: "AiUnderstandingOutput",
            message: "target_module cannot be empty".to_string(),
        });
    }

    if output.confidence > 100 {
        return Err(AppError::AiSchema {
            stage: "understanding",
            schema: "AiUnderstandingOutput",
            message: "confidence must be between 0 and 100".to_string(),
        });
    }

    Ok(())
}

pub fn validate_decision_output(output: &AiDecisionOutput) -> Result<(), AppError> {
    if output.decision_type.trim().is_empty() {
        return Err(AppError::AiSchema {
            stage: "decision",
            schema: "AiDecisionOutput",
            message: "decision_type cannot be empty".to_string(),
        });
    }

    if output.module.trim().is_empty() {
        return Err(AppError::AiSchema {
            stage: "decision",
            schema: "AiDecisionOutput",
            message: "module cannot be empty".to_string(),
        });
    }

    Ok(())
}

pub fn validate_tool_output(output: &AiToolOutput) -> Result<(), AppError> {
    if output.normalized_value.trim().is_empty() {
        return Err(AppError::AiSchema {
            stage: "tool",
            schema: "AiToolOutput",
            message: "normalized_value cannot be empty".to_string(),
        });
    }

    if output.cache_key.trim().is_empty() {
        return Err(AppError::AiSchema {
            stage: "tool",
            schema: "AiToolOutput",
            message: "cache_key cannot be empty".to_string(),
        });
    }

    if output.confidence > 100 {
        return Err(AppError::AiSchema {
            stage: "tool",
            schema: "AiToolOutput",
            message: "confidence must be between 0 and 100".to_string(),
        });
    }

    Ok(())
}

pub async fn retry_understanding_validation(
    provider: &dyn AiUnderstandingProvider,
    input: AiUnderstandingInput,
    max_attempts: usize,
) -> Result<AiUnderstandingOutput, AppError> {
    for attempt in 1..=max_attempts {
        let output = provider.understand_input(input.clone()).await?;
        match validate_understanding_output(&output) {
            Ok(()) => return Ok(output),
            Err(error) => {
                if attempt == max_attempts {
                    return Err(AppError::AiRetryExhausted {
                        stage: "understanding",
                        attempts: max_attempts,
                        message: error.to_string(),
                    });
                }
            }
        }
    }

    Err(AppError::AiRetryExhausted {
        stage: "understanding",
        attempts: max_attempts,
        message: "understanding validation exhausted".to_string(),
    })
}

pub async fn retry_decision_validation(
    provider: &dyn AiDecisionProvider,
    input: AiDecisionInput,
    max_attempts: usize,
) -> Result<AiDecisionOutput, AppError> {
    for attempt in 1..=max_attempts {
        let output = provider.decide_input(input.clone()).await?;
        match validate_decision_output(&output) {
            Ok(()) => return Ok(output),
            Err(error) => {
                if attempt == max_attempts {
                    return Err(AppError::AiRetryExhausted {
                        stage: "decision",
                        attempts: max_attempts,
                        message: error.to_string(),
                    });
                }
            }
        }
    }

    Err(AppError::AiRetryExhausted {
        stage: "decision",
        attempts: max_attempts,
        message: "decision validation exhausted".to_string(),
    })
}

fn clone_app_error(error: &AppError) -> AppError {
    match error {
        AppError::Config(message) => AppError::Config(message.clone()),
        AppError::Validation(message) => AppError::Validation(message.clone()),
        AppError::AiDecode {
            stage,
            encoding,
            message,
        } => AppError::AiDecode {
            stage,
            encoding,
            message: message.clone(),
        },
        AppError::AiSchema {
            stage,
            schema,
            message,
        } => AppError::AiSchema {
            stage,
            schema,
            message: message.clone(),
        },
        AppError::AiRetryExhausted {
            stage,
            attempts,
            message,
        } => AppError::AiRetryExhausted {
            stage,
            attempts: *attempts,
            message: message.clone(),
        },
        AppError::NotFound(message) => AppError::NotFound(message.clone()),
        AppError::InternalState(message) => AppError::InternalState(message.clone()),
        AppError::Database(_) => {
            AppError::InternalState("database error not supported in fake".to_string())
        }
        AppError::Migration(_) => {
            AppError::InternalState("migration error not supported in fake".to_string())
        }
        AppError::Internal => AppError::Internal,
    }
}

pub struct FakeAiUnderstander {
    response: MutexLike<Result<AiUnderstandingOutput, AppError>>,
}

enum MutexLike<T> {
    Ready(T),
}

impl FakeAiUnderstander {
    pub fn with_response(response: AiUnderstandingOutput) -> Self {
        Self {
            response: MutexLike::Ready(Ok(response)),
        }
    }

    pub fn with_error(error: AppError) -> Self {
        Self {
            response: MutexLike::Ready(Err(error)),
        }
    }
}

pub struct FakeAiDecisionEngine {
    response: StoredResult<AiDecisionOutput>,
}

pub struct FakeAiToolProvider {
    response: StoredResult<AiToolOutput>,
}

enum StoredResult<T> {
    Ready(Result<T, AppError>),
}

impl FakeAiDecisionEngine {
    pub fn with_response(response: AiDecisionOutput) -> Self {
        Self {
            response: StoredResult::Ready(Ok(response)),
        }
    }

    pub fn with_error(error: AppError) -> Self {
        Self {
            response: StoredResult::Ready(Err(error)),
        }
    }
}

impl FakeAiToolProvider {
    pub fn with_response(response: AiToolOutput) -> Self {
        Self {
            response: StoredResult::Ready(Ok(response)),
        }
    }

    pub fn with_error(error: AppError) -> Self {
        Self {
            response: StoredResult::Ready(Err(error)),
        }
    }
}

#[async_trait]
impl AiUnderstandingProvider for FakeAiUnderstander {
    async fn understand_input(
        &self,
        _input: AiUnderstandingInput,
    ) -> Result<AiUnderstandingOutput, AppError> {
        match &self.response {
            MutexLike::Ready(Ok(output)) => Ok(output.clone()),
            MutexLike::Ready(Err(error)) => Err(clone_app_error(error)),
        }
    }
}

#[async_trait]
impl AiDecisionProvider for FakeAiDecisionEngine {
    async fn decide_input(&self, _input: AiDecisionInput) -> Result<AiDecisionOutput, AppError> {
        match &self.response {
            StoredResult::Ready(Ok(output)) => Ok(output.clone()),
            StoredResult::Ready(Err(error)) => Err(clone_app_error(error)),
        }
    }
}

#[async_trait]
impl AiToolProvider for FakeAiToolProvider {
    async fn call(&self, _input: AiToolInput) -> Result<AiToolOutput, AppError> {
        match &self.response {
            StoredResult::Ready(Ok(output)) => Ok(output.clone()),
            StoredResult::Ready(Err(error)) => Err(clone_app_error(error)),
        }
    }
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
        AiDecisionInput, AiDecisionOutput, AiExecutionOutcome, AiExecutionStatus, AiIntent,
        AiToolInput, AiToolKind, AiToolOutput, AiUnderstandingInput, AiUnderstandingOutput,
        AiRunContext, AiRunRecord, AiRunResult, ValidationOutcome,
    };
    use crate::service::ai::{
        decode_model_payload, encode_model_payload, retry_decision_validation,
        retry_understanding_validation, validate_decision_output, validate_tool_output,
        validate_understanding_output, ModelPayloadEncoding,
    };
    use crate::error::AppError;
    use crate::service::ai::{
        AiDecisionEngine, AiDecisionProvider, AiExecutor, AiRunner, AiUnderstander,
        AiToolProvider, AiUnderstandingProvider, AiValidator, FakeAiDecisionEngine,
        FakeAiToolProvider, FakeAiUnderstander,
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

    #[tokio::test]
    async fn fake_understander_returns_record_understanding_output() {
        let understander = FakeAiUnderstander::with_response(AiUnderstandingOutput {
            intent: AiIntent::Record,
            target_module: "diet".to_string(),
            references: vec![],
            extracted_entities: vec!["meal".to_string()],
            confidence: 91,
            needs_clarification: false,
            clarification_question: None,
        });

        let result = understander
            .understand_input(AiUnderstandingInput {
                raw_log_id: "log-1".to_string(),
                user_id: "user-1".to_string(),
                message_text: "晚饭吃了鸡胸肉".to_string(),
                channel: "web".to_string(),
                context_date: Some("2026-03-26".to_string()),
                timezone: Some("Asia/Shanghai".to_string()),
            })
            .await
            .expect("understanding should succeed");

        assert_eq!(result.intent, AiIntent::Record);
        assert_eq!(result.target_module, "diet");
        assert_eq!(result.extracted_entities, vec!["meal".to_string()]);
    }

    #[tokio::test]
    async fn fake_understander_can_return_update_and_clarification_result() {
        let understander = FakeAiUnderstander::with_response(AiUnderstandingOutput {
            intent: AiIntent::Update,
            target_module: "diet".to_string(),
            references: vec!["last_meal".to_string()],
            extracted_entities: vec!["meal_type:lunch".to_string()],
            confidence: 72,
            needs_clarification: true,
            clarification_question: Some("你是要把最近一餐改成午饭吗？".to_string()),
        });

        let result = understander
            .understand_input(AiUnderstandingInput {
                raw_log_id: "log-2".to_string(),
                user_id: "user-1".to_string(),
                message_text: "把刚才那条晚饭改成午饭".to_string(),
                channel: "telegram".to_string(),
                context_date: Some("2026-03-26".to_string()),
                timezone: Some("Asia/Shanghai".to_string()),
            })
            .await
            .expect("understanding should succeed");

        assert_eq!(result.intent, AiIntent::Update);
        assert!(result.needs_clarification);
        assert_eq!(
            result.clarification_question.as_deref(),
            Some("你是要把最近一餐改成午饭吗？")
        );
    }

    #[tokio::test]
    async fn fake_understander_surface_provider_failure_without_hiding_it() {
        let understander =
            FakeAiUnderstander::with_error(AppError::InternalState("provider unavailable".to_string()));

        let error = understander
            .understand_input(AiUnderstandingInput {
                raw_log_id: "log-3".to_string(),
                user_id: "user-1".to_string(),
                message_text: "最近我作息怎么样".to_string(),
                channel: "web".to_string(),
                context_date: None,
                timezone: None,
            })
            .await
            .expect_err("understanding should fail");

        match error {
            AppError::InternalState(message) => {
                assert!(message.contains("provider unavailable"));
            }
            other => panic!("expected internal state error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fake_decision_engine_returns_structured_query_plan() {
        let engine = FakeAiDecisionEngine::with_response(AiDecisionOutput {
            decision_type: "query_only".to_string(),
            module: "ledger".to_string(),
            action_count: 1,
        });

        let result = engine
            .decide_input(AiDecisionInput {
                understanding: AiUnderstandingOutput {
                    intent: AiIntent::Query,
                    target_module: "ledger".to_string(),
                    references: vec!["this_month".to_string()],
                    extracted_entities: vec!["metric:expense_total".to_string()],
                    confidence: 92,
                    needs_clarification: false,
                    clarification_question: None,
                },
                state_summary: "ledger data available".to_string(),
            })
            .await
            .expect("decision should succeed");

        assert_eq!(result.decision_type, "query_only");
        assert_eq!(result.module, "ledger");
    }

    #[tokio::test]
    async fn fake_decision_engine_supports_mutation_plan_without_reusing_raw_message() {
        let engine = FakeAiDecisionEngine::with_response(AiDecisionOutput {
            decision_type: "apply_mutation".to_string(),
            module: "diet".to_string(),
            action_count: 2,
        });

        let result = engine
            .decide_input(AiDecisionInput {
                understanding: AiUnderstandingOutput {
                    intent: AiIntent::Update,
                    target_module: "diet".to_string(),
                    references: vec!["last_meal".to_string()],
                    extracted_entities: vec!["meal_type:lunch".to_string()],
                    confidence: 81,
                    needs_clarification: false,
                    clarification_question: None,
                },
                state_summary: "one recent meal record found".to_string(),
            })
            .await
            .expect("decision should succeed");

        assert_eq!(result.decision_type, "apply_mutation");
        assert_eq!(result.action_count, 2);
    }

    #[tokio::test]
    async fn fake_decision_engine_surfaces_provider_failure() {
        let engine =
            FakeAiDecisionEngine::with_error(AppError::InternalState("decision failed".to_string()));

        let error = engine
            .decide_input(AiDecisionInput {
                understanding: AiUnderstandingOutput {
                    intent: AiIntent::Suggest,
                    target_module: "inventory".to_string(),
                    references: vec![],
                    extracted_entities: vec!["goal:dinner".to_string()],
                    confidence: 78,
                    needs_clarification: false,
                    clarification_question: None,
                },
                state_summary: "inventory has eggs and tomatoes".to_string(),
            })
            .await
            .expect_err("decision should fail");

        match error {
            AppError::InternalState(message) => {
                assert!(message.contains("decision failed"));
            }
            other => panic!("expected internal state error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fake_tool_provider_returns_structured_food_category_output() {
        let provider = FakeAiToolProvider::with_response(AiToolOutput {
            kind: AiToolKind::FoodCategory,
            normalized_value: "protein.chicken".to_string(),
            confidence: 96,
            cache_key: "food:chicken_box".to_string(),
        });

        let result = provider
            .call(AiToolInput {
                kind: AiToolKind::FoodCategory,
                payload: "鸡胸肉便当".to_string(),
            })
            .await
            .expect("tool call should succeed");

        assert_eq!(result.kind, AiToolKind::FoodCategory);
        assert_eq!(result.normalized_value, "protein.chicken");
    }

    #[tokio::test]
    async fn fake_tool_provider_returns_structured_bill_category_output() {
        let provider = FakeAiToolProvider::with_response(AiToolOutput {
            kind: AiToolKind::BillCategory,
            normalized_value: "transport.taxi".to_string(),
            confidence: 91,
            cache_key: "bill:taxi".to_string(),
        });

        let result = provider
            .call(AiToolInput {
                kind: AiToolKind::BillCategory,
                payload: "滴滴打车 48".to_string(),
            })
            .await
            .expect("tool call should succeed");

        assert_eq!(result.kind, AiToolKind::BillCategory);
        assert_eq!(result.normalized_value, "transport.taxi");
    }

    #[tokio::test]
    async fn fake_tool_provider_surfaces_tool_failure() {
        let provider =
            FakeAiToolProvider::with_error(AppError::InternalState("tool failed".to_string()));

        let error = provider
            .call(AiToolInput {
                kind: AiToolKind::BillCategory,
                payload: "瑞幸 26".to_string(),
            })
            .await
            .expect_err("tool call should fail");

        match error {
            AppError::InternalState(message) => {
                assert!(message.contains("tool failed"));
            }
            other => panic!("expected internal state error, got {other:?}"),
        }
    }

    #[test]
    fn model_payload_encoding_supports_json_and_toon() {
        assert_eq!(ModelPayloadEncoding::Json.as_str(), "json");
        assert_eq!(ModelPayloadEncoding::Toon.as_str(), "toon");
    }

    #[test]
    fn encode_model_payload_uses_json_encoding_when_requested() {
        let encoded = encode_model_payload(
            ModelPayloadEncoding::Json,
            r#"{"intent":"record","target_module":"diet"}"#,
        )
        .expect("json encoding should succeed");

        assert_eq!(encoded, r#"{"intent":"record","target_module":"diet"}"#);
    }

    #[test]
    fn encode_model_payload_marks_toon_payload_without_changing_internal_semantics() {
        let encoded = encode_model_payload(
            ModelPayloadEncoding::Toon,
            "intent: record\ntarget_module: diet",
        )
        .expect("toon encoding should succeed");

        assert_eq!(encoded, "intent: record\ntarget_module: diet");
    }

    #[test]
    fn decode_model_payload_accepts_json_without_treating_it_as_internal_model_change() {
        let decoded = decode_model_payload(
            ModelPayloadEncoding::Json,
            r#"{"intent":"record","target_module":"diet"}"#,
        )
        .expect("json decoding should succeed");

        assert_eq!(decoded, r#"{"intent":"record","target_module":"diet"}"#);
    }

    #[test]
    fn decode_model_payload_accepts_toon_before_schema_validation() {
        let decoded = decode_model_payload(
            ModelPayloadEncoding::Toon,
            "intent: record\ntarget_module: diet",
        )
        .expect("toon decoding should succeed");

        assert_eq!(decoded, "intent: record\ntarget_module: diet");
    }

    #[test]
    fn decode_model_payload_rejects_empty_toon_payload() {
        let error = decode_model_payload(ModelPayloadEncoding::Toon, "   ")
            .expect_err("empty toon should fail");

        match error {
            AppError::AiDecode {
                stage,
                encoding,
                message,
            } => {
                assert_eq!(stage, "decode");
                assert_eq!(encoding, "toon");
                assert!(message.contains("toon payload cannot be empty"));
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn decode_stage_returns_distinct_error_for_json_decode_failure() {
        let error = decode_model_payload(ModelPayloadEncoding::Json, "{invalid json")
            .expect_err("invalid json should fail");

        match error {
            AppError::AiDecode {
                stage,
                encoding,
                message,
            } => {
                assert_eq!(stage, "decode");
                assert_eq!(encoding, "json");
                assert!(message.contains("invalid json payload"));
            }
            other => panic!("expected ai decode error, got {other:?}"),
        }
    }

    #[test]
    fn decode_stage_returns_distinct_error_for_toon_decode_failure() {
        let error = decode_model_payload(ModelPayloadEncoding::Toon, "::::")
            .expect_err("invalid toon should fail");

        match error {
            AppError::AiDecode {
                stage,
                encoding,
                message,
            } => {
                assert_eq!(stage, "decode");
                assert_eq!(encoding, "toon");
                assert!(message.contains("invalid toon payload"));
            }
            other => panic!("expected ai decode error, got {other:?}"),
        }
    }

    #[test]
    fn understanding_schema_validation_rejects_missing_target_module() {
        let error = validate_understanding_output(&AiUnderstandingOutput {
            intent: AiIntent::Record,
            target_module: "".to_string(),
            references: vec![],
            extracted_entities: vec![],
            confidence: 91,
            needs_clarification: false,
            clarification_question: None,
        })
        .expect_err("missing target module should fail");

        match error {
            AppError::AiSchema {
                stage,
                schema,
                message,
            } => {
                assert_eq!(stage, "understanding");
                assert_eq!(schema, "AiUnderstandingOutput");
                assert!(message.contains("target_module"));
            }
            other => panic!("expected ai schema error, got {other:?}"),
        }
    }

    #[test]
    fn decision_schema_validation_rejects_missing_decision_type() {
        let error = validate_decision_output(&AiDecisionOutput {
            decision_type: "".to_string(),
            module: "diet".to_string(),
            action_count: 1,
        })
        .expect_err("missing decision_type should fail");

        match error {
            AppError::AiSchema {
                stage,
                schema,
                message,
            } => {
                assert_eq!(stage, "decision");
                assert_eq!(schema, "AiDecisionOutput");
                assert!(message.contains("decision_type"));
            }
            other => panic!("expected ai schema error, got {other:?}"),
        }
    }

    #[test]
    fn tool_schema_validation_rejects_missing_normalized_value() {
        let error = validate_tool_output(&AiToolOutput {
            kind: AiToolKind::BillCategory,
            normalized_value: "".to_string(),
            confidence: 94,
            cache_key: "bill:luckin".to_string(),
        })
        .expect_err("missing normalized_value should fail");

        match error {
            AppError::AiSchema {
                stage,
                schema,
                message,
            } => {
                assert_eq!(stage, "tool");
                assert_eq!(schema, "AiToolOutput");
                assert!(message.contains("normalized_value"));
            }
            other => panic!("expected ai schema error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn retry_policy_retries_understanding_schema_failure_and_then_succeeds() {
        struct FlakyUnderstander {
            calls: Mutex<usize>,
        }

        #[async_trait]
        impl AiUnderstandingProvider for FlakyUnderstander {
            async fn understand_input(
                &self,
                _input: AiUnderstandingInput,
            ) -> Result<AiUnderstandingOutput, AppError> {
                let mut calls = self.calls.lock().expect("mutex should not be poisoned");
                *calls += 1;

                if *calls == 1 {
                    return Ok(AiUnderstandingOutput {
                        intent: AiIntent::Record,
                        target_module: "".to_string(),
                        references: vec![],
                        extracted_entities: vec![],
                        confidence: 80,
                        needs_clarification: false,
                        clarification_question: None,
                    });
                }

                Ok(AiUnderstandingOutput {
                    intent: AiIntent::Record,
                    target_module: "diet".to_string(),
                    references: vec![],
                    extracted_entities: vec!["meal".to_string()],
                    confidence: 92,
                    needs_clarification: false,
                    clarification_question: None,
                })
            }
        }

        let provider = FlakyUnderstander {
            calls: Mutex::new(0),
        };

        let result = retry_understanding_validation(
            &provider,
            AiUnderstandingInput {
                raw_log_id: "log-1".to_string(),
                user_id: "user-1".to_string(),
                message_text: "晚饭吃了鸡胸肉".to_string(),
                channel: "web".to_string(),
                context_date: Some("2026-03-26".to_string()),
                timezone: Some("Asia/Shanghai".to_string()),
            },
            2,
        )
        .await
        .expect("retry should succeed");

        assert_eq!(result.target_module, "diet");
    }

    #[tokio::test]
    async fn retry_policy_returns_explainable_error_after_exhausting_retries() {
        struct AlwaysInvalidDecision;

        #[async_trait]
        impl AiDecisionProvider for AlwaysInvalidDecision {
            async fn decide_input(
                &self,
                _input: AiDecisionInput,
            ) -> Result<AiDecisionOutput, AppError> {
                Ok(AiDecisionOutput {
                    decision_type: "".to_string(),
                    module: "diet".to_string(),
                    action_count: 1,
                })
            }
        }

        let error = retry_decision_validation(
            &AlwaysInvalidDecision,
            AiDecisionInput {
                understanding: AiUnderstandingOutput {
                    intent: AiIntent::Record,
                    target_module: "diet".to_string(),
                    references: vec![],
                    extracted_entities: vec!["meal".to_string()],
                    confidence: 90,
                    needs_clarification: false,
                    clarification_question: None,
                },
                state_summary: "diet state ready".to_string(),
            },
            2,
        )
        .await
        .expect_err("retry should fail after exhausting retries");

        match error {
            AppError::AiRetryExhausted {
                stage,
                attempts,
                message,
            } => {
                assert_eq!(stage, "decision");
                assert_eq!(attempts, 2);
                assert!(message.contains("decision_type"));
            }
            other => panic!("expected retry exhausted error, got {other:?}"),
        }
    }
}
