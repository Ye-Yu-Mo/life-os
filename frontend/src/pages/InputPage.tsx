import { Col, Row, Space, Tag, Typography } from 'antd'
import LogImportForm from '../components/LogImportForm'
import LogInputForm from '../components/LogInputForm'

export default function InputPage() {
  return (
    <div className="page-shell">
      <section className="page-hero">
        <Space direction="vertical" size={14} className="page-hero-copy">
          <Tag className="page-kicker">Input Milestone M1</Tag>
          <Typography.Title level={1} className="page-hero-title">
            Capture First. Structure Later.
          </Typography.Title>
          <Typography.Paragraph className="page-hero-text">
            先把事实写进系统。不要在输入阶段发明复杂规则。这个界面只负责把原始表达稳稳送进
            raw logs。
          </Typography.Paragraph>
        </Space>
      </section>

      <Row justify="center">
        <Col xs={24} lg={22} xl={20}>
          <Row gutter={[24, 24]} align="stretch">
            <Col xs={24} xl={12}>
              <LogInputForm />
            </Col>
            <Col xs={24} xl={12}>
              <LogImportForm />
            </Col>
          </Row>
        </Col>
      </Row>
    </div>
  )
}
