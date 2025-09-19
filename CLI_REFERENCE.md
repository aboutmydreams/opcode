# opcode CLI 快速参考

## 概述
opcode 现在支持命令行接口 (CLI)，允许用户直接从终端打开特定项目。

## 安装 CLI

### 1. 构建应用
```bash
# 开发构建
bun run tauri build --debug

# 生产构建  
bun run tauri build
```

### 2. 安装 CLI 支持
```bash
# 运行安装脚本
./scripts/install-cli.sh

# 验证安装
opcode --help
```

## 使用方法

### 基本用法
```bash
# GUI 模式 (无参数)
opcode

# 打开特定项目
opcode <项目路径>
```

### 示例
```bash
# 打开当前目录
opcode .

# 相对路径
opcode ./my-project
opcode ../other-project

# 绝对路径  
opcode /Users/username/projects/my-app
opcode ~/coding/website

# Windows 路径
opcode C:\projects\my-app
```

## 工作原理

1. **CLI 模式检测**: 当 opcode 启动时检查命令行参数
2. **路径解析**: 将相对路径转换为绝对路径
3. **路径验证**: 检查提供的路径是否存在
4. **GUI 启动**: 启动 GUI 并传递项目路径信息
5. **自动加载**: 前端自动检测并加载指定的项目

## 技术实现

### 后端 (Rust)
- `main.rs`: 处理命令行参数和路径解析
- `InitialProjectPath` 状态: 存储初始项目路径
- `get_initial_project_path` 命令: 前端获取初始路径的接口

### 前端 (React/TypeScript)
- `App.tsx`: 在应用启动时检查初始项目路径
- 自动匹配和加载对应的 Claude 项目
- 显示成功/错误通知

### 安装脚本 (`scripts/install-cli.sh`)
- 自动检测操作系统 (macOS/Linux)
- 查找 opcode 可执行文件
- 创建符号链接到用户 bin 目录
- 更新 PATH 环境变量

## 故障排除

### 常见问题

**1. "opcode: command not found"**
```bash
# 重新运行安装脚本
./scripts/install-cli.sh

# 手动添加到 PATH
export PATH="$HOME/.local/bin:$PATH"

# 或重启终端
```

**2. "Path does not exist"**
```bash
# 检查路径是否正确
ls -la /path/to/project

# 使用绝对路径
opcode $(realpath ./project)
```

**3. "Project not found in Claude projects"**  
- 确保项目路径在 `~/.claude/projects/` 中被 Claude Code CLI 识别
- 先在项目目录中运行 `claude` 初始化项目

### 卸载 CLI
```bash
# 运行卸载
./scripts/install-cli.sh uninstall

# 手动清理 (如需要)
rm -f ~/.local/bin/opcode
rm -f ~/bin/opcode
```

## 与现有工作流的集成

### VS Code 用户
可以在 VS Code 终端中直接使用:
```bash
# 打开当前工作区
opcode .
```

### 项目快速访问
```bash
# 创建项目快捷别名
alias myproject="opcode ~/projects/my-important-project"

# 在 shell 配置中添加
echo 'alias myproject="opcode ~/projects/my-important-project"' >> ~/.zshrc
```

### 与其他工具链结合
```bash
# 在构建脚本中使用
#!/bin/bash
cd /path/to/project
npm run build
opcode .  # 构建完成后打开 opcode
```

## 路线图

未来可能的改进:
- [ ] 支持更多命令行参数 (`--agent`, `--session` 等)
- [ ] 支持批量打开多个项目
- [ ] 与系统文件管理器集成
- [ ] Windows 安装程序中自动配置 CLI

---

如需帮助或反馈，请在 GitHub 提交 Issue。