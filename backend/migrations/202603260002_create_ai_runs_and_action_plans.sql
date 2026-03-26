CREATE TABLE ai_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    raw_log_id UUID NOT NULL REFERENCES raw_logs (id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    status TEXT NOT NULL,
    attempts INTEGER NOT NULL,
    stage_trace JSONB NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_runs_raw_log_created_at ON ai_runs (raw_log_id, created_at DESC);
CREATE INDEX idx_ai_runs_status_created_at ON ai_runs (status, created_at DESC);

CREATE TABLE ai_action_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ai_run_id UUID NOT NULL REFERENCES ai_runs (id) ON DELETE CASCADE,
    plan_kind TEXT NOT NULL,
    module TEXT NOT NULL,
    action_count INTEGER NOT NULL,
    summary TEXT NOT NULL,
    snapshot JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_action_plans_run_created_at ON ai_action_plans (ai_run_id, created_at DESC);
