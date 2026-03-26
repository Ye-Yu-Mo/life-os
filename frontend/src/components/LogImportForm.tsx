import { Alert, Button, Space, Typography } from 'antd'
import { useState } from 'react'
import {
  getApiErrorMessage,
  importRawLogs,
  type ImportRawLogsPayload,
  type ImportRawLogsResult,
} from '../api/logs'

const FILE_INPUT_ID = 'raw-log-import-file'

export default function LogImportForm() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [importResult, setImportResult] = useState<ImportRawLogsResult | null>(null)

  async function handleSubmit() {
    if (!selectedFile) {
      setSubmitError('Select a JSON or CSV file first')
      return
    }

    setIsSubmitting(true)
    setSubmitError(null)
    setImportResult(null)

    try {
      const payload = await buildImportPayload(selectedFile)
      const result = await importRawLogs(payload)
      setImportResult(result)
    } catch (error) {
      setSubmitError(getApiErrorMessage(error))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Space direction="vertical" size={20} className="log-input-stack">
      <section className="log-input-card">
        <Typography.Title level={2} className="log-input-title">
          Import Raw Logs
        </Typography.Title>
        <Typography.Paragraph className="log-input-subtitle">
          批量导入只做一件事：把外部文件稳定写进事实源，不顺手发明别的流程。
        </Typography.Paragraph>

        <Space direction="vertical" size={16} style={{ display: 'flex' }}>
          <label htmlFor={FILE_INPUT_ID} className="log-import-label">
            Import File
          </label>
          <input
            id={FILE_INPUT_ID}
            className="log-import-file-input"
            type="file"
            accept=".json,.csv"
            onChange={(event) => {
              setSubmitError(null)
              setImportResult(null)
              setSelectedFile(event.target.files?.[0] ?? null)
            }}
          />
          <Typography.Text type="secondary">
            Supports `.json` import payloads and raw `.csv` files.
          </Typography.Text>

          <Button type="primary" onClick={() => void handleSubmit()} loading={isSubmitting}>
            Import Raw Logs
          </Button>
        </Space>

        {submitError ? (
          <Alert type="error" showIcon message="Import failed" description={submitError} />
        ) : null}
      </section>

      {importResult ? (
        <section className="log-result-card">
          <Typography.Title level={4}>Import Result</Typography.Title>
          <dl className="log-result-grid">
            <dt>Total</dt>
            <dd>{importResult.total_count}</dd>
            <dt>Success</dt>
            <dd>{importResult.success_count}</dd>
            <dt>Failure</dt>
            <dd>{importResult.failure_count}</dd>
            <dt>Errors</dt>
            <dd>{importResult.errors.length === 0 ? 'None' : importResult.errors.join('; ')}</dd>
          </dl>
        </section>
      ) : null}
    </Space>
  )
}

async function buildImportPayload(file: File): Promise<ImportRawLogsPayload> {
  const fileName = file.name.toLowerCase()

  if (fileName.endsWith('.csv')) {
    const content = await file.text()

    if (content.trim().length === 0) {
      throw new Error('Import file cannot be empty')
    }

    return { format: 'csv', content }
  }

  if (fileName.endsWith('.json')) {
    const content = await file.text()

    if (content.trim().length === 0) {
      throw new Error('Import file cannot be empty')
    }

    try {
      return JSON.parse(content) as ImportRawLogsPayload
    } catch {
      throw new Error('Invalid JSON import file')
    }
  }

  throw new Error('Only JSON and CSV files are supported')
}
