import { Layout, Menu, Typography, type MenuProps } from 'antd'
import { Navigate, Route, Routes, useLocation, useNavigate } from 'react-router-dom'
import './App.css'
import InputPage from './pages/InputPage'

type MenuClickInfo = Parameters<NonNullable<MenuProps['onClick']>>[0]

function App() {
  const location = useLocation()
  const navigate = useNavigate()
  const handleMenuClick: MenuProps['onClick'] = ({ key }: MenuClickInfo) => {
    navigate(String(key))
  }

  return (
    <Layout className="app-shell">
      <Layout.Header className="app-header">
        <div className="app-brand">
          <Typography.Title level={4} className="app-brand-title">
            Life OS
          </Typography.Title>
          <Typography.Text className="app-brand-text">
            0.1.0 Input And Raw Logs
          </Typography.Text>
        </div>

        <Menu
          theme="dark"
          mode="horizontal"
          selectedKeys={[location.pathname]}
          items={[{ key: '/', label: 'Quick Input' }]}
          onClick={handleMenuClick}
          className="app-menu"
        />
      </Layout.Header>

      <Layout.Content className="app-content">
        <Routes>
          <Route path="/" element={<InputPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Routes>
      </Layout.Content>
    </Layout>
  )
}

export default App
