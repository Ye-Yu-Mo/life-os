import { Button, List, Space, Tag, Typography } from 'antd'
import type { RawLog } from '../api/logs'
import { formatInputChannelLabel, formatSourceTypeLabel } from './rawLogLabels'

type RawLogsTableProps = {
  logs: RawLog[]
  onViewDetail: (id: string) => void
}

export default function RawLogsTable({ logs, onViewDetail }: RawLogsTableProps) {
  return (
    <List
      itemLayout="vertical"
      dataSource={logs}
      renderItem={(log) => (
        <List.Item
          className="raw-log-list-item"
          actions={[
            <Button key={`view-${log.id}`} type="link" onClick={() => onViewDetail(log.id)}>
              Open Dossier
            </Button>,
          ]}
        >
          <Space direction="vertical" size={8} style={{ display: 'flex' }}>
            <Typography.Text className="raw-log-list-eyebrow">
              Fact Stream Entry
            </Typography.Text>
            <Space wrap>
              <Tag color="blue">{log.parse_status}</Tag>
              <Tag>{formatInputChannelLabel(log.input_channel)}</Tag>
              <Tag>{formatSourceTypeLabel(log.source_type)}</Tag>
            </Space>
            <Typography.Text strong className="raw-log-list-text">
              {log.raw_text}
            </Typography.Text>
            <Typography.Text type="secondary">{log.created_at}</Typography.Text>
          </Space>
        </List.Item>
      )}
    />
  )
}
