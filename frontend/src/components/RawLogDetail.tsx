import { Descriptions, Drawer, Space, Tag, Typography } from 'antd'
import type { RawLog } from '../api/logs'
import { formatInputChannelLabel, formatSourceTypeLabel } from './rawLogLabels'

type RawLogDetailProps = {
  log: RawLog | null
  open: boolean
  loading?: boolean
  onClose: () => void
}

export default function RawLogDetail({
  log,
  open,
  loading = false,
  onClose,
}: RawLogDetailProps) {
  return (
    <Drawer
      title="Raw Log Dossier"
      placement="right"
      width={480}
      open={open}
      onClose={onClose}
      destroyOnClose
      loading={loading}
    >
      {log ? (
        <div className="dossier-stack">
          <section className="dossier-status-card">
            <Space direction="vertical" size={8}>
              <Typography.Text className="dossier-section-label">
                Parse Status
              </Typography.Text>
              <Tag color="blue">{log.parse_status}</Tag>
            </Space>
            <Space wrap>
              <Tag>{formatInputChannelLabel(log.input_channel)}</Tag>
              <Tag>{formatSourceTypeLabel(log.source_type)}</Tag>
            </Space>
          </section>

          <section className="dossier-evidence-card">
            <Typography.Text className="dossier-section-label">
              Raw Text Evidence
            </Typography.Text>
            <Typography.Paragraph className="raw-log-detail-text">
              {log.raw_text}
            </Typography.Paragraph>
          </section>

          {log.ai_result ? (
            <section className="dossier-evidence-card">
              <Typography.Text className="dossier-section-label">
                AI Decision Result
              </Typography.Text>
              <Space direction="vertical" size={12} style={{ display: 'flex' }}>
                <Space wrap>
                  <Tag color="blue">{log.ai_result.status}</Tag>
                  <Tag>{log.parse_status}</Tag>
                </Space>

                <div>
                  <Typography.Text className="dossier-section-label">
                    Processing Summary
                  </Typography.Text>
                  <Typography.Paragraph className="raw-log-detail-text">
                    {log.ai_result.summary}
                  </Typography.Paragraph>
                </div>

                <div>
                  <Typography.Text className="dossier-section-label">
                    Action Preview
                  </Typography.Text>
                  <Typography.Paragraph className="raw-log-detail-text">
                    {log.ai_result.action_preview}
                  </Typography.Paragraph>
                </div>

                {log.ai_result.failure_reason ? (
                  <div>
                    <Typography.Text className="dossier-section-label">
                      Failure Reason
                    </Typography.Text>
                    <Typography.Paragraph className="raw-log-detail-text">
                      {log.ai_result.failure_reason}
                    </Typography.Paragraph>
                  </div>
                ) : null}

                {log.ai_result.retry_summary ? (
                  <div>
                    <Typography.Text className="dossier-section-label">
                      Retry Result
                    </Typography.Text>
                    <Typography.Paragraph className="raw-log-detail-text">
                      {log.ai_result.retry_summary}
                    </Typography.Paragraph>
                  </div>
                ) : null}

                {log.ai_result.clarification_question ? (
                  <div>
                    <Typography.Text className="dossier-section-label">
                      Clarification Prompt
                    </Typography.Text>
                    <Typography.Paragraph className="raw-log-detail-text">
                      {log.ai_result.clarification_question}
                    </Typography.Paragraph>
                  </div>
                ) : null}
              </Space>
            </section>
          ) : null}

          <Descriptions column={1} bordered size="small">
            <Descriptions.Item label="ID">{log.id}</Descriptions.Item>
            <Descriptions.Item label="User ID">{log.user_id}</Descriptions.Item>
            <Descriptions.Item label="Context Date">
              {log.context_date ?? '-'}
            </Descriptions.Item>
            <Descriptions.Item label="Timezone">{log.timezone ?? '-'}</Descriptions.Item>
            <Descriptions.Item label="Created At">{log.created_at}</Descriptions.Item>
            <Descriptions.Item label="Updated At">{log.updated_at}</Descriptions.Item>
          </Descriptions>
        </div>
      ) : null}
    </Drawer>
  )
}
