# OpCode API Server 设计文档

## 架构概述

将原有90个Tauri command重构为RESTful API + WebSocket架构，分4个阶段实施。

## API 模块设计

### 1. 核心模块 (Core - 优先级1)

#### `/api/agents` - Agent管理
```
GET    /agents                    # 列出所有agents
POST   /agents                    # 创建agent  
GET    /agents/{id}               # 获取agent详情
PUT    /agents/{id}               # 更新agent
DELETE /agents/{id}               # 删除agent
POST   /agents/{id}/execute       # 执行agent
DELETE /agents/{id}/sessions/{sid} # 终止agent会话
GET    /agents/{id}/runs          # agent运行历史
GET    /agents/{id}/runs/{rid}    # 特定运行详情
POST   /agents/import             # 导入agent
GET    /agents/export/{id}        # 导出agent
```

#### `/api/claude` - Claude Code集成
```
GET    /claude/projects           # 列出项目
POST   /claude/projects           # 创建项目
GET    /claude/projects/{id}/sessions # 项目会话列表
POST   /claude/execute            # 执行新会话
POST   /claude/continue           # 继续对话
POST   /claude/resume             # 恢复会话
DELETE /claude/sessions/{id}      # 取消执行
GET    /claude/settings           # 获取设置
PUT    /claude/settings           # 保存设置
```

### 2. 扩展模块 (Extended - 优先级2)

#### `/api/mcp` - MCP服务器管理
```
GET    /mcp/servers               # 列出MCP服务器
POST   /mcp/servers               # 添加服务器
GET    /mcp/servers/{name}        # 获取服务器详情
DELETE /mcp/servers/{name}        # 删除服务器
POST   /mcp/servers/{name}/test   # 测试连接
POST   /mcp/import                # 从Claude Desktop导入
```

#### `/api/storage` - 数据存储
```
GET    /storage/tables            # 列出数据表
GET    /storage/tables/{name}     # 获取表数据
POST   /storage/tables/{name}/rows # 插入行
PUT    /storage/tables/{name}/rows/{id} # 更新行
DELETE /storage/tables/{name}/rows/{id} # 删除行
POST   /storage/sql               # 执行SQL (受限)
```

#### `/api/usage` - 使用统计
```
GET    /usage/stats               # 总体统计
GET    /usage/stats/date-range    # 按日期范围
GET    /usage/details             # 详细数据
GET    /usage/sessions            # 会话统计
```

### 3. 系统模块 (System - 优先级3)

#### `/api/commands` - 自定义命令
```
GET    /commands                  # 列出斜杠命令
GET    /commands/{id}             # 获取命令详情
POST   /commands                  # 创建命令
PUT    /commands/{id}             # 更新命令
DELETE /commands/{id}             # 删除命令
```

#### `/api/proxy` - 代理设置
```
GET    /proxy/settings            # 获取代理设置
PUT    /proxy/settings            # 保存代理设置
```

## WebSocket设计

### 实时通信端点
```
WS /ws/agents/{id}/execution       # Agent执行实时输出
WS /ws/claude/sessions/{id}        # Claude会话实时消息
WS /ws/system/events               # 系统事件通知
```

### 消息格式
```json
{
  "type": "execution_output" | "session_message" | "system_event",
  "data": { ... },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## 数据模型优化

### 统一错误响应
```json
{
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent with ID 123 not found",
    "details": { ... }
  }
}
```

### 分页响应
```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "hasNext": true
  }
}
```

## 安全性设计

1. **API Key认证** - 所有API需要认证
2. **SQL注入防护** - storage API限制SQL操作
3. **文件访问控制** - 限制文件系统访问范围
4. **资源限制** - 限制并发执行数量

## 实施计划

### 阶段1: 核心功能 (1-2周)
- Agents管理完整API
- Claude基础集成
- WebSocket实时通信框架

### 阶段2: 扩展功能 (2-3周)  
- MCP服务器管理
- 数据存储操作
- 使用统计分析

### 阶段3: 系统功能 (1周)
- 自定义命令管理
- 代理设置
- 系统优化

### 阶段4: 完善优化 (1周)
- 性能优化
- 安全加固
- 文档完善

## 技术栈优化

### 新增依赖
```toml
# WebSocket支持
axum-extra = { version = "0.9", features = ["ws"] }
tokio-tungstenite = "0.21"

# 认证和安全
jsonwebtoken = "9"
bcrypt = "0.15"

# 配置管理
config = "0.14"
serde_yaml = "0.9"

# 日志和监控
tracing = "0.1"
tracing-subscriber = "0.3"
metrics = "0.22"
```

## 关键改进点

1. **API设计统一** - RESTful风格，资源导向
2. **错误处理标准化** - 统一错误格式和状态码  
3. **实时通信** - WebSocket支持长连接
4. **安全性增强** - 认证、授权、输入验证
5. **可观测性** - 完整的日志和指标系统

这个设计将90个分散的command整合为约45个RESTful端点，大大简化了API复杂度。