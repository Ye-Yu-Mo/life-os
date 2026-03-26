import { Col, Row } from 'antd'
import LogInputForm from '../components/LogInputForm'

export default function InputPage() {
  return (
    <Row justify="center">
      <Col xs={24} lg={18} xl={16}>
        <LogInputForm />
      </Col>
    </Row>
  )
}
