# opcode 开发文档

## 项目概述

opcode 是一个基于 Tauri 2 的跨平台桌面应用，为 Claude Code CLI 提供强大的图形界面和工具集。该项目采用现代化的全栈技术架构，结合 Rust 的高性能后端和 React 的丰富前端生态。

## 技术架构

### 技术栈
- **前端**: React 18 + TypeScript + Vite 6
- **后端**: Rust (Tauri 2)
- **UI框架**: Tailwind CSS v4 + shadcn/ui
- **数据库**: SQLite (rusqlite)
- **包管理**: Bun
- **构建工具**: Tauri CLI

### 架构图
```
┌─────────────────────────────────────────────────────────────┐
│                    opcode 应用架构                          │
├─────────────────────────────────────────────────────────────┤
│  前端 (React + TypeScript)                                  │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │  组件层     │ │   状态管理   │ │   工具库     │           │
│  │ components/ │ │   stores/   │ │    lib/     │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────┤
│  Tauri 桥接层 (IPC 通信)                                    │
├─────────────────────────────────────────────────────────────┤
│  后端 (Rust)                                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │ Commands    │ │ Checkpoint  │ │  Process    │           │
│  │ 命令处理层   │ │  检查点管理  │ │  进程管理   │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
├─────────────────────────────────────────────────────────────┤
│  数据层 (SQLite)                                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐           │
│  │   代理存储   │ │  会话历史   │ │  使用统计   │           │
│  └─────────────┘ └─────────────┘ └─────────────┘           │
└─────────────────────────────────────────────────────────────┘
```

## 核心模块详解

### 1. 前端模块 (src/)

#### 组件架构
```
src/components/
├── 项目管理
│   ├── ProjectList.tsx        # 项目列表
│   ├── ProjectSettings.tsx    # 项目设置
│   └── SessionList.tsx       # 会话列表
├── AI代理
│   ├── Agents.tsx            # 代理管理界面
│   ├── CreateAgent.tsx       # 创建代理
│   ├── AgentExecution.tsx    # 代理执行
│   └── AgentRunView.tsx      # 运行视图
├── Claude 集成
│   ├── ClaudeCodeSession.tsx # Claude 会话
│   ├── ClaudeBinaryDialog.tsx# 二进制设置
│   └── ClaudeFileEditor.tsx  # 文件编辑器
├── 检查点系统
│   ├── CheckpointSettings.tsx# 检查点设置  
│   └── TimelineNavigator.tsx # 时间线导航
└── UI 组件
    ├── ui/                   # shadcn/ui 组件库
    └── widgets/              # 自定义小部件
```

#### 状态管理
```
src/stores/
├── agentStore.ts      # 代理状态管理 (Zustand)
└── sessionStore.ts    # 会话状态管理
```

#### 服务层
```
src/services/
├── sessionPersistence.ts  # 会话持久化
└── tabPersistence.ts     # 标签持久化
```

### 2. 后端模块 (src-tauri/src/)

#### 命令层 (commands/)
- **agents.rs**: 代理生命周期管理，执行，数据库操作
- **claude.rs**: Claude Code CLI 集成，项目管理，会话处理
- **mcp.rs**: Model Context Protocol 服务器管理
- **usage.rs**: 使用统计和分析
- **storage.rs**: 数据库直接操作接口
- **proxy.rs**: 代理设置管理
- **slash_commands.rs**: 斜杠命令系统

