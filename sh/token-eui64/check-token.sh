#!/bin/bash
# 检查 Token 状态（改进版 - 支持多地址分析）

# 配置参数（请根据实际情况修改）
CONNECTION_NAME="Wired connection 1"
INTERFACE="eth0"

echo "========================================"
echo "  IPv6 Token 状态检查"
echo "========================================"
echo ""

# 1. NetworkManager 配置
echo "【NetworkManager 配置】"
echo "----------------------------------------"
nmcli connection show "$CONNECTION_NAME" | grep -E "ipv6.method|ipv6.token|ipv6.addr-gen-mode|ipv6.ip6-privacy|ipv6.dhcp-timeout"
echo ""

# 2. IPv6 地址列表
echo "【IPv6 地址列表】"
echo "----------------------------------------"
ip -6 addr show $INTERFACE | grep "inet6"
echo ""

# 3. 全局地址详细分析
echo "【全局地址分析】"
echo "----------------------------------------"

# 获取所有全局地址
GLOBAL_ADDRS=$(ip -6 addr show $INTERFACE | grep "scope global" | grep -v "temporary" | awk '{print $2}')
TOKEN_NM=$(nmcli connection show "$CONNECTION_NAME" | grep "ipv6.token:" | awk '{print $2}')

if [ -z "$GLOBAL_ADDRS" ]; then
    echo "❌ 未检测到全局地址"
else
    # 分析每个地址
    ADDR_COUNT=0
    HAS_TOKEN=false
    HAS_DHCPV6=false
    HAS_SLAAC=false
    TOKEN_ADDR=""
    DHCPV6_ADDR=""
  
    echo "检测到 $(echo "$GLOBAL_ADDRS" | wc -l) 个全局地址:"
    echo ""
  
    while IFS= read -r addr_with_prefix; do
        ADDR_COUNT=$((ADDR_COUNT + 1))
        ADDR=$(echo $addr_with_prefix | cut -d'/' -f1)
        PREFIX_LEN=$(echo $addr_with_prefix | cut -d'/' -f2)
  
        echo "地址 #$ADDR_COUNT: $ADDR"
        echo "  前缀长度: /$PREFIX_LEN"
  
        if [ "$PREFIX_LEN" = "128" ]; then
            echo "  类型: DHCPv6"
            echo "  优先级: ⭐⭐⭐ (高)"
            echo "  说明: 由 DHCPv6 服务器分配的单个地址"
            HAS_DHCPV6=true
            DHCPV6_ADDR="$ADDR"
        elif [ "$PREFIX_LEN" = "64" ]; then
            # 检查是否是Token地址
            if [ "$TOKEN_NM" != "--" ] && echo "$ADDR" | grep -q "${TOKEN_NM#::}"; then
                echo "  类型: SLAAC + Token"
                echo "  优先级: ⭐⭐ (中)"
                echo "  说明: ✅ Token 地址已生效！"
                HAS_TOKEN=true
                TOKEN_ADDR="$ADDR"
            elif echo "$ADDR" | grep -q "ff:fe"; then
                echo "  类型: SLAAC + EUI64"
                echo "  优先级: ⭐ (低)"
                echo "  说明: 基于 MAC 地址的 EUI64 格式"
                HAS_SLAAC=true
            else
                echo "  类型: SLAAC + 隐私扩展"
                echo "  优先级: ⭐ (低)"
                echo "  说明: 隐私扩展或其他方式生成"
                HAS_SLAAC=true
            fi
        fi
        echo ""
    done <<< "$GLOBAL_ADDRS"
fi

echo ""

