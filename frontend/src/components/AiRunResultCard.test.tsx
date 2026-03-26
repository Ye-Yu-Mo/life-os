import { render, screen } from '@testing-library/react'
import AiRunResultCard from './AiRunResultCard'

describe('AiRunResultCard', () => {
  it('shows summary and action preview for a completed run', () => {
    render(
      <AiRunResultCard
        result={{
          run_id: 'run-1',
          raw_log_id: 'log-1',
          status: 'completed',
          parse_status: 'parsed',
          summary: 'recorded wake event',
          action_preview: 'create sleep.wake event',
          failure_reason: null,
          clarification_question: null,
        }}
      />,
    )

    expect(screen.getByText(/processing summary/i)).toBeInTheDocument()
    expect(screen.getByText(/recorded wake event/i)).toBeInTheDocument()
    expect(screen.getByText(/action preview/i)).toBeInTheDocument()
    expect(screen.getByText(/create sleep\.wake event/i)).toBeInTheDocument()
  })

  it('shows failure reason and clarification question when run needs review', () => {
    render(
      <AiRunResultCard
        result={{
          run_id: 'run-2',
          raw_log_id: 'log-2',
          status: 'rejected',
          parse_status: 'needs_review',
          summary: 'message needs clarification',
          action_preview: 'hold mutation until user reply',
          failure_reason: 'missing exact wake time',
          clarification_question: '你是 9:40 起床，还是 10:40 起床？',
        }}
      />,
    )

    expect(screen.getByText(/failure reason/i)).toBeInTheDocument()
    expect(screen.getByText(/missing exact wake time/i)).toBeInTheDocument()
    expect(screen.getByText(/clarification prompt/i)).toBeInTheDocument()
    expect(screen.getByText(/你是 9:40 起床，还是 10:40 起床？/i)).toBeInTheDocument()
  })
})
