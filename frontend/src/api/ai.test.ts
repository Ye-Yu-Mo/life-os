import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockedPost = vi.fn()

vi.mock('axios', () => ({
  default: {
    create: () => ({
      post: mockedPost,
    }),
    isAxiosError: (error: unknown) => {
      return typeof error === 'object' && error !== null && 'isAxiosError' in error
    },
  },
}))

describe('ai api', () => {
  beforeEach(() => {
    mockedPost.mockReset()
  })

  it('posts ai message payload and returns processing result', async () => {
    mockedPost.mockResolvedValue({
      data: {
        run_id: 'run-1',
        raw_log_id: 'log-1',
        status: 'completed',
        parse_status: 'parsed',
        summary: 'recorded wake event',
        action_preview: 'create sleep.wake event',
        failure_reason: null,
        clarification_question: null,
      },
    })

    const aiApi = await import('./ai')
    const result = await aiApi.submitAiMessage({
      user_id: '550e8400-e29b-41d4-a716-446655440001',
      message_text: '今天 9:40 起床',
      context_date: '2026-03-26',
      timezone: 'Asia/Shanghai',
    })

    expect(mockedPost).toHaveBeenCalledWith(
      '/ai/messages',
      expect.objectContaining({
        user_id: '550e8400-e29b-41d4-a716-446655440001',
        message_text: '今天 9:40 起床',
      }),
    )
    expect(result.parse_status).toBe('parsed')
    expect(result.action_preview).toBe('create sleep.wake event')
  })
})
