# OpCode API Server

HTTP API服务器，用于替代Tauri桌面应用的后端功能。

## 🚀 快速开始

### 1. 安装 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 2. 运行 API 服务器

```bash
cd api-server
chmod +x run.sh
./run.sh
```

或者手动编译运行：

```bash
cd api-server
cargo build --release
cargo run --release
```

### 3. 访问 API

- **API 服务器**: http://localhost:3000
- **API 文档 (Swagger UI)**: http://localhost:3000/docs
- **健康检查**: http://localhost:3000/health

## 📚 API 端点

### Agents (智能体管理)

- `GET /api/agents` - 获取所有智能体
- `POST /api/agents` - 创建新智能体
- `GET /api/agents/{id}` - 获取特定智能体
- `DELETE /api/agents/{id}` - 删除智能体

### Claude Sessions (Claude 会话管理)

- `GET /api/claude/projects` - 获取所有项目
- `POST /api/claude/sessions` - 启动新的 Claude 会话

### Storage (存储管理)

- `GET /api/storage/usage` - 获取存储使用统计

## 🛠️ 配置

### 环境变量

- `PORT` - 服务器端口 (默认: 3000)
- `RUST_LOG` - 日志级别 (默认: info)

### 依赖要求

- **Claude Code CLI**: 需要安装 Claude Code 命令行工具
- **SQLite**: 自动包含，无需额外安装

## 🏗️ 架构说明

这个API服务器从原始的Tauri桌面应用中提取了核心业务逻辑：

### 原始架构 → 新架构

- **Tauri IPC** → **HTTP REST API**
- **Tauri 状态管理** → **Arc<Service> 依赖注入**  
- **桌面应用** → **Web API + Swagger文档**

### 技术栈

- **Web框架**: Axum
- **文档生成**: utoipa + Swagger UI
- **数据库**: SQLite (rusqlite)
- **异步运行时**: Tokio
- **跨域支持**: tower-http CORS

## 🔧 开发

### 添加新的 API 端点

1. 在 `src/models/` 中定义数据模型
2. 在 `src/services/` 中实现业务逻辑
3. 在 `src/handlers/` 中创建HTTP处理器
4. 在 `src/main.rs` 中注册路由

### 数据库 Schema

数据库文件位置: `~/.claude/opcode.db`

主要表：
- `agents` - 智能体配置
- `agent_runs` - 智能体执行记录
- `mcp_servers` - MCP服务器配置
- `slash_commands` - 斜杠命令配置

## 🐛 故障排除

### Claude二进制文件未找到

确保 Claude Code CLI 已安装并在 PATH 中：

```bash
which claude
# 或者安装 Claude Code
npm install -g @anthropic/claude
```

### 数据库权限问题

确保 `~/.claude` 目录有写权限：

```bash
mkdir -p ~/.claude
chmod 755 ~/.claude
```

## 📄 许可证

AGPL-3.0 License