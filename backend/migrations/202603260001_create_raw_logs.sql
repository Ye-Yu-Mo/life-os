CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE raw_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    raw_text TEXT NOT NULL,
    input_channel TEXT NOT NULL,
    source_type TEXT NOT NULL,
    context_date DATE,
    timezone TEXT,
    parse_status TEXT NOT NULL DEFAULT 'pending',
    parser_version TEXT,
    parse_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_raw_logs_user_created_at ON raw_logs (user_id, created_at DESC);
CREATE INDEX idx_raw_logs_parse_status_created_at ON raw_logs (parse_status, created_at DESC);
