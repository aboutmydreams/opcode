# WebSocket API 文档

## 概述

OpCode API Server 提供 WebSocket 接口，用于与 Claude Code 进行实时交互。用户可以启动 Claude Code 会话并通过 WebSocket 接收实时的流式输出。

## API 端点

### 1. 启动 Claude Code 会话

**端点**: `POST /claude/execute`

**请求体**:
```json
{
  "project_path": "/path/to/your/project",
  "prompt": "你想要 Claude 执行的任务",
  "model": "claude-3-5-sonnet-20241022",
  "session_type": "new",
  "session_id": "optional-for-resume"
}
```

**参数说明**:
- `project_path`: 项目路径（绝对路径）
- `prompt`: 发送给 Claude 的提示
- `model`: Claude 模型名称
- `session_type`: 会话类型
  - `"new"`: 启动新会话
  - `"continue"`: 继续当前项目的最新会话
  - `"resume"`: 恢复指定的会话（需要提供 session_id）
- `session_id`: 仅在 `session_type` 为 `"resume"` 时需要

**响应**:
```json
{
  "session_id": "uuid-string",
  "status": "started",
  "websocket_url": "/ws/claude/{session_id}"
}
```

### 2. WebSocket 连接

**端点**: `WS /ws/claude/{session_id}`

连接到此端点可以接收 Claude Code 的实时输出。

**消息格式**:

1. **普通输出消息**（Claude Code 的原始 JSON 输出）:
```json
{
  "type": "system",
  "subtype": "init",
  "session_id": "actual-claude-session-id",
  "timestamp": "2025-01-19T10:30:00Z"
}
```

2. **错误消息**:
```json
{
  "type": "error",
  "message": "错误描述"
}
```

3. **完成消息**:
```json
{
  "type": "complete",
  "success": true,
  "code": 0
}
```

4. **取消消息**:
```json
{
  "type": "cancelled",
  "session_id": "session-id"
}
```

### 3. 取消会话

**端点**: `POST /claude/cancel/{session_id}`

**响应**:
```json
{
  "session_id": "session-id",
  "cancelled": true
}
```

## 使用流程

1. **启动会话**: 调用 `POST /claude/execute` 获取 `session_id`
2. **连接 WebSocket**: 使用 `session_id` 连接到 `ws://localhost:3000/ws/claude/{session_id}`
3. **接收消息**: 监听 WebSocket 消息接收实时输出
4. **取消会话**（可选）: 调用 `POST /claude/cancel/{session_id}`

## 错误处理

- Claude binary 未找到: HTTP 500，消息 "Claude binary not found"
- 项目路径无效: HTTP 500，消息 "Failed to spawn Claude process"
- 会话不存在: 取消时返回 `"cancelled": false`

## 注意事项

- WebSocket 连接会在 Claude Code 进程结束后自动关闭
- 同一个 session_id 可以被多个 WebSocket 客户端同时连接
- 会话在完成或取消后会自动清理
- 需要确保 Claude Code CLI 已安装且在 PATH 中可用

## 示例消息流

```
1. POST /claude/execute → 返回 session_id
2. WS 连接建立
3. 接收: {"type": "system", "subtype": "init", ...}
4. 接收: {"type": "user_message", "content": "..."}
5. 接收: {"type": "assistant_message", "content": "..."}
6. 接收: {"type": "complete", "success": true, "code": 0}
7. WS 连接关闭
```