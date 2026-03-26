import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import AiConsolePage from './AiConsolePage'
import * as aiApi from '../api/ai'

vi.mock('../api/ai', async () => {
  const actual = await vi.importActual<typeof import('../api/ai')>('../api/ai')

  return {
    ...actual,
    submitAiMessage: vi.fn(),
  }
})

const mockedSubmitAiMessage = vi.mocked(aiApi.submitAiMessage)

describe('AiConsolePage', () => {
  beforeEach(() => {
    mockedSubmitAiMessage.mockReset()
  })

  it('submits a message and shows ai processing result', async () => {
    const user = userEvent.setup()
    mockedSubmitAiMessage.mockResolvedValue({
      run_id: 'run-1',
      raw_log_id: 'log-1',
      status: 'completed',
      parse_status: 'parsed',
      summary: 'recorded wake event',
      action_preview: 'create sleep.wake event',
      failure_reason: null,
      clarification_question: null,
    })

    render(<AiConsolePage />)

    await user.clear(screen.getByLabelText(/user id/i))
    await user.type(screen.getByLabelText(/user id/i), '550e8400-e29b-41d4-a716-446655440001')
    await user.type(screen.getByLabelText(/message/i), '今天 9:40 起床')
    await user.click(screen.getByRole('button', { name: /run ai/i }))

    await waitFor(() => {
      expect(mockedSubmitAiMessage).toHaveBeenCalledWith(
        expect.objectContaining({
          user_id: '550e8400-e29b-41d4-a716-446655440001',
          message_text: '今天 9:40 起床',
        }),
      )
    })

    expect(await screen.findByText(/recorded wake event/i)).toBeInTheDocument()
    expect(screen.getByText(/create sleep\.wake event/i)).toBeInTheDocument()
  })

  it('shows clarification and failure feedback from ai result', async () => {
    const user = userEvent.setup()
    mockedSubmitAiMessage.mockResolvedValue({
      run_id: 'run-2',
      raw_log_id: 'log-2',
      status: 'rejected',
      parse_status: 'needs_review',
      summary: 'message needs clarification',
      action_preview: 'hold mutation until user reply',
      failure_reason: 'missing exact wake time',
      clarification_question: '你是 9:40 起床，还是 10:40 起床？',
    })

    render(<AiConsolePage />)

    await user.clear(screen.getByLabelText(/user id/i))
    await user.type(screen.getByLabelText(/user id/i), '550e8400-e29b-41d4-a716-446655440001')
    await user.type(screen.getByLabelText(/message/i), '今天起床了')
    await user.click(screen.getByRole('button', { name: /run ai/i }))

    expect(await screen.findByText(/message needs clarification/i)).toBeInTheDocument()
    expect(screen.getByText(/missing exact wake time/i)).toBeInTheDocument()
    expect(screen.getByText(/你是 9:40 起床，还是 10:40 起床？/i)).toBeInTheDocument()
  })
})
