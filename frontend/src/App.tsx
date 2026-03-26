import { ConfigProvider, Layout, Menu, Typography, type MenuProps } from 'antd'
import { Navigate, Route, Routes, useLocation, useNavigate } from 'react-router-dom'
import './App.css'
import AiConsolePage from './pages/AiConsolePage'
import InputPage from './pages/InputPage'
import RawLogsPage from './pages/RawLogsPage'

type MenuClickInfo = Parameters<NonNullable<MenuProps['onClick']>>[0]

function App() {
  const location = useLocation()
  const navigate = useNavigate()
  const handleMenuClick: MenuProps['onClick'] = ({ key }: MenuClickInfo) => {
    navigate(String(key))
  }

  return (
    <ConfigProvider
      theme={{
        token: {
          colorPrimary: '#1e5eff',
          borderRadius: 16,
          colorTextBase: '#16324f',
          colorTextHeading: '#10233a',
          colorBgContainer: '#ffffff',
          fontFamily:
            '"IBM Plex Sans", "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", sans-serif',
        },
      }}
    >
      <Layout className="app-shell">
        <Layout.Header className="app-header">
          <div className="app-brand">
            <div className="app-brand-mark" aria-hidden="true">
              LO
            </div>
            <div className="app-brand-copy">
              <Typography.Title level={4} className="app-brand-title">
                Life OS
              </Typography.Title>
              <Typography.Text className="app-brand-text">
                Capture First. Structure Later.
              </Typography.Text>
            </div>
          </div>

          <Menu
            mode="horizontal"
            selectedKeys={[location.pathname]}
            items={[
              { key: '/', label: 'Quick Capture' },
              { key: '/ai', label: 'AI Console' },
              { key: '/logs', label: 'Raw Logs Archive' },
            ]}
            onClick={handleMenuClick}
            className="app-menu"
          />
        </Layout.Header>

        <Layout.Content className="app-content">
          <Routes>
            <Route path="/" element={<InputPage />} />
            <Route path="/ai" element={<AiConsolePage />} />
            <Route path="/logs" element={<RawLogsPage />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </Layout.Content>
      </Layout>
    </ConfigProvider>
  )
}

export default App
