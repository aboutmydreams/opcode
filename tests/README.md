# OpCode 自动化测试

本目录包含针对 OpCode API Server 的自动化测试套件。

## 测试文件

- `api_tests.py` - REST API 接口测试
- `websocket_tests.py` - WebSocket 接口测试  
- `run_tests.sh` - 测试运行器脚本

## 快速开始

### 1. 启动服务器
```bash
# 在项目根目录运行
cargo run
# 或
./run.sh
```

### 2. 运行所有测试
```bash
cd tests
./run_tests.sh
```

### 3. 指定服务器地址
```bash
./run_tests.sh -u http://localhost:8080
```

## 测试选项

```bash
# 显示帮助
./run_tests.sh -h

# 仅运行API测试
./run_tests.sh --api-only

# 仅运行WebSocket测试
./run_tests.sh --ws-only

# 详细输出
./run_tests.sh -v

# 设置超时时间（秒）
./run_tests.sh -t 60
```

## 单独运行测试

### API 测试
```bash
python3 api_tests.py [服务器地址]
python3 api_tests.py http://localhost:3000
```

### WebSocket 测试
```bash
python3 websocket_tests.py [服务器地址]
python3 websocket_tests.py localhost:3000
```

## 测试内容

### API 测试 (`api_tests.py`)
- 健康检查接口
- API 文档接口
- Agents 管理 API
- Claude 集成 API
- MCP 服务器管理 API
- 存储 API
- 错误处理测试

### WebSocket 测试 (`websocket_tests.py`)
- Claude WebSocket 基本连接
- 无效会话处理
- 连接管理
- 消息格式验证

## 环境要求

- Python 3.6+
- OpCode API Server 运行中
- 无额外依赖（使用 Python 标准库）

## 测试结果

测试脚本会输出：
- 每个测试的通过/失败状态
- 总体成功率
- 详细的错误信息（如果有）

成功示例：
```
=== API测试结果 ===
通过: 7/7 (100.0%)
总体状态: ✓ 成功
```

失败示例：
```
=== API测试结果 ===
通过: 5/7 (71.4%)
总体状态: ✗ 部分失败
```

## 故障排除

1. **连接失败**: 确保 API Server 正在运行
2. **权限错误**: 检查测试脚本执行权限
3. **超时**: 增加超时时间或检查服务器性能
4. **Python 版本**: 确保使用 Python 3.6+

## 扩展测试

可以通过修改测试脚本来添加更多测试用例：

1. 在 `api_tests.py` 中添加新的 `test_*` 方法
2. 在 `websocket_tests.py` 中添加新的测试场景
3. 更新 `run_all_tests()` 方法以包含新测试