import { Alert, Space, Tag, Typography } from 'antd'
import type { AiRunResult } from '../api/ai'

type AiRunResultCardProps = {
  result: AiRunResult
}

export default function AiRunResultCard({ result }: AiRunResultCardProps) {
  return (
    <section className="result-panel ai-result-panel">
      <Space direction="vertical" size={18} style={{ display: 'flex' }}>
        <Space wrap>
          <Tag color="blue">{result.status}</Tag>
          <Tag>{result.parse_status}</Tag>
          <Tag>{result.run_id}</Tag>
        </Space>

        <div>
          <Typography.Title level={4} className="result-panel-title">
            Processing Summary
          </Typography.Title>
          <Typography.Paragraph className="ai-result-copy">
            {result.summary}
          </Typography.Paragraph>
        </div>

        <div>
          <Typography.Text className="dossier-section-label">Action Preview</Typography.Text>
          <Typography.Paragraph className="ai-result-copy">
            {result.action_preview}
          </Typography.Paragraph>
        </div>

        {result.failure_reason ? (
          <Alert
            type="error"
            showIcon
            message="Failure Reason"
            description={result.failure_reason}
          />
        ) : null}

        {result.clarification_question ? (
          <Alert
            type="warning"
            showIcon
            message="Clarification Prompt"
            description={result.clarification_question}
          />
        ) : null}
      </Space>
    </section>
  )
}
