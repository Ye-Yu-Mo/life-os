import { Alert, Button, Empty, Flex, Space, Spin, Typography } from 'antd'
import { useEffect, useState } from 'react'
import {
  fetchRawLogById,
  fetchRawLogs,
  getApiErrorMessage,
  type RawLog,
} from '../api/logs'
import RawLogDetail from '../components/RawLogDetail'
import RawLogsTable from '../components/RawLogsTable'

export default function RawLogsPage() {
  const [logs, setLogs] = useState<RawLog[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [pageError, setPageError] = useState<string | null>(null)
  const [selectedLog, setSelectedLog] = useState<RawLog | null>(null)
  const [detailOpen, setDetailOpen] = useState(false)
  const [isDetailLoading, setIsDetailLoading] = useState(false)

  async function loadLogs() {
    setIsLoading(true)
    setPageError(null)

    try {
      const nextLogs = await fetchRawLogs()
      setLogs(nextLogs)
    } catch (error) {
      setPageError(getApiErrorMessage(error))
    } finally {
      setIsLoading(false)
    }
  }

  async function handleViewDetail(id: string) {
    setDetailOpen(true)
    setIsDetailLoading(true)

    try {
      const detail = await fetchRawLogById(id)
      setSelectedLog(detail)
    } catch (error) {
      setSelectedLog(null)
      setPageError(getApiErrorMessage(error))
    } finally {
      setIsDetailLoading(false)
    }
  }

  useEffect(() => {
    void loadLogs()
  }, [])

  return (
    <Space direction="vertical" size={20} style={{ display: 'flex' }}>
      <Flex justify="space-between" align="center" gap={16} wrap="wrap">
        <div>
          <Typography.Title level={2} className="log-input-title">
            Raw Logs
          </Typography.Title>
          <Typography.Paragraph className="log-input-subtitle">
            这里直接看事实源。先确认数据是否入库，再谈解析和复盘。
          </Typography.Paragraph>
        </div>

        <Button onClick={() => void loadLogs()}>Refresh</Button>
      </Flex>

      {pageError ? (
        <Alert type="error" showIcon message="Load failed" description={pageError} />
      ) : null}

      {isLoading ? (
        <div className="raw-logs-loading">
          <Spin size="large" />
        </div>
      ) : logs.length === 0 ? (
        <section className="log-result-card">
          <Empty description="No raw logs yet" />
        </section>
      ) : (
        <section className="log-result-card">
          <RawLogsTable logs={logs} onViewDetail={handleViewDetail} />
        </section>
      )}

      <RawLogDetail
        log={selectedLog}
        open={detailOpen}
        loading={isDetailLoading}
        onClose={() => {
          setDetailOpen(false)
          setSelectedLog(null)
        }}
      />
    </Space>
  )
}
