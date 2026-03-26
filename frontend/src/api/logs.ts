import axios from 'axios'

const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL ?? 'http://127.0.0.1:3000',
  timeout: 10000,
})

export type CreateRawLogPayload = {
  user_id: string
  raw_text: string
  input_channel: 'web'
  source_type: 'manual'
  context_date?: string
  timezone?: string
}

export type RawLog = {
  id: string
  user_id: string
  raw_text: string
  input_channel: string
  source_type: string
  context_date?: string | null
  timezone?: string | null
  parse_status: string
  parser_version?: string | null
  parse_error?: string | null
  created_at: string
  updated_at: string
}

type ApiErrorBody = {
  error?: {
    code?: string
    message?: string
  }
}

export async function createRawLog(payload: CreateRawLogPayload) {
  const response = await apiClient.post<RawLog>('/logs', payload)
  return response.data
}

export function getApiErrorMessage(error: unknown) {
  if (axios.isAxiosError<ApiErrorBody>(error)) {
    return error.response?.data?.error?.message ?? error.message
  }

  if (error instanceof Error) {
    return error.message
  }

  return 'Unknown request error'
}
