import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import LogInputForm from './LogInputForm'
import * as logsApi from '../api/logs'

vi.mock('../api/logs', async () => {
  const actual = await vi.importActual<typeof import('../api/logs')>('../api/logs')

  return {
    ...actual,
    createRawLog: vi.fn(),
  }
})

const mockedCreateRawLog = vi.mocked(logsApi.createRawLog)

describe('LogInputForm', () => {
  beforeEach(() => {
    mockedCreateRawLog.mockReset()
  })

  it('shows validation feedback when raw text is empty', async () => {
    const user = userEvent.setup()

    render(<LogInputForm />)

    await user.click(screen.getByRole('button', { name: /submit raw log/i }))

    expect(await screen.findByText(/raw text is required/i)).toBeInTheDocument()
    expect(mockedCreateRawLog).not.toHaveBeenCalled()
  })

  it('submits raw log and shows latest created result', async () => {
    const user = userEvent.setup()
    mockedCreateRawLog.mockResolvedValue({
      id: '550e8400-e29b-41d4-a716-446655440000',
      user_id: '550e8400-e29b-41d4-a716-446655440001',
      raw_text: '今天 9:40 起床',
      input_channel: 'web',
      source_type: 'manual',
      context_date: '2026-03-26',
      timezone: 'Asia/Shanghai',
      parse_status: 'pending',
      parser_version: null,
      parse_error: null,
      created_at: '2026-03-26T02:00:00Z',
      updated_at: '2026-03-26T02:00:00Z',
    })

    render(<LogInputForm />)

    await user.type(
      screen.getByPlaceholderText('今天 9:40 起床，凌晨 2:10 睡，睡得一般'),
      '今天 9:40 起床',
    )
    await user.type(screen.getByPlaceholderText('2026-03-26'), '2026-03-26')
    await user.click(screen.getByRole('button', { name: /submit raw log/i }))

    await waitFor(() => {
      expect(mockedCreateRawLog).toHaveBeenCalledWith(
        expect.objectContaining({
          raw_text: '今天 9:40 起床',
          input_channel: 'web',
          source_type: 'manual',
        }),
      )
    })

    expect(await screen.findByText(/latest created raw log/i)).toBeInTheDocument()
    expect(screen.getByText('550e8400-e29b-41d4-a716-446655440000')).toBeInTheDocument()
  })

  it('shows backend error message when submit fails', async () => {
    const user = userEvent.setup()
    mockedCreateRawLog.mockRejectedValue({
      isAxiosError: true,
      message: 'Request failed',
      response: {
        data: {
          error: {
            message: 'raw_text cannot be empty',
          },
        },
      },
    })

    render(<LogInputForm />)

    await user.type(
      screen.getByPlaceholderText('今天 9:40 起床，凌晨 2:10 睡，睡得一般'),
      '今天 9:40 起床',
    )
    await user.click(screen.getByRole('button', { name: /submit raw log/i }))

    expect(await screen.findByText(/submit failed/i)).toBeInTheDocument()
    expect(screen.getByText(/raw_text cannot be empty/i)).toBeInTheDocument()
  })
})
