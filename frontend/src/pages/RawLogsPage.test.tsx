import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import RawLogsPage from './RawLogsPage'
import * as logsApi from '../api/logs'

vi.mock('../api/logs', async () => {
  const actual = await vi.importActual<typeof import('../api/logs')>('../api/logs')

  return {
    ...actual,
    fetchRawLogs: vi.fn(),
    fetchRawLogById: vi.fn(),
  }
})

const mockedFetchRawLogs = vi.mocked(logsApi.fetchRawLogs)
const mockedFetchRawLogById = vi.mocked(logsApi.fetchRawLogById)

describe('RawLogsPage', () => {
  beforeEach(() => {
    mockedFetchRawLogs.mockReset()
    mockedFetchRawLogById.mockReset()
  })

  it('shows empty state when there are no raw logs', async () => {
    mockedFetchRawLogs.mockResolvedValue([])

    render(<RawLogsPage />)

    expect(await screen.findByText(/no raw logs yet/i)).toBeInTheDocument()
    expect(screen.getByText(/nothing has entered the fact stream yet/i)).toBeInTheDocument()
  })

  it('renders raw logs list and refreshes data', async () => {
    const user = userEvent.setup()
    mockedFetchRawLogs
      .mockResolvedValueOnce([
        {
          id: 'log-1',
          user_id: 'user-1',
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
        },
      ])
      .mockResolvedValueOnce([
        {
          id: 'log-2',
          user_id: 'user-1',
          raw_text: '晚上跑步 35 分钟',
          input_channel: 'web',
          source_type: 'manual',
          context_date: '2026-03-26',
          timezone: 'Asia/Shanghai',
          parse_status: 'pending',
          parser_version: null,
          parse_error: null,
          created_at: '2026-03-26T03:00:00Z',
          updated_at: '2026-03-26T03:00:00Z',
        },
      ])

    render(<RawLogsPage />)

    expect(await screen.findByText('今天 9:40 起床')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: /reload stream/i }))

    await waitFor(() => {
      expect(mockedFetchRawLogs).toHaveBeenCalledTimes(2)
    })

    expect(await screen.findByText('晚上跑步 35 分钟')).toBeInTheDocument()
  })

  it('opens detail view when a row is selected', async () => {
    const user = userEvent.setup()
    mockedFetchRawLogs.mockResolvedValue([
      {
        id: 'log-1',
        user_id: 'user-1',
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
      },
    ])
    mockedFetchRawLogById.mockResolvedValue({
      id: 'log-1',
      user_id: 'user-1',
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

    render(<RawLogsPage />)

    await user.click(await screen.findByRole('button', { name: /open dossier/i }))

    await waitFor(() => {
      expect(mockedFetchRawLogById).toHaveBeenCalledWith('log-1')
    })

    expect(await screen.findByText(/raw log dossier/i)).toBeInTheDocument()
    expect(screen.getAllByText('今天 9:40 起床').length).toBeGreaterThan(0)
  })
})
