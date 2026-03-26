# Changelog

变更记录

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)

版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)

---

## 版本号说明

- **主版本号（Major）**：不兼容的 API 变更或架构重构
- **次版本号（Minor）**：向后兼容的功能新增（新模块、新页面、新接口）
- **修订号（Patch）**：向后兼容的问题修正、小优化、文档更新

---

## [0.2.0] - 2026-03-26

### Added
- 增加批量导入接口 `POST /logs/import`。
- 增加 JSON / CSV 两种 raw logs 导入格式。
- 增加导入结果反馈：
  - `total_count`
  - `success_count`
  - `failure_count`
  - `errors`
- 增加前端文件上传导入入口，支持 `.json` / `.csv` 文件。
- 增加输入渠道扩展：
  - `telegram`
  - `feishu`
  - `wechat_bridge`
- 增加 Raw Logs 列表和详情页的来源展示。
- 增加 Telegram connector 基础骨架与配置入口。
- 增加 Feishu connector 预留模块与配置结构。
- 增加 WeChat Bridge 预留模块与配置结构。
- 增加统一 connector 抽象：
  - `ConnectorKind`
  - `ConnectorRuntimeMode`

### Changed
- 所有输入渠道继续统一落到 `raw_logs` 主链路。
- connector 输入现在有显式 service 入口 `create_connector_input(...)`。
- 单条写入、connector 写入、批量导入现在共享统一输入校验逻辑。
- 批量导入失败语义收紧为“未持久化任何记录”。
- `README.md`、`MEMORY.md`、`backend/.env.example` 已更新到当前实现状态。

### Fixed
- 修复 CLI 默认地址与 README / 测试不一致的问题。
- 修复批量导入失败时错误语义不清晰的问题。
- 修复 Raw Logs 页面直接暴露内部渠道字符串的问题。

### Security
- 增加 Telegram allowlist chat 配置边界。
- 增加 WeChat Bridge 共享密钥预留配置位。

### Tests
- 后端 `cargo test` 全量通过。
- 前端 `pnpm test` 全量通过。

## [0.1.0] - 2026-03-26

### Added
- 初始化 Rust 后端基础工程：
  - `axum`
  - `tokio`
  - `sqlx`
  - 配置加载
  - 统一错误处理
  - `/health` 健康检查
- 增加 PostgreSQL `raw_logs` 事实源表迁移。
- 增加 `raw_logs` 领域模型、repository、service 分层。
- 增加原始日志 API：
  - `POST /logs`
  - `GET /logs`
  - `GET /logs/{id}`
- 增加输入校验：
  - 空文本校验
  - 超长文本校验
  - `context_date` 格式校验
- 初始化 Web 前端基础工程：
  - Vite
  - React
  - Ant Design
  - React Router
- 增加 Quick Input 页面。
- 增加 Raw Logs 页面。
- 增加前端 API 封装：
  - 创建 raw log
  - 获取 raw log 列表
  - 获取 raw log 详情
- 增加本地运行骨架：
  - `docker-compose.yml`
  - `backend/.env.example`
  - `frontend/.env.example`
  - 根目录 `README.md`
- 增加前端测试基础设施：
  - Vitest
  - Testing Library
  - jsdom
  - 测试环境 setup

### Changed
- 前端默认 API 基地址改为 `/api`，通过 Vite 代理转发到后端。
- 后端启动流程改为自动加载 `.env`、连接数据库、执行 migrations 后再启动 HTTP 服务。
- 主后端路由挂载 `/logs`，不再只有健康检查。
- 前端主导航包含 `Quick Input` 与 `Raw Logs`。

### Fixed
- 修复前端 `invalid hook call` 问题。
- 修复前端 `.env` 配置未真正接入 Vite 代理的问题。
- 修复后端 `context_date` 直接写入 `DATE` 列的问题。
- 修复测试环境中 `antd` 依赖浏览器 API 缺失的问题。
- 修复测试配置污染生产构建的问题。

### Tests
- 后端：
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- 前端：
  - `pnpm test`
  - `npm run build`
