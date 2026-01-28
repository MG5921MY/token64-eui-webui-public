# IPv6 Token WebUI - CLI 使用指南

## 📋 命令行参数

程序支持以下命令行参数来自定义运行配置：

### 基本用法

```bash
ipv6-token-webui [选项]
```

### 可用参数

| 参数 | 短参数 | 默认值 | 说明 |
|------|--------|--------|------|
| `--port` | `-p` | `5000` | 服务器监听端口 |
| `--host` | `-H` | `0.0.0.0` | 服务器监听地址 |
| `--log-level` | `-l` | `info` | 日志级别 (trace, debug, info, warn, error) |
| `--help` | `-h` | - | 显示帮助信息 |
| `--version` | `-V` | - | 显示版本信息 |

## 🚀 使用示例

### 1. 使用默认配置启动

```bash
./ipv6-token-webui
```

服务器将在 `http://0.0.0.0:5000` 启动

### 2. 指定端口

```bash
# 使用 8080 端口
./ipv6-token-webui --port 8080

# 或使用短参数
./ipv6-token-webui -p 8080
```

访问地址: `http://localhost:8080`

### 3. 指定监听地址

```bash
# 只监听本地回环地址（更安全）
./ipv6-token-webui --host 127.0.0.1

# 监听特定 IP
./ipv6-token-webui --host 192.168.1.100
```

### 4. 组合多个参数

```bash
# 自定义端口和地址
./ipv6-token-webui --host 127.0.0.1 --port 3000

# 使用短参数
./ipv6-token-webui -H 127.0.0.1 -p 3000
```

### 5. 调整日志级别

```bash
# 详细调试日志
./ipv6-token-webui --log-level debug

# 只显示警告和错误
./ipv6-token-webui --log-level warn

# 最详细的跟踪日志
./ipv6-token-webui -l trace
```

### 6. 查看帮助信息

```bash
./ipv6-token-webui --help
```

### 7. 查看版本信息

```bash
./ipv6-token-webui --version
```

## 🔧 高级配置

### 使用环境变量

除了命令行参数，还可以使用环境变量配置：

```bash
# Linux
export SERVER_HOST=127.0.0.1
export SERVER_PORT=8080
export LOG_LEVEL=debug
./ipv6-token-webui
```

**注意**: 命令行参数的优先级高于环境变量

### 配置优先级

配置加载优先级（从高到低）：
1. 命令行参数
2. 环境变量
3. 配置文件 (`config.toml`)
4. 默认值

## 🐳 Docker 使用

### 使用自定义端口

```bash
docker run -d \
  --name ipv6-token-webui \
  --network host \
  --cap-add=NET_ADMIN \
  ipv6-token-webui \
  --port 8080
```

### 使用环境变量

```bash
docker run -d \
  --name ipv6-token-webui \
  --network host \
  --cap-add=NET_ADMIN \
  -e SERVER_PORT=8080 \
  -e LOG_LEVEL=debug \
  ipv6-token-webui
```

### Docker Compose

```yaml
version: '3.8'

services:
  ipv6-token-webui:
    image: ipv6-token-webui:latest
    network_mode: host
    cap_add:
      - NET_ADMIN
    command: ["--port", "8080", "--log-level", "debug"]
    restart: unless-stopped
```

## 📊 日志级别说明

| 级别 | 说明 | 适用场景 |
|------|------|----------|
| `trace` | 最详细的日志，包含所有执行细节 | 深度调试 |
| `debug` | 调试信息，包含详细的操作步骤 | 开发和故障排查 |
| `info` | 一般信息，显示重要操作 | 生产环境（默认） |
| `warn` | 警告信息，可能的问题 | 生产环境（精简） |
| `error` | 错误信息，只显示错误 | 生产环境（最精简） |

## 🔒 安全建议

### 1. 限制监听地址

在生产环境中，建议只监听本地回环地址：

```bash
./ipv6-token-webui --host 127.0.0.1
```

然后使用 nginx 或其他反向代理提供外部访问。

### 2. 使用非特权端口

避免使用 1-1024 的特权端口：

```bash
# 推荐使用 1024 以上的端口
./ipv6-token-webui --port 8080
```

### 3. 配合防火墙

```bash
# 只允许本地访问
sudo ufw allow from 127.0.0.1 to any port 5000

# 允许特定 IP 访问
sudo ufw allow from 192.168.1.0/24 to any port 5000
```

## 🛠️ Systemd 服务配置

创建服务文件 `/etc/systemd/system/ipv6-token-webui.service`:

```ini
[Unit]
Description=IPv6 Token WebUI
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/ipv6-token-webui
ExecStart=/opt/ipv6-token-webui/ipv6-token-webui --port 5000 --host 127.0.0.1 --log-level info
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

启用并启动服务：

```bash
sudo systemctl daemon-reload
sudo systemctl enable ipv6-token-webui
sudo systemctl start ipv6-token-webui
sudo systemctl status ipv6-token-webui
```

查看日志：

```bash
sudo journalctl -u ipv6-token-webui -f
```

## 📝 常见问题

### Q: 如何更改默认端口？

A: 使用 `--port` 参数：
```bash
./ipv6-token-webui --port 8080
```

### Q: 如何只允许本地访问？

A: 使用 `--host 127.0.0.1`：
```bash
./ipv6-token-webui --host 127.0.0.1
```

### Q: 如何查看详细的调试信息？

A: 使用 `--log-level debug`：
```bash
./ipv6-token-webui --log-level debug
```

### Q: 端口被占用怎么办？

A: 更换端口或停止占用该端口的程序：
```bash
# 查看端口占用
sudo netstat -tlnp | grep 5000

# 使用其他端口
./ipv6-token-webui --port 5001
```

### Q: 如何在后台运行？

A: 使用 `nohup` 或 systemd：
```bash
# 使用 nohup
nohup ./ipv6-token-webui > /var/log/ipv6-webui.log 2>&1 &

# 或使用 systemd（推荐）
sudo systemctl start ipv6-token-webui
```

## 🔗 相关文档

- [项目说明.md](项目说明.md) - 项目概述
- [Linux部署指南.md](Linux部署指南.md) - Linux 完整部署指南
- [部署指南.md](部署指南.md) - 多种部署方式

---

**最后更新**: 2026-01-26
