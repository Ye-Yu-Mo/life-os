import { render, screen } from '@testing-library/react'
import RawLogDetail from './RawLogDetail'

describe('RawLogDetail', () => {
  it('shows human-readable source labels in the detail drawer', async () => {
    render(
      <RawLogDetail
        open
        onClose={() => {}}
        log={{
          id: 'log-1',
          user_id: 'user-1',
          raw_text: '来自企业微信桥接的日志',
          input_channel: 'wechat_bridge',
          source_type: 'synced',
          context_date: '2026-03-26',
          timezone: 'Asia/Shanghai',
          parse_status: 'pending',
          parser_version: null,
          parse_error: null,
          ai_result: null,
          created_at: '2026-03-26T02:00:00Z',
          updated_at: '2026-03-26T02:00:00Z',
        }}
      />,
    )

    expect(await screen.findByText(/raw text evidence/i)).toBeInTheDocument()
    expect(await screen.findByText('WeChat Bridge')).toBeInTheDocument()
    expect(screen.getByText('Synced')).toBeInTheDocument()
  })

  it('shows ai decision result, failure reason, retry summary, and clarification prompt', async () => {
    render(
      <RawLogDetail
        open
        onClose={() => {}}
        log={{
          id: 'log-2',
          user_id: 'user-1',
          raw_text: '今天起床了',
          input_channel: 'web',
          source_type: 'manual',
          context_date: '2026-03-26',
          timezone: 'Asia/Shanghai',
          parse_status: 'needs_review',
          parser_version: 'm3-parser',
          parse_error: 'missing exact wake time',
          ai_result: {
            status: 'rejected',
            summary: 'message needs clarification',
            action_preview: 'hold mutation until user reply',
            failure_reason: 'missing exact wake time',
            clarification_question: '你是 9:40 起床，还是 10:40 起床？',
            retry_summary: 'retry 2 stopped before execution',
          },
          created_at: '2026-03-26T02:00:00Z',
          updated_at: '2026-03-26T02:05:00Z',
        }}
      />,
    )

    expect(await screen.findByText(/ai decision result/i)).toBeInTheDocument()
    expect(screen.getByText(/message needs clarification/i)).toBeInTheDocument()
    expect(screen.getByText(/hold mutation until user reply/i)).toBeInTheDocument()
    expect(screen.getByText(/missing exact wake time/i)).toBeInTheDocument()
    expect(screen.getByText(/retry 2 stopped before execution/i)).toBeInTheDocument()
    expect(screen.getByText(/你是 9:40 起床，还是 10:40 起床？/i)).toBeInTheDocument()
  })
})
