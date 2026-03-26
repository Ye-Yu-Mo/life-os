import { Alert, Button, Form, Input, Typography } from 'antd'
import { useState } from 'react'
import { createRawLog, getApiErrorMessage, type RawLog } from '../api/logs'

type LogInputFormValues = {
  userId: string
  rawText: string
  contextDate?: string
  timezone?: string
}

export default function LogInputForm() {
  const [form] = Form.useForm<LogInputFormValues>()
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [createdLog, setCreatedLog] = useState<RawLog | null>(null)
  const [submitError, setSubmitError] = useState<string | null>(null)

  async function handleSubmit(values: LogInputFormValues) {
    setIsSubmitting(true)
    setSubmitError(null)

    try {
      const created = await createRawLog({
        user_id: values.userId.trim(),
        raw_text: values.rawText,
        input_channel: 'web',
        source_type: 'manual',
        context_date: values.contextDate?.trim() || undefined,
        timezone: values.timezone?.trim() || undefined,
      })

      setCreatedLog(created)
      form.resetFields(['rawText', 'contextDate', 'timezone'])
    } catch (error) {
      setSubmitError(getApiErrorMessage(error))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <section className="feature-card feature-card-accent">
      <div className="feature-card-header">
        <div>
          <Typography.Title level={2} className="feature-card-title">
            Quick Capture
          </Typography.Title>
          <Typography.Paragraph className="feature-card-subtitle">
            单条输入走最短路径。别在这里做解析，别在这里做总结，先把原文保存下来。
          </Typography.Paragraph>
        </div>
      </div>

      <Form
        form={form}
        layout="vertical"
        initialValues={{
          userId: '550e8400-e29b-41d4-a716-446655440001',
          timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        }}
        onFinish={handleSubmit}
      >
        <Form.Item
          label="User ID"
          name="userId"
          rules={[
            { required: true, message: 'User ID is required' },
            { type: 'string', min: 1, message: 'User ID cannot be empty' },
          ]}
        >
          <Input placeholder="550e8400-e29b-41d4-a716-446655440001" />
        </Form.Item>

        <Form.Item
          label="Raw Text"
          name="rawText"
          rules={[
            { required: true, message: 'Raw text is required' },
            { whitespace: true, message: 'Raw text cannot be empty' },
            { max: 10000, message: 'Raw text is too long' },
          ]}
        >
          <Input.TextArea
            placeholder="今天 9:40 起床，凌晨 2:10 睡，睡得一般..."
            autoSize={{ minRows: 6, maxRows: 10 }}
            showCount
            maxLength={10000}
          />
        </Form.Item>

        <div className="log-input-meta-grid">
          <Form.Item
            label="Context Date"
            name="contextDate"
            rules={[
              {
                pattern: /^\d{4}-\d{2}-\d{2}$/,
                message: 'Use YYYY-MM-DD format',
              },
            ]}
          >
            <Input placeholder="2026-03-26" />
          </Form.Item>

          <Form.Item label="Timezone" name="timezone">
            <Input placeholder="Asia/Shanghai" />
          </Form.Item>
        </div>

        <Form.Item className="log-input-actions">
          <Button type="primary" htmlType="submit" loading={isSubmitting} size="large" block>
            Save Raw Log
          </Button>
        </Form.Item>
      </Form>

      {submitError ? (
        <Alert type="error" showIcon message="Write Rejected" description={submitError} />
      ) : null}

      {createdLog ? (
        <section className="result-panel">
          <Typography.Title level={4} className="result-panel-title">
            Latest Write Accepted
          </Typography.Title>
          <dl className="log-result-grid">
            <dt>Log ID</dt>
            <dd>{createdLog.id}</dd>
            <dt>Status</dt>
            <dd>{createdLog.parse_status}</dd>
            <dt>Created At</dt>
            <dd>{createdLog.created_at}</dd>
            <dt>Raw Text</dt>
            <dd>{createdLog.raw_text}</dd>
          </dl>
        </section>
      ) : null}
    </section>
  )
}
