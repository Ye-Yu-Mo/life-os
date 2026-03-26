import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import RawLogsTable from './RawLogsTable'

describe('RawLogsTable', () => {
  it('shows human-readable source labels in the list', async () => {
    const user = userEvent.setup()
    const onViewDetail = vi.fn()

    render(
      <RawLogsTable
        logs={[
          {
            id: 'log-1',
            user_id: 'user-1',
            raw_text: '收到一条 Telegram 消息',
            input_channel: 'telegram',
            source_type: 'imported',
            context_date: '2026-03-26',
            timezone: 'Asia/Shanghai',
            parse_status: 'pending',
            parser_version: null,
            parse_error: null,
            created_at: '2026-03-26T02:00:00Z',
            updated_at: '2026-03-26T02:00:00Z',
          },
        ]}
        onViewDetail={onViewDetail}
      />,
    )

    expect(screen.getByText('Telegram')).toBeInTheDocument()
    expect(screen.getByText('Imported')).toBeInTheDocument()
    expect(screen.getByText(/fact stream entry/i)).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: /open dossier/i }))
    expect(onViewDetail).toHaveBeenCalledWith('log-1')
  })
})
