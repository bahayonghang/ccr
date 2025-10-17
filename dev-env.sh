#!/usr/bin/env bash
# 🛠️ CCR 开发环境配置脚本
# 
# 此脚本设置环境变量，使CCR在开发时使用临时目录而不是真实配置
# 这样可以避免在开发测试时污染本地的真实配置文件
#
# 使用方法：
#   source dev-env.sh        # 启用开发环境
#   source dev-env.sh clean  # 清理开发环境并删除临时文件

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# 开发环境临时目录
DEV_TMP_DIR="${TMPDIR:-/tmp}/ccr_dev_$$"

# 清理函数
cleanup_dev_env() {
    echo -e "${YELLOW}${BOLD}🧹 清理开发环境${RESET}"
    
    # 取消环境变量
    unset CCR_CONFIG_PATH
    unset CCR_SETTINGS_PATH
    unset CCR_BACKUP_DIR
    unset CCR_HISTORY_PATH
    unset CCR_LOCK_DIR
    
    # 删除临时文件（如果传递了clean参数）
    if [ "$1" = "clean" ]; then
        if [ -d "$DEV_TMP_DIR" ]; then
            echo -e "${RED}🗑️  删除临时目录: $DEV_TMP_DIR${RESET}"
            rm -rf "$DEV_TMP_DIR"
        fi
    fi
    
    echo -e "${GREEN}✅ 开发环境已清理${RESET}"
}

# 设置开发环境
setup_dev_env() {
    echo -e "${CYAN}${BOLD}╔════════════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${CYAN}${BOLD}║   🛠️  CCR 开发环境配置                                        ║${RESET}"
    echo -e "${CYAN}${BOLD}╚════════════════════════════════════════════════════════════════╝${RESET}"
    echo ""
    
    # 创建临时目录
    mkdir -p "$DEV_TMP_DIR"/{backups,locks}
    echo -e "${BLUE}📁 创建临时目录: $DEV_TMP_DIR${RESET}"
    
    # 设置环境变量
    export CCR_CONFIG_PATH="$DEV_TMP_DIR/config.toml"
    export CCR_SETTINGS_PATH="$DEV_TMP_DIR/settings.json"
    export CCR_BACKUP_DIR="$DEV_TMP_DIR/backups"
    export CCR_HISTORY_PATH="$DEV_TMP_DIR/history.json"
    export CCR_LOCK_DIR="$DEV_TMP_DIR/locks"
    
    echo ""
    echo -e "${GREEN}${BOLD}✅ 开发环境已配置${RESET}"
    echo ""
    echo -e "${CYAN}📋 环境变量设置：${RESET}"
    echo -e "  ${BOLD}CCR_CONFIG_PATH${RESET}  = ${YELLOW}$CCR_CONFIG_PATH${RESET}"
    echo -e "  ${BOLD}CCR_SETTINGS_PATH${RESET} = ${YELLOW}$CCR_SETTINGS_PATH${RESET}"
    echo -e "  ${BOLD}CCR_BACKUP_DIR${RESET}    = ${YELLOW}$CCR_BACKUP_DIR${RESET}"
    echo -e "  ${BOLD}CCR_HISTORY_PATH${RESET}  = ${YELLOW}$CCR_HISTORY_PATH${RESET}"
    echo -e "  ${BOLD}CCR_LOCK_DIR${RESET}      = ${YELLOW}$CCR_LOCK_DIR${RESET}"
    echo ""
    echo -e "${CYAN}💡 提示：${RESET}"
    echo -e "  • 现在可以安全地运行 ${BOLD}cargo run${RESET} 或 ${BOLD}just run${RESET}"
    echo -e "  • 所有文件操作都在临时目录中进行"
    echo -e "  • 不会影响你的真实配置文件"
    echo -e "  • 使用 ${BOLD}source dev-env.sh clean${RESET} 清理并删除临时文件"
    echo ""
}

# 主逻辑
if [ "$1" = "clean" ]; then
    cleanup_dev_env clean
else
    setup_dev_env
fi
