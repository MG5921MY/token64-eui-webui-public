# IPv6 Token 配置脚本

这些脚本用于在 Linux NetworkManager 系统上配置 IPv6 Token 模式。

---

## 📋 脚本说明

| 脚本 | 功能 | 说明 |
|------|------|------|
| `enable-token-888.sh` | 启用 Token ::888 | 配置固定 IPv6 后缀 |
| `disable-token.sh` | 禁用 Token | 恢复默认配置 |
| `check-token.sh` | 检查状态 | 查看当前配置和地址 |

---

## 🚀 快速开始

### 1. 修改配置参数

**在使用前，请先修改脚本中的参数：**

```bash
# 编辑 enable-token-888.sh
nano enable-token-888.sh

# 修改以下参数：
CONNECTION_NAME="Wired connection 1"  # 你的连接名称
INTERFACE="eth0"                      # 你的网卡名称
TOKEN="::888"                         # 你想要的 Token 后缀
PRIVACY_MODE=2                        # 0=禁用临时IPv6, 2=启用临时IPv6
```

**如何查找你的参数：**

```bash
# 查看连接名称
nmcli connection show

# 查看网卡名称
ip link show

# 示例输出：
# NAME                UUID                                  TYPE      DEVICE
# Wired connection 1  5fb06bd0-0bb0-7ffb-45f1-d6edd65f3e03  ethernet  eth0
#  ↑ 连接名称                                                          ↑ 网卡名称
```

### 2. 添加执行权限

```bash
chmod +x enable-token-888.sh
chmod +x disable-token.sh
chmod +x check-token.sh
```

### 3. 启用 Token

```bash
# 需要 root 权限
sudo ./enable-token-888.sh
```

**预期输出：**
```
=== 启用 IPv6 Token ::888 + 临时地址优先出站 ===

配置前地址:
无

1. 配置 IPv6 方法...
2. 禁用 DHCPv6（关键）...
3. 设置地址生成模式（必须在 Token 之前）...
4. 设置 Token...
5. 设置隐私模式 (2)...
   启用临时IPv6地址并优先出站
6. 清除旧地址...
7. 重启连接...

等待10秒...

=== 配置完成 ===

新地址:
inet6 2001:db8:1234:5678::888/64 scope global

Token状态:
ipv6.token:                             ::888

✅ Token ::888 已生效！
```

### 4. 检查状态

```bash
sudo ./check-token.sh
```

### 5. 禁用 Token（可选）

```bash
sudo ./disable-token.sh
```

---

## 📖 详细说明

### Token 模式

**什么是 Token？**
- Token 是自定义的 IPv6 地址后缀
- 例如：`::888`、`::1`、`::abc`

**优点：**
- ✅ 自定义后缀，易记
- ✅ 不暴露 MAC 地址
- ✅ 地址固定不变
- ✅ 隐私性好

**示例：**
```
路由器前缀: 2001:db8:1234:5678::/64
自定义Token: ::888
最终地址:   2001:db8:1234:5678::888/64
```

### 隐私模式

**PRIVACY_MODE 参数说明：**

| 值 | 模式 | 说明 |
|----|------|------|
| 0 | 禁用临时地址 | 仅使用 Token 地址，可能被跟踪 |
| 2 | 启用临时地址 | Token 地址用于入站，临时地址用于出站 |

**推荐：** 使用 `PRIVACY_MODE=2`（方案二）

**原因：**
- Token 地址固定，方便远程访问（入站）
- 临时地址随机，保护隐私（出站）
- 避免被跟踪

---

## 🔍 常见问题

### Q1: 如何查看当前 IPv6 地址？

```bash
ip -6 addr show eth0 | grep "scope global"
```

### Q2: Token 配置了但未生效？

**可能原因：**
1. 路由器未启用 SLAAC（A flag=0）
2. 参数配置错误
3. 网络连接未重启

**解决方案：**
```bash
# 1. 检查路由器配置
sudo apt install -y ndisc6
rdisc6 eth0

# 2. 重新运行脚本
sudo ./enable-token-888.sh

# 3. 检查状态
sudo ./check-token.sh
```

### Q3: 重启后配置丢失？

**不会丢失！** NetworkManager 配置是持久化的。

**验证：**
```bash
# 重启后检查
sudo reboot

# 重新登录后查看
nmcli connection show "Wired connection 1" | grep ipv6
ip -6 addr show eth0 | grep "scope global"
```

### Q4: 如何更换 Token 后缀？

**方法 1：修改脚本**
```bash
nano enable-token-888.sh
# 修改 TOKEN="::888" 为你想要的后缀
sudo ./enable-token-888.sh
```

**方法 2：直接命令**
```bash
sudo nmcli connection modify "Wired connection 1" ipv6.token "::新后缀"
sudo nmcli connection down "Wired connection 1"
sudo nmcli connection up "Wired connection 1"
```

### Q5: 同时有 DHCPv6 和 Token 地址？

**这是正常的！** 如果路由器启用了 DHCPv6（M flag=1）。

**查看：**
```bash
ip -6 addr show eth0 | grep "scope global"

# 可能输出：
# inet6 2001:db8::a10/128 scope global    # DHCPv6 地址
# inet6 2001:db8::888/64 scope global     # Token 地址
```

**说明：**
- DHCPv6 地址（/128）：系统可能优先用于出站
- Token 地址（/64）：用于入站连接
- 两个地址都可以使用

**如需仅使用 Token：**
```bash
# 删除 DHCPv6 地址
sudo ip -6 addr del 2001:db8::a10/128 dev eth0
```

---

## 📝 脚本参数说明

### enable-token-888.sh

```bash
CONNECTION_NAME="Wired connection 1"  # NetworkManager 连接名称
INTERFACE="eth0"                      # 网卡接口名称
TOKEN="::888"                         # Token 后缀（可自定义）
PRIVACY_MODE=2                        # 隐私模式（0 或 2）
```

### disable-token.sh

```bash
CONNECTION_NAME="Wired connection 1"  # NetworkManager 连接名称
INTERFACE="eth0"                      # 网卡接口名称
```

### check-token.sh

```bash
CONNECTION_NAME="Wired connection 1"  # NetworkManager 连接名称
INTERFACE="eth0"                      # 网卡接口名称
```

---

## 🎯 使用场景

### 场景 1: 家庭 NAS 服务器

```bash
# 配置固定 IPv6 地址
TOKEN="::888"
PRIVACY_MODE=2

# 启用 Token
sudo ./enable-token-888.sh

# 在路由器上配置端口转发
# 目标地址: 2001:db8:1234:5678::888
# 端口: 80, 443, 22 等
```

### 场景 2: 云服务器

```bash
# 配置简洁的 Token
TOKEN="::1"
PRIVACY_MODE=0  # 服务器通常不需要临时地址

# 启用 Token
sudo ./enable-token-888.sh
```

### 场景 3: 开发测试

```bash
# 使用易记的 Token
TOKEN="::dev"
PRIVACY_MODE=2

# 启用 Token
sudo ./enable-token-888.sh

# 测试访问
curl -6 http://[2001:db8:1234:5678::dev]:8080
```

---

## 🔗 相关链接

- [项目主页](https://github.com/MG5921MY/token64-eui-webui-public)
- [WebUI 版本](../README.md)
- [Docker 部署](../code/Docker部署说明.md)

---

## 📄 许可证

MIT License - Copyright (c) 2026 clearlove.ymg

---

**创建时间**: 2026-01-26  
**适用系统**: Linux + NetworkManager（Debian 12, Ubuntu 20.04+, CentOS 8+）
