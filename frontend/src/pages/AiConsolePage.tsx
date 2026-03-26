import { Alert, Button, Form, Input, Space, Tag, Typography } from 'antd'
import { useState } from 'react'
import AiRunResultCard from '../components/AiRunResultCard'
import { submitAiMessage, type AiRunResult } from '../api/ai'
import { getApiErrorMessage } from '../api/logs'

type AiConsoleFormValues = {
  userId: string
  messageText: string
  contextDate?: string
  timezone?: string
}

export default function AiConsolePage() {
  const [form] = Form.useForm<AiConsoleFormValues>()
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [result, setResult] = useState<AiRunResult | null>(null)

  async function handleSubmit(values: AiConsoleFormValues) {
    setIsSubmitting(true)
    setSubmitError(null)

    try {
      const nextResult = await submitAiMessage({
        user_id: values.userId.trim(),
        message_text: values.messageText.trim(),
        context_date: values.contextDate?.trim() || undefined,
        timezone: values.timezone?.trim() || undefined,
      })

      setResult(nextResult)
    } catch (error) {
      setSubmitError(getApiErrorMessage(error))
      setResult(null)
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="page-shell">
      <section className="page-hero">
        <Space direction="vertical" size={14} className="page-hero-copy">
          <Tag className="page-kicker">AI Milestone M3</Tag>
          <Typography.Title level={1} className="page-hero-title">
            AI Console
          </Typography.Title>
          <Typography.Paragraph className="page-hero-text">
            这里不是聊天玩具。它负责显示系统如何理解一条消息，准备执行什么动作，为什么失败，
            或者为什么需要你澄清。
          </Typography.Paragraph>
        </Space>
      </section>

      <section className="feature-card feature-card-accent ai-console-card">
        <div className="feature-card-header">
          <div>
            <Typography.Title level={2} className="feature-card-title">
              AI Message Run
            </Typography.Title>
            <Typography.Paragraph className="feature-card-subtitle">
              先发一条消息，再看系统给出的摘要、动作预览和失败原因。
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
              { whitespace: true, message: 'User ID cannot be empty' },
            ]}
          >
            <Input placeholder="550e8400-e29b-41d4-a716-446655440001" />
          </Form.Item>

          <Form.Item
            label="Message"
            name="messageText"
            rules={[
              { required: true, message: 'Message is required' },
              { whitespace: true, message: 'Message cannot be empty' },
            ]}
          >
            <Input.TextArea
              placeholder="今天 9:40 起床"
              autoSize={{ minRows: 5, maxRows: 8 }}
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
              Run AI
            </Button>
          </Form.Item>
        </Form>

        {submitError ? (
          <Alert type="error" showIcon message="AI Run Failed" description={submitError} />
        ) : null}
      </section>

      {result ? <AiRunResultCard result={result} /> : null}
    </div>
  )
}
