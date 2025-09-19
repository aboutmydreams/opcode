# WebSocket 测试方法

## 方法 1: 使用 JavaScript/Node.js

### 安装依赖
```bash
npm install ws node-fetch
```

### 测试脚本 (test-websocket.js)
```javascript
const WebSocket = require('ws');
const fetch = require('node-fetch');

async function testClaudeWebSocket() {
    try {
        // 1. 启动 Claude Code 会话
        console.log('🚀 启动 Claude Code 会话...');
        const response = await fetch('http://localhost:3000/claude/execute', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                project_path: '/Users/apple/coding/project/ccAgent/opcode',
                prompt: '列出当前目录下的文件',
                model: 'claude-3-5-sonnet-20241022',
                session_type: 'new'
            })
        });

        const result = await response.json();
        console.log('✅ 会话已启动:', result);

        const sessionId = result.session_id;

        // 2. 连接 WebSocket
        console.log(`🔌 连接 WebSocket: ws://localhost:3000/ws/claude/${sessionId}`);
        const ws = new WebSocket(`ws://localhost:3000/ws/claude/${sessionId}`);

        // 3. 监听消息
        ws.on('open', () => {
            console.log('✅ WebSocket 连接已建立');
        });

        ws.on('message', (data) => {
            const message = data.toString();
            console.log('📨 收到消息:', message);
            
            try {
                const parsed = JSON.parse(message);
                if (parsed.type === 'complete') {
                    console.log('🎉 会话完成！');
                    ws.close();
                }
            } catch (e) {
                // 普通文本消息
            }
        });

        ws.on('error', (error) => {
            console.error('❌ WebSocket 错误:', error);
        });

        ws.on('close', () => {
            console.log('🔒 WebSocket 连接已关闭');
        });

        // 10 秒后自动取消（如果还在运行）
        setTimeout(async () => {
            if (ws.readyState === WebSocket.OPEN) {
                console.log('⏰ 10秒超时，取消会话...');
                await fetch(`http://localhost:3000/claude/cancel/${sessionId}`, {
                    method: 'POST'
                });
            }
        }, 10000);

    } catch (error) {
        console.error('❌ 错误:', error);
    }
}

// 运行测试
testClaudeWebSocket();
```

### 运行测试
```bash
node test-websocket.js
```

## 方法 2: 使用 Python

### 安装依赖
```bash
pip install websockets requests asyncio
```

### 测试脚本 (test_websocket.py)
```python
import asyncio
import json
import requests
import websockets

async def test_claude_websocket():
    try:
        # 1. 启动 Claude Code 会话
        print("🚀 启动 Claude Code 会话...")
        response = requests.post('http://localhost:3000/claude/execute', json={
            'project_path': '/Users/apple/coding/project/ccAgent/opcode',
            'prompt': '列出当前目录下的文件',
            'model': 'claude-3-5-sonnet-20241022',
            'session_type': 'new'
        })
        
        result = response.json()
        print(f"✅ 会话已启动: {result}")
        
        session_id = result['session_id']
        
        # 2. 连接 WebSocket
        websocket_url = f"ws://localhost:3000/ws/claude/{session_id}"
        print(f"🔌 连接 WebSocket: {websocket_url}")
        
        async with websockets.connect(websocket_url) as websocket:
            print("✅ WebSocket 连接已建立")
            
            # 3. 监听消息
            try:
                async for message in websocket:
                    print(f"📨 收到消息: {message}")
                    
                    try:
                        parsed = json.loads(message)
                        if parsed.get('type') == 'complete':
                            print("🎉 会话完成！")
                            break
                    except json.JSONDecodeError:
                        # 普通文本消息
                        pass
                        
            except websockets.exceptions.ConnectionClosed:
                print("🔒 WebSocket 连接已关闭")
                
    except Exception as error:
        print(f"❌ 错误: {error}")

# 运行测试
if __name__ == "__main__":
    asyncio.run(test_claude_websocket())
```

### 运行测试
```bash
python test_websocket.py
```

## 方法 3: 使用 curl 和 websocat

### 安装 websocat
```bash
# macOS
brew install websocat

# Linux
wget https://github.com/vi/websocat/releases/download/v1.11.0/websocat.x86_64-unknown-linux-musl
chmod +x websocat.x86_64-unknown-linux-musl
sudo mv websocat.x86_64-unknown-linux-musl /usr/local/bin/websocat
```

### 测试步骤

1. **启动会话**:
```bash
curl -X POST http://localhost:3000/claude/execute \
  -H "Content-Type: application/json" \
  -d '{
    "project_path": "/Users/apple/coding/project/ccAgent/opcode",
    "prompt": "列出当前目录下的文件",
    "model": "claude-3-5-sonnet-20241022",
    "session_type": "new"
  }'
```

2. **记录返回的 session_id**，然后连接 WebSocket:
```bash
websocat ws://localhost:3000/ws/claude/YOUR_SESSION_ID
```

3. **取消会话**（在另一个终端）:
```bash
curl -X POST http://localhost:3000/claude/cancel/YOUR_SESSION_ID
```

## 方法 4: 使用浏览器开发者工具

在浏览器控制台中运行：

```javascript
// 1. 启动会话
const response = await fetch('http://localhost:3000/claude/execute', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
        project_path: '/Users/apple/coding/project/ccAgent/opcode',
        prompt: '列出当前目录下的文件',
        model: 'claude-3-5-sonnet-20241022',
        session_type: 'new'
    })
});

const result = await response.json();
console.log('会话启动:', result);

// 2. 连接 WebSocket
const ws = new WebSocket(`ws://localhost:3000/ws/claude/${result.session_id}`);

ws.onopen = () => console.log('WebSocket 连接已建立');
ws.onmessage = (event) => console.log('收到消息:', event.data);
ws.onerror = (error) => console.error('WebSocket 错误:', error);
ws.onclose = () => console.log('WebSocket 连接已关闭');
```

## 预期输出示例

```
🚀 启动 Claude Code 会话...
✅ 会话已启动: {
  "session_id": "abc123-def456-ghi789",
  "status": "started", 
  "websocket_url": "/ws/claude/abc123-def456-ghi789"
}
🔌 连接 WebSocket: ws://localhost:3000/ws/claude/abc123-def456-ghi789
✅ WebSocket 连接已建立
📨 收到消息: {"type":"system","subtype":"init","session_id":"real-claude-session-id"}
📨 收到消息: {"type":"user_message","content":"列出当前目录下的文件"}
📨 收到消息: {"type":"assistant_message","content":"我来帮你列出当前目录下的文件..."}
📨 收到消息: {"type":"complete","success":true,"code":0}
🎉 会话完成！
🔒 WebSocket 连接已关闭
```

## 故障排除

1. **连接被拒绝**: 确保 API 服务器在运行 (`cargo run`)
2. **Claude binary not found**: 安装 Claude Code CLI 或设置正确的二进制路径
3. **权限错误**: 确保项目路径存在且有访问权限
4. **WebSocket 立即关闭**: 检查服务器日志查看具体错误信息