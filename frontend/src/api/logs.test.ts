import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockedPost = vi.fn()
const mockedGet = vi.fn()

vi.mock('axios', () => ({
  default: {
    create: () => ({
      post: mockedPost,
      get: mockedGet,
    }),
    isAxiosError: (error: unknown) => {
      return typeof error === 'object' && error !== null && 'isAxiosError' in error
    },
  },
}))

describe('logs api', () => {
  beforeEach(() => {
    mockedPost.mockReset()
    mockedGet.mockReset()
  })

  it('posts import payload to /logs/import', async () => {
    mockedPost.mockResolvedValue({
      data: {
        total_count: 2,
        success_count: 2,
        failure_count: 0,
        errors: [],
      },
    })

    const logsApi = await import('./logs')
    const importRawLogs = (logsApi as typeof logsApi & {
      importRawLogs?: (payload: unknown) => Promise<unknown>
    }).importRawLogs

    expect(typeof importRawLogs).toBe('function')

    await importRawLogs?.({
      format: 'json',
      records: [
        {
          user_id: '550e8400-e29b-41d4-a716-446655440001',
          raw_text: '今天 9:40 起床',
          input_channel: 'import',
          source_type: 'imported',
          context_date: '2026-03-26',
          timezone: 'Asia/Shanghai',
        },
        {
          user_id: '550e8400-e29b-41d4-a716-446655440001',
          raw_text: '晚上跑步 35 分钟',
          input_channel: 'import',
          source_type: 'imported',
          context_date: '2026-03-26',
          timezone: 'Asia/Shanghai',
        },
      ],
    })

    expect(mockedPost).toHaveBeenCalledWith(
      '/logs/import',
      expect.objectContaining({
        format: 'json',
      }),
    )
  })

  it('reads parse status and parse error from raw log detail response', async () => {
    mockedGet.mockResolvedValue({
      data: {
        id: 'log-1',
        user_id: 'user-1',
        raw_text: '今天 9:40 起床',
        input_channel: 'web',
        source_type: 'manual',
        context_date: '2026-03-26',
        timezone: 'Asia/Shanghai',
        parse_status: 'needs_review',
        parser_version: 'm3-test',
        parse_error: 'missing wake time',
        created_at: '2026-03-26T02:00:00Z',
        updated_at: '2026-03-26T02:05:00Z',
      },
    })

    const logsApi = await import('./logs')
    const detail = await logsApi.fetchRawLogById('log-1')

    expect(mockedGet).toHaveBeenCalledWith('/logs/log-1')
    expect(detail.parse_status).toBe('needs_review')
    expect(detail.parse_error).toBe('missing wake time')
  })

  it('reads ai decision snapshot from raw log detail response', async () => {
    mockedGet.mockResolvedValue({
      data: {
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
        created_at: '2026-03-26T02:00:00Z',
        updated_at: '2026-03-26T02:05:00Z',
        ai_result: {
          status: 'rejected',
          summary: 'message needs clarification',
          action_preview: 'hold mutation until user reply',
          failure_reason: 'missing exact wake time',
          clarification_question: '你是 9:40 起床，还是 10:40 起床？',
          retry_summary: 'retry 2 stopped before execution',
        },
      },
    })

    const logsApi = await import('./logs')
    const detail = await logsApi.fetchRawLogById('log-2')

    expect(detail.ai_result?.status).toBe('rejected')
    expect(detail.ai_result?.summary).toBe('message needs clarification')
    expect(detail.ai_result?.retry_summary).toBe('retry 2 stopped before execution')
  })
})
