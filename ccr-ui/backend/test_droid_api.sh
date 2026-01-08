#!/bin/bash
# Droid API 测试脚本
# 测试 Custom Models 和 Profiles API

set -e

BASE_URL="http://localhost:8081/api"
DROID_API="$BASE_URL/droid"

echo "🧪 开始测试 Droid API..."
echo "================================"

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试结果统计
PASSED=0
FAILED=0

# 测试函数
test_api() {
    local name="$1"
    local method="$2"
    local url="$3"
    local data="$4"
    local expected_status="$5"

    echo -e "\n${YELLOW}测试: $name${NC}"
    echo "请求: $method $url"

    if [ -n "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$url")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$url")
    fi

    # 分离响应体和状态码
    body=$(echo "$response" | head -n -1)
    status=$(echo "$response" | tail -n 1)

    echo "状态码: $status"
    echo "响应: $body" | jq '.' 2>/dev/null || echo "$body"

    if [ "$status" = "$expected_status" ]; then
        echo -e "${GREEN}✓ 通过${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ 失败 (期望: $expected_status, 实际: $status)${NC}"
        ((FAILED++))
    fi
}

echo -e "\n${YELLOW}=== 1. Custom Models API 测试 ===${NC}"

# 1.1 列出所有自定义模型（初始应该为空）
test_api "列出自定义模型" "GET" "$DROID_API/models" "" "200"

# 1.2 添加自定义模型
test_api "添加自定义模型" "POST" "$DROID_API/models" '{
  "model": "claude-sonnet-4-5",
  "displayName": "Claude Sonnet 4.5",
  "baseUrl": "https://api.anthropic.com/v1",
  "apiKey": "sk-ant-test-key",
  "provider": "anthropic",
  "maxOutputTokens": 8192
}' "200"

# 1.3 再次列出模型（应该有一个）
test_api "列出自定义模型（添加后）" "GET" "$DROID_API/models" "" "200"

# 1.4 更新自定义模型
test_api "更新自定义模型" "PUT" "$DROID_API/models/claude-sonnet-4-5" '{
  "model": "claude-sonnet-4-5",
  "displayName": "Claude Sonnet 4.5 Updated",
  "baseUrl": "https://api.anthropic.com/v1",
  "apiKey": "sk-ant-test-key-updated",
  "provider": "anthropic",
  "maxOutputTokens": 16384
}' "200"

# 1.5 删除自定义模型
test_api "删除自定义模型" "DELETE" "$DROID_API/models/claude-sonnet-4-5" "" "200"

# 1.6 验证删除（应该为空）
test_api "列出自定义模型（删除后）" "GET" "$DROID_API/models" "" "200"

echo -e "\n${YELLOW}=== 2. Profiles API 测试 ===${NC}"

# 2.1 列出所有 Profiles（初始应该为空或有默认）
test_api "列出 Profiles" "GET" "$DROID_API/profiles" "" "200"

# 2.2 添加 Profile
test_api "添加 Profile" "POST" "$DROID_API/profiles" '{
  "name": "test-profile",
  "description": "Test Profile",
  "base_url": "https://api.anthropic.com/v1",
  "api_key": "sk-ant-test-key",
  "model": "claude-sonnet-4-5",
  "provider": "anthropic",
  "provider_type": "anthropic",
  "max_output_tokens": 8192,
  "display_name": "Test Profile",
  "enabled": true
}' "200"

# 2.3 再次列出 Profiles（应该有一个）
test_api "列出 Profiles（添加后）" "GET" "$DROID_API/profiles" "" "200"

# 2.4 更新 Profile
test_api "更新 Profile" "PUT" "$DROID_API/profiles/test-profile" '{
  "description": "Updated Test Profile",
  "base_url": "https://api.anthropic.com/v1",
  "api_key": "sk-ant-test-key-updated",
  "model": "claude-sonnet-4-5",
  "provider": "anthropic",
  "provider_type": "anthropic",
  "max_output_tokens": 16384,
  "display_name": "Updated Test Profile",
  "enabled": true
}' "200"

# 2.5 切换 Profile
test_api "切换 Profile" "POST" "$DROID_API/profiles/test-profile/switch" "" "200"

# 2.6 删除 Profile
test_api "删除 Profile" "DELETE" "$DROID_API/profiles/test-profile" "" "200"

# 2.7 验证删除
test_api "列出 Profiles（删除后）" "GET" "$DROID_API/profiles" "" "200"

echo -e "\n${YELLOW}=== 3. MCP API 测试 ===${NC}"

# 3.1 列出 MCP 服务器
test_api "列出 MCP 服务器" "GET" "$DROID_API/mcp" "" "200"

# 3.2 添加 MCP 服务器
test_api "添加 MCP 服务器" "POST" "$DROID_API/mcp" '{
  "name": "test-mcp",
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-filesystem"],
  "timeout": 30000
}' "200"

# 3.3 列出 MCP 服务器（添加后）
test_api "列出 MCP 服务器（添加后）" "GET" "$DROID_API/mcp" "" "200"

# 3.4 删除 MCP 服务器
test_api "删除 MCP 服务器" "DELETE" "$DROID_API/mcp/test-mcp" "" "200"

echo -e "\n${YELLOW}================================${NC}"
echo -e "测试完成！"
echo -e "${GREEN}通过: $PASSED${NC}"
echo -e "${RED}失败: $FAILED${NC}"
echo -e "总计: $((PASSED + FAILED))"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 所有测试通过！${NC}"
    exit 0
else
    echo -e "\n${RED}❌ 有测试失败${NC}"
    exit 1
fi
