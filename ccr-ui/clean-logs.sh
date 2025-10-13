#!/bin/bash
# 日志清理脚本
# 自动删除14天前的日志文件

set -e

LOG_DIR="logs"
RETENTION_DAYS=14

echo "🧹 开始清理日志文件..."
echo "📁 日志目录: $LOG_DIR"
echo "⏰ 保留天数: $RETENTION_DAYS 天"
echo ""

# 确保日志目录存在
if [ ! -d "$LOG_DIR" ]; then
    echo "❌ 日志目录不存在: $LOG_DIR"
    exit 0
fi

# 计算14天前的日期
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    CUTOFF_DATE=$(date -v-${RETENTION_DAYS}d '+%Y-%m-%d')
else
    # Linux
    CUTOFF_DATE=$(date -d "$RETENTION_DAYS days ago" '+%Y-%m-%d')
fi

echo "📅 删除 $CUTOFF_DATE 之前的日志文件"
echo ""

# 统计信息
DELETED_COUNT=0
DELETED_SIZE=0

# 查找并删除旧的日志文件
# 日志文件命名模式: backend.log.2025-10-01, frontend.log.2025-10-01
while IFS= read -r -d '' logfile; do
    # 提取日期部分
    if [[ "$logfile" =~ ([0-9]{4}-[0-9]{2}-[0-9]{2}) ]]; then
        FILE_DATE="${BASH_REMATCH[1]}"
        
        # 比较日期
        if [[ "$FILE_DATE" < "$CUTOFF_DATE" ]]; then
            FILE_SIZE=$(stat -f%z "$logfile" 2>/dev/null || stat -c%s "$logfile" 2>/dev/null || echo 0)
            echo "🗑️  删除: $logfile ($FILE_SIZE bytes)"
            rm -f "$logfile"
            DELETED_COUNT=$((DELETED_COUNT + 1))
            DELETED_SIZE=$((DELETED_SIZE + FILE_SIZE))
        fi
    fi
done < <(find "$LOG_DIR" -name "*.log.*" -type f -print0)

# 转换字节数为可读格式
if [ $DELETED_SIZE -gt 0 ]; then
    if [ $DELETED_SIZE -gt 1048576 ]; then
        SIZE_MB=$((DELETED_SIZE / 1048576))
        SIZE_DISPLAY="${SIZE_MB} MB"
    elif [ $DELETED_SIZE -gt 1024 ]; then
        SIZE_KB=$((DELETED_SIZE / 1024))
        SIZE_DISPLAY="${SIZE_KB} KB"
    else
        SIZE_DISPLAY="${DELETED_SIZE} bytes"
    fi
fi

echo ""
echo "✅ 清理完成！"
echo "📊 删除文件数: $DELETED_COUNT"
if [ $DELETED_COUNT -gt 0 ]; then
    echo "💾 释放空间: $SIZE_DISPLAY"
fi

# 列出当前保留的日志文件
echo ""
echo "📋 当前日志文件:"
ls -lh "$LOG_DIR"/*.log 2>/dev/null | awk '{print "   " $9 " (" $5 ")"}'
ls -lh "$LOG_DIR"/*.log.* 2>/dev/null | awk '{print "   " $9 " (" $5 ")"}' | head -10

# 统计当前日志总大小
TOTAL_SIZE=$(du -sh "$LOG_DIR" 2>/dev/null | awk '{print $1}')
echo ""
echo "📦 日志目录总大小: $TOTAL_SIZE"

