import { Descriptions, Drawer, Tag, Typography } from 'antd'
import type { RawLog } from '../api/logs'
import { formatInputChannelLabel, formatSourceTypeLabel } from './rawLogLabels'

type RawLogDetailProps = {
  log: RawLog | null
  open: boolean
  loading?: boolean
  onClose: () => void
}

export default function RawLogDetail({
  log,
  open,
  loading = false,
  onClose,
}: RawLogDetailProps) {
  return (
    <Drawer
      title="Latest Raw Log Detail"
      placement="right"
      width={440}
      open={open}
      onClose={onClose}
      destroyOnClose
      loading={loading}
    >
      {log ? (
        <Descriptions column={1} bordered size="small">
          <Descriptions.Item label="ID">{log.id}</Descriptions.Item>
          <Descriptions.Item label="User ID">{log.user_id}</Descriptions.Item>
          <Descriptions.Item label="Status">
            <Tag color="blue">{log.parse_status}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="Input Channel">
            {formatInputChannelLabel(log.input_channel)}
          </Descriptions.Item>
          <Descriptions.Item label="Source Type">
            {formatSourceTypeLabel(log.source_type)}
          </Descriptions.Item>
          <Descriptions.Item label="Context Date">
            {log.context_date ?? '-'}
          </Descriptions.Item>
          <Descriptions.Item label="Timezone">{log.timezone ?? '-'}</Descriptions.Item>
          <Descriptions.Item label="Created At">{log.created_at}</Descriptions.Item>
          <Descriptions.Item label="Updated At">{log.updated_at}</Descriptions.Item>
          <Descriptions.Item label="Raw Text">
            <Typography.Paragraph className="raw-log-detail-text">
              {log.raw_text}
            </Typography.Paragraph>
          </Descriptions.Item>
        </Descriptions>
      ) : null}
    </Drawer>
  )
}
