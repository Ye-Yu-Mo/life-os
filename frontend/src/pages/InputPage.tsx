import { Col, Row } from 'antd'
import LogImportForm from '../components/LogImportForm'
import LogInputForm from '../components/LogInputForm'

export default function InputPage() {
  return (
    <Row justify="center">
      <Col xs={24} lg={18} xl={16}>
        <Row gutter={[0, 24]}>
          <Col span={24}>
            <LogInputForm />
          </Col>
          <Col span={24}>
            <LogImportForm />
          </Col>
        </Row>
      </Col>
    </Row>
  )
}
