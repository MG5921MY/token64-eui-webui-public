#!/bin/bash
# 禁用 Token，恢复默认
set -e

# 配置参数（请根据实际情况修改）
CONNECTION_NAME="Wired connection 1"
INTERFACE="eth0"

echo "=== 禁用 Token ==="
echo ""

if [ "$EUID" -ne 0 ]; then 
    echo "请使用 root 运行"
    exit 1
fi

echo "当前地址:"
ip -6 addr show $INTERFACE | grep "scope global" || echo "无"
echo ""

echo "1. 清除 Token..."
nmcli connection modify "$CONNECTION_NAME" ipv6.token ""

echo "2. 恢复 DHCPv6..."
nmcli connection modify "$CONNECTION_NAME" ipv6.dhcp-timeout ""

echo "3. 恢复地址生成模式..."
nmcli connection modify "$CONNECTION_NAME" ipv6.addr-gen-mode stable-privacy

echo "4. 启用隐私扩展..."
nmcli connection modify "$CONNECTION_NAME" ipv6.ip6-privacy 2

echo "5. 清除旧地址..."
ip -6 addr flush dev $INTERFACE scope global 2>/dev/null || true

echo "6. 重启连接..."
nmcli connection down "$CONNECTION_NAME" 2>/dev/null || true
sleep 2
nmcli connection up "$CONNECTION_NAME"

echo ""
echo "等待10秒..."
sleep 10

echo ""
echo "=== 恢复完成 ==="
echo ""
echo "新地址:"
ip -6 addr show $INTERFACE | grep "scope global" || echo "无"
echo ""
echo "✅ 已恢复默认配置（DHCPv6 或 SLAAC）"
