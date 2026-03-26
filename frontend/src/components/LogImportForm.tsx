import { Alert, Button, Space, Statistic, Typography } from 'antd'
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
    <section className="feature-card">
      <div className="feature-card-header">
        <div>
          <Typography.Title level={2} className="feature-card-title">
            Import Queue
          </Typography.Title>
          <Typography.Paragraph className="feature-card-subtitle">
            批量导入不是另一套系统。它只是把外部文件稳定压进同一条事实流。
          </Typography.Paragraph>
        </div>
      </div>

      <Space direction="vertical" size={16} style={{ display: 'flex' }}>
        <label htmlFor={FILE_INPUT_ID} className="log-import-label">
          Choose Import File
        </label>
        <div className="import-dropzone">
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
          <Typography.Text className="import-dropzone-title">
            {selectedFile ? selectedFile.name : 'Select JSON or CSV fact batches'}
          </Typography.Text>
          <Typography.Text type="secondary">
            Supports `.json` import payloads and raw `.csv` files.
          </Typography.Text>
        </div>

        <Button type="primary" onClick={() => void handleSubmit()} loading={isSubmitting} block>
          Run Import
        </Button>
      </Space>

      {submitError ? (
        <Alert type="error" showIcon message="Import Blocked" description={submitError} />
      ) : null}

      {importResult ? (
        <section className="result-panel">
          <Typography.Title level={4} className="result-panel-title">
            Import Report
          </Typography.Title>
          <div className="import-stats-grid">
            <div className="import-stat-card">
              <Statistic title="Total" value={importResult.total_count} />
            </div>
            <div className="import-stat-card import-stat-card-success">
              <Statistic title="Success" value={importResult.success_count} />
            </div>
            <div className="import-stat-card import-stat-card-danger">
              <Statistic title="Failure" value={importResult.failure_count} />
            </div>
          </div>
          <dl className="log-result-grid import-errors-grid">
            <dt>Errors</dt>
            <dd>{importResult.errors.length === 0 ? 'None' : importResult.errors.join('; ')}</dd>
          </dl>
        </section>
      ) : null}
    </section>
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
