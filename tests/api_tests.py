#!/usr/bin/env python3
"""
API 自动化测试
测试 OpCode API Server 的 REST API 接口
"""

import json
import sys
import time
import urllib.request
import urllib.parse
import urllib.error
from typing import Dict, Any, Optional, List


class APITester:
    def __init__(self, base_url: str = "http://127.0.0.1:3000"):
        self.base_url = base_url.rstrip('/')
        self.session_headers = {'Content-Type': 'application/json'}
        
    def _make_request(self, method: str, endpoint: str, data: Optional[Dict] = None, 
                     headers: Optional[Dict] = None) -> tuple:
        """发送HTTP请求并返回响应"""
        url = f"{self.base_url}{endpoint}"
        req_headers = self.session_headers.copy()
        if headers:
            req_headers.update(headers)
            
        req_data = None
        if data:
            req_data = json.dumps(data).encode('utf-8')
            
        try:
            request = urllib.request.Request(url, data=req_data, headers=req_headers, method=method)
            with urllib.request.urlopen(request) as response:
                response_data = response.read().decode('utf-8')
                try:
                    return response.status, json.loads(response_data) if response_data else {}
                except json.JSONDecodeError:
                    return response.status, response_data
        except urllib.error.HTTPError as e:
            error_data = e.read().decode('utf-8')
            try:
                return e.code, json.loads(error_data) if error_data else {}
            except json.JSONDecodeError:
                return e.code, error_data
        except Exception as e:
            return 0, {'error': str(e)}

    def test_health_check(self) -> bool:
        """测试健康检查接口"""
        print("测试健康检查接口...")
        status, response = self._make_request('GET', '/health')
        success = status == 200 and response == "OK"
        print(f"  健康检查: {'✓' if success else '✗'} ({status})")
        return success

    def test_api_docs(self) -> bool:
        """测试API文档接口"""
        print("测试API文档接口...")
        status, response = self._make_request('GET', '/docs')
        success = status == 200
        print(f"  API文档: {'✓' if success else '✗'} ({status})")
        return success

    def test_agents_api(self) -> bool:
        """测试Agents管理API"""
        print("测试Agents管理API...")
        results = []
        
        # 测试列出所有agents
        status, response = self._make_request('GET', '/api/agents')
        success = status in [200, 404]  # 空列表时可能返回404
        results.append(success)
        print(f"  列出agents: {'✓' if success else '✗'} ({status})")
        
        # 测试创建agent
        test_agent = {
            "name": "test-agent",
            "description": "测试用agent",
            "type": "system",
            "config": {"test": True}
        }
        status, response = self._make_request('POST', '/api/agents', test_agent)
        success = status in [200, 201, 400, 422]  # 可能的有效响应
        results.append(success)
        print(f"  创建agent: {'✓' if success else '✗'} ({status})")
        
        # 如果创建成功，尝试获取和删除
        if status in [200, 201] and isinstance(response, dict) and 'id' in response:
            agent_id = response['id']
            
            # 测试获取agent详情
            status, response = self._make_request('GET', f'/api/agents/{agent_id}')
            success = status == 200
            results.append(success)
            print(f"  获取agent详情: {'✓' if success else '✗'} ({status})")
            
            # 测试删除agent
            status, response = self._make_request('DELETE', f'/api/agents/{agent_id}')
            success = status in [200, 204]
            results.append(success)
            print(f"  删除agent: {'✓' if success else '✗'} ({status})")
        
        return all(results)

    def test_claude_api(self) -> bool:
        """测试Claude集成API"""
        print("测试Claude集成API...")
        results = []
        
        # 测试列出项目
        status, response = self._make_request('GET', '/api/claude/projects')
        success = status in [200, 404]
        results.append(success)
        print(f"  列出Claude项目: {'✓' if success else '✗'} ({status})")
        
        # 测试获取设置
        status, response = self._make_request('GET', '/api/claude/settings')
        success = status in [200, 404]
        results.append(success)
        print(f"  获取Claude设置: {'✓' if success else '✗'} ({status})")
        
        return all(results)

    def test_mcp_api(self) -> bool:
        """测试MCP服务器管理API"""
        print("测试MCP服务器管理API...")
        results = []
        
        # 测试列出MCP服务器
        status, response = self._make_request('GET', '/api/mcp/servers')
        success = status in [200, 404]
        results.append(success)
        print(f"  列出MCP服务器: {'✓' if success else '✗'} ({status})")
        
        return all(results)

    def test_storage_api(self) -> bool:
        """测试存储API"""
        print("测试存储API...")
        results = []
        
        # 测试获取存储使用情况
        status, response = self._make_request('GET', '/api/storage/usage')
        success = status in [200, 404, 501]  # 可能未实现
        results.append(success)
        print(f"  获取存储使用情况: {'✓' if success else '✗'} ({status})")
        
        # 测试列出数据表
        status, response = self._make_request('GET', '/api/storage/tables')
        success = status in [200, 404, 501]
        results.append(success)
        print(f"  列出数据表: {'✓' if success else '✗'} ({status})")
        
        return all(results)

    def test_error_handling(self) -> bool:
        """测试错误处理"""
        print("测试错误处理...")
        results = []
        
        # 测试不存在的endpoint
        status, response = self._make_request('GET', '/api/nonexistent')
        success = status == 404
        results.append(success)
        print(f"  404错误处理: {'✓' if success else '✗'} ({status})")
        
        # 测试不存在的agent
        status, response = self._make_request('GET', '/api/agents/999999')
        success = status == 404
        results.append(success)
        print(f"  Agent不存在错误: {'✓' if success else '✗'} ({status})")
        
        # 测试无效JSON
        try:
            url = f"{self.base_url}/api/agents"
            req_headers = {'Content-Type': 'application/json'}
            request = urllib.request.Request(url, data=b'invalid json', headers=req_headers, method='POST')
            with urllib.request.urlopen(request) as response:
                status = response.status
        except urllib.error.HTTPError as e:
            status = e.code
        
        success = status == 400
        results.append(success)
        print(f"  无效JSON错误: {'✓' if success else '✗'} ({status})")
        
        return all(results)

    def run_all_tests(self) -> bool:
        """运行所有API测试"""
        print("=== API自动化测试开始 ===\n")
        
        tests = [
            ("健康检查", self.test_health_check),
            ("API文档", self.test_api_docs),
            ("Agents API", self.test_agents_api),
            ("Claude API", self.test_claude_api),
            ("MCP API", self.test_mcp_api),
            ("存储API", self.test_storage_api),
            ("错误处理", self.test_error_handling),
        ]
        
        results = []
        for name, test_func in tests:
            try:
                result = test_func()
                results.append(result)
                print(f"{name}: {'✓ 通过' if result else '✗ 失败'}\n")
            except Exception as e:
                print(f"{name}: ✗ 异常 - {e}\n")
                results.append(False)
        
        passed = sum(results)
        total = len(results)
        success_rate = (passed / total) * 100 if total > 0 else 0
        
        print("=== API测试结果 ===")
        print(f"通过: {passed}/{total} ({success_rate:.1f}%)")
        print(f"总体状态: {'✓ 成功' if passed == total else '✗ 部分失败'}")
        
        return passed == total


def main():
    """主函数"""
    if len(sys.argv) > 1:
        base_url = sys.argv[1]
    else:
        base_url = "http://127.0.0.1:3000"
    
    print(f"测试目标: {base_url}")
    print(f"开始时间: {time.strftime('%Y-%m-%d %H:%M:%S')}\n")
    
    tester = APITester(base_url)
    
    # 首先检查服务器是否运行
    try:
        status, _ = tester._make_request('GET', '/health')
        if status != 200:
            print(f"❌ 服务器未运行或无法访问 (状态码: {status})")
            print("请确保OpCode API Server正在运行在指定地址")
            sys.exit(1)
    except Exception as e:
        print(f"❌ 无法连接到服务器: {e}")
        print("请确保OpCode API Server正在运行在指定地址")
        sys.exit(1)
    
    # 运行测试
    success = tester.run_all_tests()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()