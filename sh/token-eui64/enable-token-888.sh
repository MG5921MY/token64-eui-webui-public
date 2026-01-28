#!/bin/bash
# 启用 Token ::888 并使用临时 IPv6 地址优先出站
set -e

# 配置参数（请根据实际情况修改）
CONNECTION_NAME="Wired connection 1"
INTERFACE="eth0"
TOKEN="::888"
PRIVACY_MODE=2  # 0=禁用临时IPv6, 2=启用临时IPv6优先出站

echo "=== 启用 IPv6 Token $TOKEN + 临时地址优先出站 ==="
echo ""

# 检查 root
if [ "$EUID" -ne 0 ]; then 
    echo "请使用 root 运行"
    exit 1
fi

echo "配置前地址:"
ip -6 addr show $INTERFACE | grep "scope global" || echo "无"
echo ""

# 核心配置（注意顺序很重要！）
echo "1. 配置 IPv6 方法..."
nmcli connection modify "$CONNECTION_NAME" ipv6.method auto

echo "2. 禁用 DHCPv6（关键）..."
nmcli connection modify "$CONNECTION_NAME" ipv6.dhcp-timeout 0

echo "3. 设置地址生成模式（必须在 Token 之前）..."
nmcli connection modify "$CONNECTION_NAME" ipv6.addr-gen-mode eui64

echo "4. 设置 Token..."
nmcli connection modify "$CONNECTION_NAME" ipv6.token "$TOKEN"

echo "5. 设置隐私模式 ($PRIVACY_MODE)..."
if [ "$PRIVACY_MODE" -eq 0 ]; then
    echo "   禁用临时IPv6地址"
    nmcli connection modify "$CONNECTION_NAME" ipv6.ip6-privacy 0
elif [ "$PRIVACY_MODE" -eq 2 ]; then
    echo "   启用临时IPv6地址并优先出站"
    nmcli connection modify "$CONNECTION_NAME" ipv6.ip6-privacy 2
fi

echo "6. 清除旧地址..."
ip -6 addr flush dev $INTERFACE scope global 2>/dev/null || true

echo "7. 重启连接..."
nmcli connection down "$CONNECTION_NAME" 2>/dev/null || true
sleep 2
nmcli connection up "$CONNECTION_NAME"

echo ""
echo "等待10秒..."
sleep 10

echo ""
echo "=== 配置完成 ==="
echo ""
echo "新地址:"
ip -6 addr show $INTERFACE | grep "scope global"
echo ""
echo "Token状态:"
nmcli connection show "$CONNECTION_NAME" | grep "ipv6.token"
ip token show dev $INTERFACE 2>/dev/null || echo "系统 Token: 未检测到"
echo ""

# 验证
TOKEN_SUFFIX="${TOKEN#::}"
if ip -6 addr show $INTERFACE | grep "scope global" | grep -q "$TOKEN_SUFFIX"; then
    echo "✅ Token $TOKEN 已生效！"
else
    echo "⚠️  Token 可能未生效，请运行 check-token.sh 检查"
fi

# 验证默认出站地址
if [ "$PRIVACY_MODE" -eq 2 ]; then
    echo ""
    echo "默认出站地址:"
    ip -6 route get 2400:3200::1 2>/dev/null || echo "无法获取路由信息"
fi
