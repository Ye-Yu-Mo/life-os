import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import LogImportForm from './LogImportForm'
import * as logsApi from '../api/logs'

vi.mock('../api/logs', async () => {
  const actual = await vi.importActual<typeof import('../api/logs')>('../api/logs')

  return {
    ...actual,
    importRawLogs: vi.fn(),
  }
})

const mockedImportRawLogs = vi.mocked(logsApi.importRawLogs)

describe('LogImportForm', () => {
  beforeEach(() => {
    mockedImportRawLogs.mockReset()
  })

  it('uploads a json file and shows import summary', async () => {
    const user = userEvent.setup()
    mockedImportRawLogs.mockResolvedValue({
      total_count: 2,
      success_count: 2,
      failure_count: 0,
      errors: [],
    })

    render(<LogImportForm />)

    const fileInput = screen.getByLabelText(/import file/i)
    const file = new File(
      [
        JSON.stringify({
          format: 'json',
          records: [
            {
              user_id: '550e8400-e29b-41d4-a716-446655440001',
              raw_text: '今天 9:40 起床',
              input_channel: 'import',
              source_type: 'imported',
            },
          ],
        }),
      ],
      'raw-logs.json',
      { type: 'application/json' },
    )

    await user.upload(fileInput, file)
    await user.click(screen.getByRole('button', { name: /import raw logs/i }))

    await waitFor(() => {
      expect(mockedImportRawLogs).toHaveBeenCalledWith(
        expect.objectContaining({
          format: 'json',
          records: expect.any(Array),
        }),
      )
    })

    expect(await screen.findByText(/import result/i)).toBeInTheDocument()
    expect(screen.getByText('Total')).toBeInTheDocument()
    expect(screen.getByText('Success')).toBeInTheDocument()
    expect(screen.getAllByText('2')).toHaveLength(2)
  })

  it('uploads a csv file as wrapped content payload', async () => {
    const user = userEvent.setup()
    mockedImportRawLogs.mockResolvedValue({
      total_count: 2,
      success_count: 2,
      failure_count: 0,
      errors: [],
    })

    render(<LogImportForm />)

    const fileInput = screen.getByLabelText(/import file/i)
    const file = new File(
      [
        'user_id,raw_text,input_channel,source_type\n550e8400-e29b-41d4-a716-446655440001,今天 9:40 起床,import,imported\n',
      ],
      'raw-logs.csv',
      { type: 'text/csv' },
    )

    await user.upload(fileInput, file)
    await user.click(screen.getByRole('button', { name: /import raw logs/i }))

    await waitFor(() => {
      expect(mockedImportRawLogs).toHaveBeenCalledWith({
        format: 'csv',
        content:
          'user_id,raw_text,input_channel,source_type\n550e8400-e29b-41d4-a716-446655440001,今天 9:40 起床,import,imported\n',
      })
    })
  })

  it('rejects unsupported file types before request', async () => {
    const user = userEvent.setup({ applyAccept: false })

    render(<LogImportForm />)

    const fileInput = screen.getByLabelText(/import file/i)
    const file = new File(['plain text'], 'raw-logs.txt', { type: 'text/plain' })

    await user.upload(fileInput, file)
    await user.click(screen.getByRole('button', { name: /import raw logs/i }))

    expect(await screen.findByText(/only json and csv files are supported/i)).toBeInTheDocument()
    expect(mockedImportRawLogs).not.toHaveBeenCalled()
  })

  it('rejects invalid json files before request', async () => {
    const user = userEvent.setup()

    render(<LogImportForm />)

    const fileInput = screen.getByLabelText(/import file/i)
    const file = new File(['{invalid json'], 'raw-logs.json', { type: 'application/json' })

    await user.upload(fileInput, file)
    await user.click(screen.getByRole('button', { name: /import raw logs/i }))

    expect(await screen.findByText(/invalid json import file/i)).toBeInTheDocument()
    expect(mockedImportRawLogs).not.toHaveBeenCalled()
  })

  it('shows backend error message when import fails', async () => {
    const user = userEvent.setup()
    mockedImportRawLogs.mockRejectedValue({
      isAxiosError: true,
      message: 'Request failed',
      response: {
        data: {
          error: {
            message: 'record 2: raw_text cannot be empty',
          },
        },
      },
    })

    render(<LogImportForm />)

    const fileInput = screen.getByLabelText(/import file/i)
    const file = new File(
      [
        JSON.stringify({
          format: 'json',
          records: [
            {
              user_id: '550e8400-e29b-41d4-a716-446655440001',
              raw_text: '今天 9:40 起床',
              input_channel: 'import',
              source_type: 'imported',
            },
          ],
        }),
      ],
      'raw-logs.json',
      { type: 'application/json' },
    )

    await user.upload(fileInput, file)
    await user.click(screen.getByRole('button', { name: /import raw logs/i }))

    expect(await screen.findByText(/import failed/i)).toBeInTheDocument()
    expect(screen.getByText(/record 2: raw_text cannot be empty/i)).toBeInTheDocument()
  })
})