#### 核心系统
- **checkpoint/**: 检查点和时间线管理系统
  - `manager.rs`: 检查点管理器
  - `state.rs`: 状态跟踪
  - `storage.rs`: 持久化存储
- **process/**: 进程管理和注册表
  - `registry.rs`: 进程注册表
- **claude_binary.rs**: Claude 二进制文件检测和管理

### 3. 数据层

#### SQLite 数据库结构
```sql
-- 代理配置表
agents (id, name, description, system_prompt, model, icon, permissions, created_at)

-- 代理运行历史
agent_runs (id, agent_id, project_path, status, output, metrics, started_at, finished_at)

-- 应用设置
app_settings (key, value)

-- 使用统计
usage_logs (timestamp, model, tokens_input, tokens_output, cost, project)

-- 检查点数据
checkpoints (id, session_id, message, timestamp, file_changes)
```

## 开发工作流

### 环境设置
```bash
# 1. 安装依赖
bun install

# 2. 开发模式 (前后端热重载)
bun run tauri dev

# 3. 类型检查
bun run check
```

### 代码规范
1. **TypeScript**: 严格类型检查，使用接口定义
2. **Rust**: 遵循 Rust 社区标准，使用 cargo fmt
3. **组件**: 功能组件 + Hooks，避免类组件
4. **状态**: Zustand 管理全局状态，本地状态使用 useState

### 测试策略
```bash
# Rust 测试
cd src-tauri && cargo test

# 前端测试 (需要添加)
# bun test
```

## 构建和部署

### 开发构建
```bash
bun run tauri build --debug
```

### 生产构建  
```bash
bun run tauri build
```

### 平台特定
```bash
# macOS 通用二进制
bun run tauri build --target universal-apple-darwin

# macOS DMG
bun run build:dmg
```

## API 接口文档

### Tauri Commands

#### 项目管理
- `list_projects()`: 获取所有项目
- `create_project(name, path)`: 创建新项目
- `get_project_sessions(project_path)`: 获取项目会话

#### 代理管理
- `list_agents()`: 获取所有代理
- `create_agent(agent_config)`: 创建代理
- `execute_agent(agent_id, project_path, prompt)`: 执行代理

#### 会话管理
- `execute_claude_code(project_path, prompt)`: 执行 Claude 代码
- `get_claude_session_output(session_id)`: 获取会话输出
- `cancel_claude_execution(session_id)`: 取消执行

#### 检查点系统
- `create_checkpoint(session_id, message)`: 创建检查点
- `restore_checkpoint(checkpoint_id)`: 恢复检查点
- `get_session_timeline(session_id)`: 获取时间线

### 前端 API 调用示例
```typescript
import { invoke } from '@tauri-apps/api/core';

// 获取项目列表
const projects = await invoke<Project[]>('list_projects');

// 执行代理
const result = await invoke<AgentRunResult>('execute_agent', {
  agentId: 'agent-123',
  projectPath: '/path/to/project',
  prompt: 'Analyze this code'
});
```

## 扩展开发指南

### 添加新功能模块

1. **后端 (Rust)**:
   ```rust
   // src-tauri/src/commands/new_feature.rs
   #[tauri::command]
   pub async fn new_feature_command() -> Result<String, String> {
       Ok("Success".to_string())
   }
   ```

2. **前端 (React)**:
   ```typescript
   // src/components/NewFeature.tsx  
   export function NewFeature() {
     const handleAction = async () => {
       await invoke('new_feature_command');
     };
     return <button onClick={handleAction}>New Feature</button>;
   }
   ```

3. **注册命令** (main.rs):
   ```rust
   .invoke_handler(tauri::generate_handler![
       new_feature_command
   ])
   ```

### 数据库变更

1. 修改初始化脚本 (agents.rs 中的 init_database)
2. 添加迁移逻辑处理旧版本兼容
3. 更新相关的 CRUD 操作函数

### UI 组件开发

1. 使用 shadcn/ui 作为基础组件
2. 遵循 Tailwind CSS 设计系统
3. 实现响应式设计
4. 支持深色/浅色主题切换

## 性能优化

### 前端优化
- 使用 React.memo 避免不必要的重渲染
- 虚拟化长列表 (@tanstack/react-virtual)
- 代码分割和懒加载

### 后端优化  
- 异步处理长时间运行的任务
- 数据库查询优化和索引
- 进程池管理避免频繁创建销毁

### 构建优化
- Tauri 配置 release 优化 (Cargo.toml)
- Vite 构建优化配置
- 资源压缩和缓存策略

## 故障排除

### 常见问题
1. **构建失败**: 检查 Rust/Node.js/Bun 版本兼容性
2. **数据库锁定**: 确保正确关闭数据库连接
3. **进程僵尸**: 实现进程清理机制
4. **内存泄漏**: 监控长时间运行的会话

### 调试工具
- Tauri DevTools (开发模式)
- Rust 日志 (env_logger)
- 浏览器开发者工具
- SQLite 数据库查看器

## 贡献指南

1. Fork 项目并创建功能分支
2. 遵循代码规范和测试要求
3. 提交清晰的 commit 信息
4. 创建 Pull Request 并描述变更

---

这份文档会随着项目发展持续更新。如有疑问，请查阅源码或提交 Issue。