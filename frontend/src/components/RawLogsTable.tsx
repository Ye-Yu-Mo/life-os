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
          actions={[
            <Button key={`view-${log.id}`} type="link" onClick={() => onViewDetail(log.id)}>
              View Detail
            </Button>,
          ]}
        >
          <Space direction="vertical" size={8} style={{ display: 'flex' }}>
            <Space wrap>
              <Tag color="blue">{log.parse_status}</Tag>
              <Tag>{formatInputChannelLabel(log.input_channel)}</Tag>
              <Tag>{formatSourceTypeLabel(log.source_type)}</Tag>
            </Space>
            <Typography.Text strong>{log.raw_text}</Typography.Text>
            <Typography.Text type="secondary">{log.created_at}</Typography.Text>
          </Space>
        </List.Item>
      )}
    />
  )
}
