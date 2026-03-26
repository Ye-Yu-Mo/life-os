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
          created_at: '2026-03-26T02:00:00Z',
          updated_at: '2026-03-26T02:00:00Z',
        }}
      />,
    )

    expect(await screen.findByText('WeChat Bridge')).toBeInTheDocument()
    expect(screen.getByText('Synced')).toBeInTheDocument()
  })
})
