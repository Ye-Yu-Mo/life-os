import { Alert, Button, Form, Input, Space, Typography } from 'antd'
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
    <Space direction="vertical" size={20} className="log-input-stack">
      <section className="log-input-card">
        <Typography.Title level={2} className="log-input-title">
          Quick Input
        </Typography.Title>
        <Typography.Paragraph className="log-input-subtitle">
          先保留原始表达，再进入结构化链路。这个页面只做一件事：低摩擦提交原始日志。
        </Typography.Paragraph>

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
              placeholder="今天 9:40 起床，凌晨 2:10 睡，睡得一般"
              autoSize={{ minRows: 5, maxRows: 10 }}
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
            <Button type="primary" htmlType="submit" loading={isSubmitting} size="large">
              Submit Raw Log
            </Button>
          </Form.Item>
        </Form>

        {submitError ? (
          <Alert
            type="error"
            showIcon
            message="Submit failed"
            description={submitError}
          />
        ) : null}
      </section>

      {createdLog ? (
        <section className="log-result-card">
          <Typography.Title level={4}>Latest Created Raw Log</Typography.Title>
          <dl className="log-result-grid">
            <dt>ID</dt>
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
    </Space>
  )
}
