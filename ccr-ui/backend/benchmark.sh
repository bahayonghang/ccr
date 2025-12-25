#!/usr/bin/env bash
# 性能基准测试脚本
# 测试缓存优化前后的性能差异

set -e

echo "🚀 CCR UI Backend 缓存性能基准测试"
echo "======================================"
echo ""

# 检查后端是否在运行
if ! curl -s http://127.0.0.1:8081/api/version > /dev/null 2>&1; then
    echo "❌ 后端未运行，请先启动: cargo run --release"
    exit 1
fi

echo "✅ 后端已运行"
echo ""

# 测试端点
ENDPOINT="http://127.0.0.1:8081/api/claude/agents"

# 预热请求
echo "🔥 预热缓存..."
curl -s "$ENDPOINT" > /dev/null
echo "✅ 预热完成"
echo ""

# 性能测试函数
benchmark() {
    local name=$1
    local count=$2

    echo "📊 测试: $name ($count 次请求)"

    local start=$(date +%s%N)
    for i in $(seq 1 $count); do
        curl -s "$ENDPOINT" > /dev/null
    done
    local end=$(date +%s%N)

    local duration_ns=$((end - start))
    local duration_ms=$((duration_ns / 1000000))
    local avg_ms=$((duration_ms / count))

    echo "  总耗时: ${duration_ms}ms"
    echo "  平均耗时: ${avg_ms}ms/请求"
    echo "  吞吐量: $((count * 1000 / duration_ms)) req/s"
    echo ""
}

# 运行基准测试
echo "═══════════════════════════════════════"
echo "📈 缓存命中性能测试"
echo "═══════════════════════════════════════"
echo ""

benchmark "10 次请求" 10
benchmark "50 次请求" 50
benchmark "100 次请求" 100

echo "═══════════════════════════════════════"
echo "✅ 基准测试完成！"
echo "═══════════════════════════════════════"
echo ""
echo "💡 预期结果:"
echo "  - 缓存命中: < 5ms/请求"
echo "  - 吞吐量: > 200 req/s"
echo "  - 性能提升: 50-100x (相比无缓存)"