# 4. 地址优先级说明
echo "【地址使用优先级】"
echo "----------------------------------------"
if [ "$HAS_DHCPV6" = true ] && [ "$HAS_TOKEN" = true ]; then
    echo "⚠️  系统同时拥有 DHCPv6 和 Token 地址"
    echo ""
    echo "外出连接可能使用的地址:"
    echo "  首选: $DHCPV6_ADDR (/128 DHCPv6)"
    echo "  备选: $TOKEN_ADDR (/64 Token)"
    echo ""
    echo "说明:"
    echo "  - Linux 地址选择算法通常优先使用 /128 地址"
    echo "  - 应用程序可能随机选择其中一个"
    echo "  - 两个地址都可以接收入站连接"
    echo ""
    echo "建议:"
    echo "  1. 如果需要固定使用 Token 地址，可以手动删除 DHCPv6 地址:"
    echo "     ip -6 addr del $DHCPV6_ADDR/128 dev $INTERFACE"
    echo ""
    echo "  2. 或使用应用层绑定到特定地址:"
    echo "     curl --interface $TOKEN_ADDR https://example.com"
elif [ "$HAS_TOKEN" = true ]; then
    echo "✅ 系统仅使用 Token 地址，配置完美！"
    echo ""
    echo "使用的地址: $TOKEN_ADDR"
    echo "所有连接都会使用此地址"
elif [ "$HAS_DHCPV6" = true ]; then
    echo "⚠️  系统仅使用 DHCPv6 地址"
    echo ""
    echo "使用的地址: $DHCPV6_ADDR"
    echo "Token 未生效"
fi

echo ""

# 5. Token 状态
echo "【Token 配置状态】"
echo "----------------------------------------"
if [ "$TOKEN_NM" = "--" ]; then
    echo "NetworkManager: ❌ 未配置"
else
    echo "NetworkManager: ✅ $TOKEN_NM"
fi

TOKEN_SYS=$(ip token show dev $INTERFACE 2>/dev/null)
if [ -n "$TOKEN_SYS" ]; then
    echo "系统内核: ✅ $TOKEN_SYS"
else
    echo "系统内核: ⚠️  未检测到"
fi

echo ""

# 6. MAC 地址
echo "【MAC 地址】"
echo "----------------------------------------"
ip link show $INTERFACE | grep "link/ether"

echo ""

# 7. 路由通告
echo "【路由通告检查】"
echo "----------------------------------------"
if command -v rdisc6 &> /dev/null; then
    echo "检查中（5秒）..."
    timeout 5 rdisc6 $INTERFACE 2>&1 | head -15 || echo "未收到 RA"
else
    echo "rdisc6 未安装"
fi

echo ""

# 8. 综合判断
echo "【综合判断】"
echo "========================================"

if [ "$TOKEN_NM" = "--" ]; then
    echo "❌ Token 未配置"
    echo ""
    echo "启用 Token:"
    echo "  bash enable-token-888.sh"
elif [ "$HAS_TOKEN" = true ]; then
    echo "✅ Token 配置成功！"
    echo ""
    echo "Token 地址: $TOKEN_ADDR/64"
    echo ""
  
    if [ "$HAS_DHCPV6" = true ]; then
        echo "📌 当前状态: Token 和 DHCPv6 并存"
        echo ""
        echo "DHCPv6 地址: $DHCPV6_ADDR/128"
        echo ""
        echo "这是正常现象，因为路由器设置了 M flag=1（强制DHCPv6）"
        echo ""
        echo "两个地址都可以使用，系统会自动选择："
        echo "  - 出站连接：可能优先使用 DHCPv6 地址"
        echo "  - 入站连接：两个地址都可以接收"
        echo ""
        echo "如需仅使用 Token 地址，可以："
        echo "  1. 手动删除 DHCPv6: ip -6 addr del $DHCPV6_ADDR/128 dev $INTERFACE"
        echo "  2. 应用层指定地址: 在应用程序中绑定到 $TOKEN_ADDR"
    else
        echo "🎉 完美！系统仅使用 Token 地址"
    fi
else
    echo "⚠️  Token 已配置但未生效"
    echo ""
    echo "可能原因:"
    echo "  1. 路由器未发送 RA 或 A flag=0"
    echo "  2. 网络连接未重启"
    echo "  3. Token 配置未正确应用"
    echo ""
    echo "解决方案:"
    echo "  bash enable-token-888.sh"
fi

echo "========================================"
