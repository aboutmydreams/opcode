# WebSocket API 测试指南

这里提供了多种测试 OpCode API Server WebSocket 功能的方法。

## 📋 测试前准备

1. **启动 API 服务器**:
   ```bash
   cd api-server
   cargo run
   ```

2. **确保 Claude Code CLI 已安装** (可选，用于完整测试):
   - 如未安装，API 会返回相应错误信息

## 🔧 测试方法

### 方法 1: HTML 可视化测试工具 ⭐ **推荐**

最简单直观的测试方法：

1. 用浏览器打开 `websocket-test.html`
2. 填写项目路径和提示内容
3. 点击"启动会话"
4. 观察实时日志输出

**特点**:
- ✅ 可视化界面，易于使用
- ✅ 实时显示所有消息
- ✅ 支持所有会话类型（新建/继续/恢复）
- ✅ 内置错误处理和状态显示

### 方法 2: 命令行工具

#### 使用 Node.js
```bash
npm install ws node-fetch
node test-websocket.js
```

#### 使用 Python
```bash
pip install websockets requests
python test_websocket.py
```

#### 使用 curl + websocat
```bash
# 1. 启动会话
curl -X POST http://localhost:3000/claude/execute \
  -H "Content-Type: application/json" \
  -d '{"project_path":"/path/to/project","prompt":"列出文件","model":"claude-3-5-sonnet-20241022","session_type":"new"}'

# 2. 连接 WebSocket (使用返回的 session_id)
websocat ws://localhost:3000/ws/claude/YOUR_SESSION_ID
```

### 方法 3: 浏览器开发者工具

在浏览器控制台直接运行 JavaScript 代码进行测试。

## 📁 文件说明

- `WEBSOCKET_API.md` - 完整的 API 文档
- `WEBSOCKET_TESTING.md` - 详细的测试方法和代码示例
- `websocket-test.html` - 可视化测试工具
- `test-websocket.js` - Node.js 测试脚本
- `test_websocket.py` - Python 测试脚本

## 🎯 快速开始

**最快测试方式**:
1. `cargo run` (启动服务器)
2. 打开 `websocket-test.html` 
3. 点击"启动会话"

## 🐛 常见问题

1. **连接被拒绝**: 确保 API 服务器在运行
2. **Claude binary not found**: 这是正常的，表示 WebSocket 连接正常工作
3. **项目路径错误**: 使用绝对路径，确保路径存在

## 📊 预期行为

- ✅ API 返回 session_id 和 websocket_url
- ✅ WebSocket 连接成功建立  
- ✅ 如果 Claude CLI 存在，会收到实时输出
- ✅ 如果 Claude CLI 不存在，会收到错误信息
- ✅ 取消功能正常工作
- ✅ 连接在会话结束后自动关闭