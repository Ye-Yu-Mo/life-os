import axios from 'axios'

const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL ?? '/api',
  timeout: 10000,
})

export type SubmitAiMessagePayload = {
  user_id: string
  message_text: string
  context_date?: string
  timezone?: string
}

export type AiRunResult = {
  run_id: string
  raw_log_id: string
  status: string
  parse_status: string
  summary: string
  action_preview: string
  failure_reason?: string | null
  clarification_question?: string | null
}

export async function submitAiMessage(payload: SubmitAiMessagePayload) {
  const response = await apiClient.post<AiRunResult>('/ai/messages', payload)
  return response.data
}
