import { Alert, Button, Empty, Flex, Space, Spin, Tag, Typography } from 'antd'
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
    <Space direction="vertical" size={24} style={{ display: 'flex' }}>
      <Flex justify="space-between" align="center" gap={16} wrap="wrap">
        <div>
          <Typography.Title level={2} className="page-section-title">
            Raw Logs Archive
          </Typography.Title>
          <Typography.Paragraph className="page-section-subtitle">
            这里看的是事实流，不是最终事件。先确认记录进来了，再谈解析、聚合和复盘。
          </Typography.Paragraph>
        </div>

        <Tag className="page-count-tag">{logs.length} records</Tag>
      </Flex>

      {pageError ? (
        <Alert type="error" showIcon message="Load failed" description={pageError} />
      ) : null}

      <div className="toolbar-row">
        <Button onClick={() => void loadLogs()}>Reload Stream</Button>
      </div>

      {isLoading ? (
        <div className="panel-shell raw-logs-loading">
          <Spin size="large" />
        </div>
      ) : logs.length === 0 ? (
        <section className="panel-shell">
          <Empty description="No raw logs yet" />
          <Typography.Paragraph className="empty-state-note">
            Nothing has entered the fact stream yet.
          </Typography.Paragraph>
        </section>
      ) : (
        <section className="panel-shell">
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
