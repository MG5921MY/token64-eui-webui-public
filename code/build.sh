#!/bin/bash
# 构建脚本 - 编译所有平台的二进制文件

set -e

echo "🚀 开始构建 IPv6 Token WebUI"
echo ""

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 构建目标列表
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "armv7-unknown-linux-gnueabihf"
)

# 创建发布目录
mkdir -p releases

# 编译每个目标
for target in "${TARGETS[@]}"; do
    echo -e "${BLUE}📦 编译目标: $target${NC}"
    
    # 添加目标（如果未安装）
    rustup target add $target 2>/dev/null || true
    
    # 编译
    cargo build --release --target $target
    
    # 复制到发布目录
    binary_name="ipv6-webui-${target}"
    cp "target/${target}/release/ipv6-token-webui" "releases/${binary_name}"
    
    # 显示文件大小
    size=$(du -h "releases/${binary_name}" | cut -f1)
    echo -e "${GREEN}✅ 完成: releases/${binary_name} (${size})${NC}"
    echo ""
done

echo -e "${GREEN}🎉 所有平台编译完成！${NC}"
echo ""
echo "发布文件："
ls -lh releases/
