#!/bin/bash
# 开发运行脚本 - 快速启动开发服务器

set -e

echo "🚀 启动 IPv6 Token WebUI 开发服务器"
echo ""

# 设置环境变量
export RUST_LOG=debug
export SERVER_HOST="0.0.0.0"
export SERVER_PORT="5000"
export SCRIPT_DIR="../sh/token-eui64"
export FRONTEND_DIR="./frontend"

echo "📝 配置信息："
echo "  - 监听地址: ${SERVER_HOST}:${SERVER_PORT}"
echo "  - 脚本目录: ${SCRIPT_DIR}"
echo "  - 前端目录: ${FRONTEND_DIR}"
echo "  - 日志级别: ${RUST_LOG}"
echo ""

# 检查脚本目录
if [ ! -d "$SCRIPT_DIR" ]; then
    echo "❌ 错误: 脚本目录不存在: $SCRIPT_DIR"
    echo "请确保在正确的目录运行此脚本"
    exit 1
fi

# 检查前端目录
if [ ! -d "$FRONTEND_DIR" ]; then
    echo "❌ 错误: 前端目录不存在: $FRONTEND_DIR"
    exit 1
fi

echo "✅ 目录检查通过"
echo ""

# 运行开发服务器
echo "🔧 启动服务器..."
echo ""
cargo run

