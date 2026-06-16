#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
诊断脚本：检查 llmusage 和 ccr-db 数据源状态

用法：
    python diagnose_data_source.py
"""

import sqlite3
import os
import sys
from pathlib import Path
from datetime import datetime, timedelta

# Windows 终端编码修复
if sys.platform == 'win32':
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')


def find_llmusage_db():
    """查找 llmusage 数据库路径"""
    llmusage_home = os.environ.get('LLMUSAGE_HOME')
    if llmusage_home:
        return Path(llmusage_home) / 'llmusage.db'

    home = Path.home()
    return home / '.llmusage' / 'llmusage.db'


def find_ccr_db():
    """查找 ccr-db 数据库路径"""
    home = Path.home()
    # ccr-db 实际路径是 ~/.ccr-ui/ccr-ui.db（根据启动日志）
    candidates = [
        home / '.ccr-ui' / 'ccr-ui.db',
        home / '.ccr-ui' / 'ccr.db',
        home / '.ccr' / 'ccr.db',
        home / '.config' / 'ccr' / 'ccr.db',
        Path.cwd() / '.ccr' / 'ccr.db',
    ]
    for path in candidates:
        if path.exists():
            return path
    return candidates[0]  # 返回默认路径，即使不存在


def diagnose_llmusage(db_path):
    """诊断 llmusage 数据源"""
    print(f"\n{'='*60}")
    print(f"诊断 llmusage 数据源")
    print(f"{'='*60}")
    print(f"数据库路径: {db_path}")

    if not db_path.exists():
        print("❌ 数据库文件不存在！")
        return False

    print(f"✅ 数据库文件存在 (大小: {db_path.stat().st_size / 1024:.1f} KB)")

    try:
        conn = sqlite3.connect(str(db_path))
        cursor = conn.cursor()

        # 检查 schema 版本
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table'")
        tables = [row[0] for row in cursor.fetchall()]
        print(f"\n表列表 ({len(tables)} 个):")
        for table in sorted(tables):
            print(f"  - {table}")

        # 检查 buckets 表（聚合数据）
        # llmusage 新版本使用 usage_bucket_30m 而非 buckets
        bucket_table = 'usage_bucket_30m' if 'usage_bucket_30m' in tables else 'buckets'

        if bucket_table in tables:
            cursor.execute(f"SELECT COUNT(*) FROM {bucket_table} WHERE source = 'claude'")
            bucket_count = cursor.fetchone()[0]
            print(f"\n✅ {bucket_table} 表: {bucket_count} 条 Claude 记录")

            if bucket_count > 0:
                # 检查表结构
                cursor.execute(f"PRAGMA table_info({bucket_table})")
                columns = [row[1] for row in cursor.fetchall()]
                print(f"  表字段: {', '.join(columns[:10])}{'...' if len(columns) > 10 else ''}")

                cursor.execute(f"""
                    SELECT
                        date(hour_start, 'localtime') as date,
                        COUNT(*) as count
                    FROM {bucket_table}
                    WHERE source = 'claude'
                    GROUP BY date
                    ORDER BY date DESC
                    LIMIT 5
                """)
                print("  最近 5 天的记录分布:")
                for row in cursor.fetchall():
                    print(f"    {row[0]}: {row[1]} 条")
        else:
            print(f"\n❌ {bucket_table} 表不存在")

        # 检查 events 表（明细数据，用于 trends_daily）
        # llmusage 新版本使用 usage_event 而非 events
        event_table = 'usage_event' if 'usage_event' in tables else 'events'

        if event_table in tables:
            cursor.execute(f"SELECT COUNT(*) FROM {event_table} WHERE source = 'claude'")
            event_count = cursor.fetchone()[0]
            print(f"\n✅ {event_table} 表: {event_count} 条 Claude 记录")

            if event_count > 0:
                # 检查表结构
                cursor.execute(f"PRAGMA table_info({event_table})")
                columns = [row[1] for row in cursor.fetchall()]
                print(f"  表字段: {', '.join(columns[:10])}{'...' if len(columns) > 10 else ''}")

                # 检查最近 30 天的每日事件数
                thirty_days_ago = (datetime.now() - timedelta(days=30)).strftime('%Y-%m-%d')
                cursor.execute(f"""
                    SELECT
                        date(event_at, 'localtime') as date,
                        COUNT(*) as count,
                        SUM(input_tokens + output_tokens + cache_read_tokens) as total_tokens
                    FROM {event_table}
                    WHERE source = 'claude'
                      AND date(event_at, 'localtime') >= ?
                    GROUP BY date
                    ORDER BY date DESC
                    LIMIT 10
                """, (thirty_days_ago,))
                daily_data = cursor.fetchall()
                print(f"  最近 30 天的每日记录 ({len(daily_data)} 天有数据):")
                for row in daily_data:
                    print(f"    {row[0]}: {row[1]} 条事件, {row[2]:,} tokens")

                if len(daily_data) == 0:
                    print("  ⚠️  最近 30 天无数据！这是 daily_trend 返回空的原因。")
        else:
            print(f"\n❌ {event_table} 表不存在")

        # 检查 projects 表（用于 project_breakdown）
        if bucket_table in tables:
            cursor.execute(f"""
                SELECT COUNT(DISTINCT project_hash)
                FROM {bucket_table}
                WHERE source = 'claude'
            """)
            project_count = cursor.fetchone()[0]
            print(f"\n✅ 项目数: {project_count} 个不同项目")

            if project_count > 0:
                cursor.execute(f"""
                    SELECT
                        project_hash,
                        project_ref,
                        COUNT(*) as bucket_count
                    FROM {bucket_table}
                    WHERE source = 'claude'
                    GROUP BY project_hash
                    ORDER BY bucket_count DESC
                    LIMIT 5
                """)
                print("  Top 5 项目:")
                for row in cursor.fetchall():
                    print(f"    {row[0][:16]}... ({row[1]}): {row[2]} 条记录")

        conn.close()
        return event_count > 0 if event_table in tables else False

    except sqlite3.Error as e:
        print(f"❌ 数据库查询错误: {e}")
        return False


def diagnose_ccr_db(db_path):
    """诊断 ccr-db 数据源"""
    print(f"\n{'='*60}")
    print(f"诊断 ccr-db 数据源")
    print(f"{'='*60}")
    print(f"数据库路径: {db_path}")

    if not db_path.exists():
        print("❌ 数据库文件不存在！")
        return False

    print(f"✅ 数据库文件存在 (大小: {db_path.stat().st_size / 1024:.1f} KB)")

    try:
        conn = sqlite3.connect(str(db_path))
        cursor = conn.cursor()

        # 检查表列表
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table'")
        tables = [row[0] for row in cursor.fetchall()]
        print(f"\n表列表 ({len(tables)} 个):")
        for table in sorted(tables):
            print(f"  - {table}")

        # 检查 claude_tool_calls 表
        if 'claude_tool_calls' in tables:
            cursor.execute("SELECT COUNT(*) FROM claude_tool_calls")
            call_count = cursor.fetchone()[0]
            print(f"\n✅ claude_tool_calls 表: {call_count} 条记录")

            if call_count > 0:
                # 检查最近的记录
                cursor.execute("""
                    SELECT
                        session_id,
                        tool_name,
                        cost_usd,
                        called_at
                    FROM claude_tool_calls
                    ORDER BY called_at DESC
                    LIMIT 5
                """)
                print("  最近 5 条工具调用:")
                for row in cursor.fetchall():
                    print(f"    {row[0][:16]}... | {row[1]} | ${row[2]:.4f} | {row[3]}")

                # 检查不同 session 数
                cursor.execute("SELECT COUNT(DISTINCT session_id) FROM claude_tool_calls")
                session_count = cursor.fetchone()[0]
                print(f"\n  Session 总数: {session_count}")

                # 检查不同工具数
                cursor.execute("""
                    SELECT tool_name, COUNT(*) as count
                    FROM claude_tool_calls
                    GROUP BY tool_name
                    ORDER BY count DESC
                    LIMIT 10
                """)
                print("  Top 10 工具:")
                for row in cursor.fetchall():
                    print(f"    {row[0]}: {row[1]} 次调用")
            else:
                print("  ⚠️  表为空！这是 tool_heatmap/top_tools 返回空的原因。")
        else:
            print("\n❌ claude_tool_calls 表不存在")

        conn.close()
        return call_count > 0 if 'claude_tool_calls' in tables else False

    except sqlite3.Error as e:
        print(f"❌ 数据库查询错误: {e}")
        return False


def main():
    print("="*60)
    print("Claude Code Usage Insight 数据源诊断")
    print("="*60)

    # 诊断 llmusage
    llmusage_db = find_llmusage_db()
    llmusage_ok = diagnose_llmusage(llmusage_db)

    # 诊断 ccr-db
    ccr_db = find_ccr_db()
    ccr_db_ok = diagnose_ccr_db(ccr_db)

    # 总结
    print(f"\n{'='*60}")
    print("诊断总结")
    print(f"{'='*60}")

    if llmusage_ok and ccr_db_ok:
        print("✅ 两个数据源都有数据")
        print("\n建议：")
        print("  1. 检查前端是否正确调用 Tauri 命令")
        print("  2. 查看浏览器 Console 是否有错误日志")
        print("  3. 查看 Tauri 日志是否有查询错误")
    elif llmusage_ok and not ccr_db_ok:
        print("⚠️  llmusage 有数据，但 ccr-db 为空")
        print("\n建议：")
        print("  1. 检查 claude_tool_calls 表的初始化逻辑")
        print("  2. 触发工具调用记录导入")
    elif not llmusage_ok and ccr_db_ok:
        print("⚠️  ccr-db 有数据，但 llmusage events 表为空")
        print("\n建议：")
        print("  1. 运行 llmusage sync/collect 导入数据")
        print("  2. 检查 llmusage 数据导入机制")
    else:
        print("❌ 两个数据源都为空")
        print("\n建议：")
        print("  1. 运行 llmusage sync/collect 导入 token 使用记录")
        print("  2. 检查 ccr-db 表初始化逻辑")
        print("  3. 确认是否有实际的 Claude Code 使用记录")

    sys.exit(0 if (llmusage_ok and ccr_db_ok) else 1)


if __name__ == '__main__':
    main()
