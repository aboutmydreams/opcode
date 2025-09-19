#!/bin/bash
# 
# OpCode API Server 自动化测试运行器
# 运行所有API和WebSocket测试
#

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认配置
BASE_URL="http://127.0.0.1:3000"
TIMEOUT=30
VERBOSE=false
RUN_API=true
RUN_WEBSOCKET=true

# 脚本目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 打印帮助信息
show_help() {
    cat << EOF
OpCode API Server 自动化测试运行器

用法: $0 [选项]

选项:
    -u, --url URL        设置服务器URL (默认: $BASE_URL)
    -t, --timeout SEC    设置超时时间 (默认: $TIMEOUT 秒)
    -v, --verbose        详细输出
    -a, --api-only       仅运行API测试
    -w, --ws-only        仅运行WebSocket测试
    -h, --help           显示此帮助信息

示例:
    $0                                    # 运行所有测试
    $0 -u http://localhost:8080           # 指定服务器地址
    $0 --api-only                         # 仅测试API
    $0 --ws-only -v                       # 仅测试WebSocket，详细输出

环境要求:
    - Python 3.6+
    - OpCode API Server 运行中
EOF
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -u|--url)
            BASE_URL="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -a|--api-only)
            RUN_WEBSOCKET=false
            shift
            ;;
        -w|--ws-only)
            RUN_API=false
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}未知参数: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# 检查Python
check_python() {
    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}❌ 错误: 未找到 python3${NC}"
        echo "请安装 Python 3.6 或更高版本"
        exit 1
    fi
    
    # 检查Python版本
    python_version=$(python3 -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
    if [[ $(echo "$python_version 3.6" | awk '{print ($1 >= $2)}') -eq 0 ]]; then
        echo -e "${YELLOW}⚠️  警告: Python版本 $python_version 可能不兼容，建议使用 3.6+${NC}"
    fi
}

# 检查服务器
check_server() {
    echo -e "${BLUE}🔍 检查服务器状态...${NC}"
    
    # 直接使用curl或python检查健康状态，而不是手动解析URL和端口
    local health_check_success=false
    
    # 尝试使用curl检查健康状态
    if command -v curl &> /dev/null; then
        health_status=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 "$BASE_URL/health" 2>/dev/null || echo "000")
        if [[ "$health_status" == "200" ]]; then
            health_check_success=true
        fi
    fi
    
    # 如果curl失败，尝试使用python
    if [[ "$health_check_success" == "false" ]] && command -v python3 &> /dev/null; then
        if python3 -c "
import urllib.request
import sys
try:
    with urllib.request.urlopen('$BASE_URL/health', timeout=5) as response:
        sys.exit(0 if response.status == 200 else 1)
except:
    sys.exit(1)
" 2>/dev/null; then
            health_check_success=true
        fi
    fi
    
    if [[ "$health_check_success" == "false" ]]; then
        echo -e "${RED}❌ 无法连接到服务器或健康检查失败${NC}"
        echo "请确保 OpCode API Server 正在 $BASE_URL 运行"
        exit 1
    fi
    
    echo -e "${GREEN}✓ 服务器运行正常${NC}"
}

# 运行单个测试
run_test() {
    local test_name="$1"
    local test_script="$2"
    local test_args="$3"
    
    echo -e "${BLUE}🧪 运行 $test_name 测试...${NC}"
    
    if [[ ! -f "$test_script" ]]; then
        echo -e "${RED}❌ 测试脚本不存在: $test_script${NC}"
        return 1
    fi
    
    # 设置超时
    local timeout_cmd=""
    if command -v timeout &> /dev/null; then
        timeout_cmd="timeout $TIMEOUT"
    fi
    
    # 运行测试
    local start_time=$(date +%s)
    local test_output
    local test_exit_code
    
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${YELLOW}执行命令: python3 $test_script $test_args${NC}"
        $timeout_cmd python3 "$test_script" $test_args
        test_exit_code=$?
    else
        test_output=$($timeout_cmd python3 "$test_script" $test_args 2>&1)
        test_exit_code=$?
        
        # 如果测试失败，显示输出
        if [[ $test_exit_code -ne 0 ]]; then
            echo -e "${RED}测试输出:${NC}"
            echo "$test_output"
        fi
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    if [[ $test_exit_code -eq 0 ]]; then
        echo -e "${GREEN}✓ $test_name 测试通过 (耗时: ${duration}s)${NC}"
        return 0
    elif [[ $test_exit_code -eq 124 ]]; then
        echo -e "${RED}✗ $test_name 测试超时 (>${TIMEOUT}s)${NC}"
        return 1
    else
        echo -e "${RED}✗ $test_name 测试失败 (退出码: $test_exit_code, 耗时: ${duration}s)${NC}"
        return 1
    fi
}

# 主测试流程
main() {
    echo -e "${BLUE}=== OpCode API Server 自动化测试 ===${NC}"
    echo -e "目标服务器: ${YELLOW}$BASE_URL${NC}"
    echo -e "超时设置: ${YELLOW}${TIMEOUT}s${NC}"
    echo -e "开始时间: ${YELLOW}$(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo ""
    
    # 检查环境
    check_python
    check_server
    echo ""
    
    # 测试结果统计
    local total_tests=0
    local passed_tests=0
    local failed_tests=()
    
    # 运行API测试
    if [[ "$RUN_API" == "true" ]]; then
        total_tests=$((total_tests + 1))
        if run_test "API" "$SCRIPT_DIR/api_tests.py" "$BASE_URL"; then
            passed_tests=$((passed_tests + 1))
        else
            failed_tests+=("API")
        fi
        echo ""
    fi
    
    # 运行WebSocket测试  
    if [[ "$RUN_WEBSOCKET" == "true" ]]; then
        total_tests=$((total_tests + 1))
        if run_test "WebSocket" "$SCRIPT_DIR/websocket_tests.py" "${BASE_URL#http://}"; then
            passed_tests=$((passed_tests + 1))
        else
            failed_tests+=("WebSocket")
        fi
        echo ""
    fi
    
    # 总结
    echo -e "${BLUE}=== 测试总结 ===${NC}"
    echo -e "总测试数: ${YELLOW}$total_tests${NC}"
    echo -e "通过: ${GREEN}$passed_tests${NC}"
    echo -e "失败: ${RED}$((total_tests - passed_tests))${NC}"
    
    if [[ ${#failed_tests[@]} -gt 0 ]]; then
        echo -e "失败的测试: ${RED}${failed_tests[*]}${NC}"
    fi
    
    local success_rate=0
    if [[ $total_tests -gt 0 ]]; then
        success_rate=$(( (passed_tests * 100) / total_tests ))
    fi
    echo -e "成功率: ${YELLOW}${success_rate}%${NC}"
    
    echo -e "结束时间: ${YELLOW}$(date '+%Y-%m-%d %H:%M:%S')${NC}"
    
    # 退出状态
    if [[ $passed_tests -eq $total_tests ]]; then
        echo -e "${GREEN}🎉 所有测试通过!${NC}"
        exit 0
    else
        echo -e "${RED}❌ 有测试失败${NC}"
        exit 1
    fi
}

# 捕获中断信号
trap 'echo -e "\n${YELLOW}测试被中断${NC}"; exit 130' INT TERM

# 运行主程序
main "$@"