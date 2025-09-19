# Opcode HTTP API 使用指南

## 概述

这个项目现在支持HTTP API模式，允许通过REST API访问Opcode的核心功能。这为Web应用、移动应用或其他第三方集成提供了支持。

## 架构说明

- **原有架构**: Tauri桌面应用 (前端: React + TypeScript, 后端: Rust)
- **新增功能**: HTTP API服务器 (基于Axum框架)
- **共享组件**: 数据库、业务逻辑、文件系统操作

## 启动HTTP API服务器

### 编译项目

```bash
cd src-tauri
cargo build --release
```

### 启动API服务器

```bash
# 使用默认端口 3001
./target/release/opcode api

# 或指定自定义端口
./target/release/opcode api --port 8080
```

### 服务器信息

启动后可以看到：
- **API基础URL**: `http://localhost:3001/api`
- **Swagger UI**: `http://localhost:3001/docs`
- **RapiDoc**: `http://localhost:3001/rapidoc` 
- **ReDoc**: `http://localhost:3001/redoc`
- **OpenAPI JSON**: `http://localhost:3001/api-docs/openapi.json`

## API端点概览

### 健康检查
- `GET /api/health` - 检查服务状态

### Agent管理
- `GET /api/agents` - 列出所有agents
- `GET /api/agents/{id}` - 获取特定agent详情
- `POST /api/agents` - 创建新agent
- `POST /api/agents/{id}/execute` - 执行agent (暂不支持)
- `GET /api/agents/{id}/runs` - 获取agent执行历史
- `GET /api/agents/runs` - 获取所有agent执行历史

### 项目管理
- `GET /api/projects` - 列出所有项目
- `GET /api/projects/{project_id}/sessions` - 获取项目的会话

### 会话管理
- `GET /api/sessions/{session_id}/history/{project_id}` - 获取会话历史

## API使用示例

### 1. 健康检查

```bash
curl http://localhost:3001/api/health
```

响应:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.2.1",
    "timestamp": "2025-09-19T10:30:00Z",
    "services": {
      "database": "healthy",
      "checkpoint_manager": "healthy", 
      "process_registry": "healthy"
    }
  },
  "timestamp": "2025-09-19T10:30:00Z"
}
```

### 2. 列出所有Agents

```bash
curl http://localhost:3001/api/agents
```

支持分页:
```bash
curl "http://localhost:3001/api/agents?page=1&limit=10"
```

### 3. 创建新Agent

```bash
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Web开发助手",
    "icon": "🌐",
    "system_prompt": "你是一个专业的Web开发助手...",
    "default_task": "帮助用户解决Web开发问题",
    "model": "sonnet",
    "enable_file_read": true,
    "enable_file_write": true,
    "enable_network": false
  }'
```

### 4. 获取项目列表

```bash
curl http://localhost:3001/api/projects
```

### 5. 获取项目会话

```bash
curl http://localhost:3001/api/projects/{project_id}/sessions
```

## 响应格式

所有API响应都遵循统一格式：

```json
{
  "success": boolean,
  "data": any,           // 成功时的数据
  "message": string,     // 可选的消息
  "timestamp": string    // ISO 8601格式时间戳
}
```

错误响应：
```json
{
  "error": true,
  "message": "错误描述",
  "status": 400
}
```

## CORS配置

API支持以下域的跨域请求：
- `http://localhost:3000`
- `http://localhost:5173`
- `http://127.0.0.1:3000`
- `http://127.0.0.1:5173`

## 功能限制

### 当前不支持的功能：
1. **Agent执行** - 需要桌面应用的完整环境
2. **Claude Code集成** - 需要本地Claude安装
3. **文件系统写入** - 出于安全考虑受限

### 只读功能：
- Agent管理 (创建、查看、列表)
- 项目和会话查看
- 历史记录访问
- 系统状态检查

## 前端集成示例

### JavaScript/TypeScript

```typescript
class OpcodeAPI {
  private baseUrl = 'http://localhost:3001/api';

  async getAgents(page?: number, limit?: number) {
    const params = new URLSearchParams();
    if (page) params.append('page', page.toString());
    if (limit) params.append('limit', limit.toString());
    
    const response = await fetch(`${this.baseUrl}/agents?${params}`);
    return response.json();
  }

  async createAgent(agent: CreateAgentRequest) {
    const response = await fetch(`${this.baseUrl}/agents`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(agent)
    });
    return response.json();
  }

  async getProjects() {
    const response = await fetch(`${this.baseUrl}/projects`);
    return response.json();
  }
}
```

### React Hook示例

```typescript
import { useState, useEffect } from 'react';

interface Agent {
  id: number;
  name: string;
  icon: string;
  system_prompt: string;
  // ... 其他字段
}

export function useAgents() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    fetch('http://localhost:3001/api/agents')
      .then(res => res.json())
      .then(data => {
        if (data.success) {
          setAgents(data.data);
        }
      })
      .finally(() => setLoading(false));
  }, []);

  return { agents, loading };
}
```

## 部署建议

### 开发环境
```bash
# 启动API服务器
./opcode api --port 3001

# 启动前端开发服务器
npm run dev
```

### 生产环境
1. 编译发布版本: `cargo build --release`
2. 配置反向代理 (nginx/apache)
3. 设置防火墙规则
4. 配置HTTPS (推荐)

### Docker化 (可选)

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/opcode /usr/local/bin/
EXPOSE 3001
CMD ["opcode", "api", "--port", "3001"]
```

## 安全注意事项

1. **生产环境请使用HTTPS**
2. **限制API访问源** - 修改CORS配置
3. **考虑添加认证机制** - JWT/API Key
4. **监控API使用情况**
5. **定期更新依赖包**

## 故障排除

### 常见问题

1. **端口被占用**
   ```bash
   Error: Address already in use
   ```
   解决：使用不同端口或停止占用进程

2. **数据库连接失败**
   确保 `~/.opcode/agents.db` 可访问

3. **CORS错误**
   检查请求来源是否在允许列表中

### 日志查看
```bash
RUST_LOG=info ./opcode api
```

## 未来计划

- [ ] 添加认证和授权
- [ ] WebSocket支持实时通信
- [ ] 文件上传/下载功能
- [ ] Agent执行的HTTP API支持
- [ ] 更多的管理功能
- [ ] 性能监控和指标
- [ ] 批量操作API

## 联系支持

如有问题，请访问：
- GitHub: https://github.com/getAsterisk/opcode
- Issues: https://github.com/getAsterisk/opcode/issues