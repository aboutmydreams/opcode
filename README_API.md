# Opcode API Server - 部署指南

这是 Opcode 的纯 API 版本，移除了所有 GUI 相关功能，仅保留 HTTP API 服务。

## 编译项目

```bash
# 1. 确保已安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. 进入项目目录并编译
npm run build
# 或直接运行：cd src-tauri && cargo build --release
```

## 启动API服务器

```bash
# 默认端口 3001
npm run dev
# 或直接运行：./src-tauri/target/release/opcode api

# 自定义端口
./src-tauri/target/release/opcode api --port 8080
```

## 验证API

```bash
# 健康检查
curl http://localhost:3001/api/health

# 查看API文档
open http://localhost:3001/docs
```

## 主要端点

- `GET /api/health` - 健康检查
- `GET /api/agents` - 列出agents
- `POST /api/agents` - 创建agent
- `GET /api/projects` - 列出项目

## 注意事项

⚠️ **当前限制**：
- Agent执行功能暂不支持HTTP API
- 仅支持数据查询和基本管理操作
- 生产环境请配置HTTPS

✅ **已支持**：
- Agent CRUD操作
- 项目和会话查看
- 完整的OpenAPI文档
- CORS跨域支持

查看完整文档：`HTTP_API_GUIDE.md`