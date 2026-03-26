# life-os

`life-os` 当前开发基线是 `0.2.0`：

一个可以独立运行的渐进版本，当前已支持：

* Web 输入自然语言
* CLI 输入自然语言
* 后端保存 `raw_logs`
* 查看 Raw Logs 列表和详情

## 目录结构

* `backend/`: Rust + Axum + SQLx
* `frontend/`: Vite + React + Ant Design
* `docker-compose.yml`: 本地 PostgreSQL

## 环境要求

* Rust 1.89+
* Node.js 20+
* pnpm 10+
* Docker / Docker Compose

## 1. 初始化数据库

在项目根目录执行：

```bash
docker compose up -d
```

这会启动一个本地 PostgreSQL：

* Host: `127.0.0.1`
* Port: `5432`
* Database: `life_os`
* User: `postgres`
* Password: `postgres`

## 2. 配置后端环境变量

复制环境文件：

```bash
cp backend/.env.example backend/.env
```

默认配置已经指向上面的本地 PostgreSQL。

## 3. 启动后端

在 `backend/` 目录执行：

```bash
cargo run
```

后端启动时会：

* 自动读取 `backend/.env`
* 连接 PostgreSQL
* 自动执行 `backend/migrations/`
* 监听 `http://127.0.0.1:3000`

健康检查：

```bash
curl http://127.0.0.1:3000/health
```

## 4. 配置前端环境变量

复制环境文件：

```bash
cp frontend/.env.example frontend/.env
```

默认配置：

* `VITE_API_BASE_URL=/api`
* `VITE_DEV_PROXY_TARGET=http://127.0.0.1:3000`

这意味着前端开发时通过 Vite 代理访问后端，不需要手工处理跨域和地址拼接。

## 5. 启动前端

在 `frontend/` 目录执行：

```bash
pnpm install
pnpm dev
```

然后访问终端提示的地址，默认是：

```text
http://127.0.0.1:5173
```

## 6. 本地联调说明

当前联调链路是：

`浏览器 -> /api/* -> Vite proxy -> backend:3000`

前端代码默认访问：

* `POST /api/logs`
* `GET /api/logs`
* `GET /api/logs/{id}`

Vite 会把这些请求转发到后端：

* `POST /logs`
* `GET /logs`
* `GET /logs/{id}`

## 7. 测试与构建

后端：

```bash
cd backend
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

前端：

```bash
cd frontend
pnpm test
npm run build
```

## 8. CLI 输入

在 `backend/` 目录执行：

```bash
cargo run --bin logs_cli -- --user-id 550e8400-e29b-41d4-a716-446655440001 "今天 9:40 起床"
```

可选参数：

* `--context-date 2026-03-26`
* `--timezone Asia/Shanghai`

CLI 默认把请求提交到：

```text
http://127.0.0.1:3000/logs
```

如果后端不是这个地址，设置：

```bash
export LIFE_OS_API_BASE_URL=http://127.0.0.1:3000
```

## 9. 常见问题

### `invalid hook call`

不要混用 `npm` 和 `pnpm`。

前端当前使用 `pnpm` 管理依赖。混用会把 React 依赖树装脏，直接导致运行时 hook 错误。

### 后端启动失败

先确认：

* `docker compose up -d` 已执行
* `backend/.env` 存在
* `DATABASE_URL` 指向本地 PostgreSQL

### 前端请求失败

先确认：

* 后端已在 `127.0.0.1:3000` 启动
* `frontend/.env` 中 `VITE_DEV_PROXY_TARGET` 正确
* 浏览器访问的是 Vite dev server，而不是直接打开静态文件
