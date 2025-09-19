#!/bin/bash

# opcode CLI 安装脚本
# 此脚本将创建 opcode 命令的全局链接，使用户可以在任何地方使用 opcode 命令

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 检测操作系统
detect_os() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "linux"* ]]; then
        echo "linux"
    else
        echo "unknown"
    fi
}

# 查找 opcode 可执行文件
find_opcode_executable() {
    local os=$(detect_os)
    local executable_name="opcode"
    
    # macOS 应用包路径
    if [[ "$os" == "macos" ]]; then
        # 在 Applications 中查找
        if [[ -f "/Applications/opcode.app/Contents/MacOS/opcode" ]]; then
            echo "/Applications/opcode.app/Contents/MacOS/opcode"
            return 0
        fi
        
        # 在用户 Applications 中查找
        if [[ -f "$HOME/Applications/opcode.app/Contents/MacOS/opcode" ]]; then
            echo "$HOME/Applications/opcode.app/Contents/MacOS/opcode"
            return 0
        fi
    fi
    
    # Linux 或构建输出路径
    if [[ -f "./src-tauri/target/release/opcode" ]]; then
        echo "$(realpath ./src-tauri/target/release/opcode)"
        return 0
    fi
    
    # 在当前目录查找
    if [[ -f "./opcode" ]]; then
        echo "$(realpath ./opcode)"
        return 0
    fi
    
    return 1
}

# 创建符号链接
create_symlink() {
    local executable_path="$1"
    local link_path="$2"
    local link_dir=$(dirname "$link_path")
    
    # 确保目标目录存在
    if [[ ! -d "$link_dir" ]]; then
        print_info "创建目录: $link_dir"
        mkdir -p "$link_dir" || {
            print_error "无法创建目录: $link_dir"
            return 1
        }
    fi
    
    # 如果链接已存在，先删除
    if [[ -L "$link_path" ]] || [[ -f "$link_path" ]]; then
        print_warning "移除现有链接: $link_path"
        rm -f "$link_path"
    fi
    
    # 创建符号链接
    print_info "创建符号链接: $link_path -> $executable_path"
    ln -s "$executable_path" "$link_path" || {
        print_error "无法创建符号链接"
        return 1
    }
    
    return 0
}

# 添加到 PATH
add_to_path() {
    local bin_dir="$1"
    local shell_rc=""
    
    # 检测用户的 shell
    if [[ -n "$ZSH_VERSION" ]] || [[ "$SHELL" == *"zsh"* ]]; then
        shell_rc="$HOME/.zshrc"
    elif [[ -n "$BASH_VERSION" ]] || [[ "$SHELL" == *"bash"* ]]; then
        shell_rc="$HOME/.bashrc"
        # macOS 默认使用 .bash_profile
        if [[ $(detect_os) == "macos" && -f "$HOME/.bash_profile" ]]; then
            shell_rc="$HOME/.bash_profile"
        fi
    else
        shell_rc="$HOME/.profile"
    fi
    
    # 检查是否已经在 PATH 中
    if echo "$PATH" | grep -q "$bin_dir"; then
        print_success "$bin_dir 已在 PATH 中"
        return 0
    fi
    
    # 添加到 shell 配置文件
    print_info "添加 $bin_dir 到 $shell_rc"
    echo "" >> "$shell_rc"
    echo "# opcode CLI" >> "$shell_rc"
    echo "export PATH=\"$bin_dir:\$PATH\"" >> "$shell_rc"
    
    print_warning "请运行 'source $shell_rc' 或重启终端来应用更改"
    return 0
}

# 主安装函数
main() {
    print_info "开始安装 opcode CLI..."
    
    # 查找可执行文件
    print_info "查找 opcode 可执行文件..."
    executable_path=$(find_opcode_executable)
    
    if [[ $? -ne 0 || -z "$executable_path" ]]; then
        print_error "未找到 opcode 可执行文件"
        print_info "请确保 opcode 已经构建或安装"
        print_info "构建命令: bun run tauri build"
        exit 1
    fi
    
    print_success "找到可执行文件: $executable_path"
    
    # 确定安装位置
    local bin_dir=""
    local link_path=""
    
    # 优先使用用户级别的 bin 目录
    if [[ -d "$HOME/.local/bin" ]]; then
        bin_dir="$HOME/.local/bin"
    elif [[ -d "$HOME/bin" ]]; then
        bin_dir="$HOME/bin"
    else
        # 创建用户级别的 bin 目录
        bin_dir="$HOME/.local/bin"
        mkdir -p "$bin_dir"
    fi
    
    link_path="$bin_dir/opcode"
    
    # 创建符号链接
    if create_symlink "$executable_path" "$link_path"; then
        print_success "符号链接创建成功"
    else
        print_error "符号链接创建失败"
        exit 1
    fi
    
    # 添加到 PATH
    add_to_path "$bin_dir"
    
    # 验证安装
    if [[ -x "$link_path" ]]; then
        print_success "opcode CLI 安装成功！"
        print_info "使用方法:"
        echo "  opcode                  # 启动 GUI 模式"
        echo "  opcode ./project        # 用指定项目启动"
        echo "  opcode /path/to/project # 用绝对路径启动"
        
        # 测试命令（如果 PATH 中已经有的话）
        if command -v opcode >/dev/null 2>&1; then
            print_success "opcode 命令已可用"
        else
            print_warning "opcode 命令尚未在当前终端会话中生效"
            print_info "请重启终端或运行 'source ~/.zshrc' (或相应的配置文件)"
        fi
    else
        print_error "安装验证失败"
        exit 1
    fi
}

# 卸载函数
uninstall() {
    print_info "卸载 opcode CLI..."
    
    # 移除符号链接
    local possible_paths=(
        "$HOME/.local/bin/opcode"
        "$HOME/bin/opcode"
        "/usr/local/bin/opcode"
    )
    
    for path in "${possible_paths[@]}"; do
        if [[ -L "$path" ]] || [[ -f "$path" ]]; then
            print_info "移除: $path"
            rm -f "$path"
        fi
    done
    
    print_success "opcode CLI 已卸载"
    print_warning "请手动从 shell 配置文件中移除 PATH 条目（如果有的话）"
}

# 显示帮助
show_help() {
    echo "opcode CLI 安装脚本"
    echo ""
    echo "用法:"
    echo "  $0 [install]    安装 opcode CLI（默认）"
    echo "  $0 uninstall    卸载 opcode CLI"
    echo "  $0 help         显示此帮助信息"
    echo ""
    echo "安装后，您可以在任何地方使用："
    echo "  opcode                  # GUI 模式"
    echo "  opcode ./project        # 指定项目路径"
}

# 解析命令行参数
case "${1:-install}" in
    "install"|"")
        main
        ;;
    "uninstall")
        uninstall
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        print_error "未知选项: $1"
        show_help
        exit 1
        ;;
esac