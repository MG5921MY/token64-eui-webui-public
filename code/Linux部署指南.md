# 🐧 IPv6 Token WebUI - Linux 部署指南

## 📋 系统要求

### 最低要求
- **操作系统**: Debian 12 / Ubuntu 20.04+ / CentOS 8+ / Arch Linux
- **架构**: x86_64 / ARM64 / ARM32
- **内存**: 最少 512MB RAM
- **磁盘**: 最少 50MB 可用空间

### 依赖软件
- NetworkManager (用于管理网络连接)
- iproute2 (提供 `ip` 命令)
- sudo (用于执行需要权限的命令)

## 🚀 快速开始

### 方法一：直接运行二进制文件（推荐）

#### 1. 编译二进制文件

```bash
# 安装 Rust (如果还没有)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 克隆仓库
git clone <仓库地址>
cd ipv6-token-webui/code

# 编译 Release 版本
cargo build --release

# 二进制文件位于
ls -lh target/release/ipv6-token-webui
```

#### 2. 运行程序

```bash
# 使用默认配置运行（端口 5000）
./ipv6-token-webui

# 或指定端口
./ipv6-token-webui --port 8080

# 或指定监听地址
./ipv6-token-webui --host 127.0.0.1 --port 8080

# 查看帮助
./ipv6-token-webui --help
```

#### 3. 访问 Web 界面

打开浏览器访问 `http://localhost:5000`

或从其他设备访问: `http://服务器IP:5000`

### 方法二：使用 systemd 服务（生产环境推荐）

#### 1. 安装二进制文件

```bash
# 创建安装目录
sudo mkdir -p /opt/ipv6-token-webui

# 复制二进制文件
sudo cp target/release/ipv6-token-webui /opt/ipv6-token-webui/

# 设置权限
sudo chmod +x /opt/ipv6-token-webui/ipv6-token-webui
```

#### 2. 创建 systemd 服务文件

```bash
sudo nano /etc/systemd/system/ipv6-token-webui.service
```

添加以下内容：

```ini
[Unit]
Description=IPv6 Token WebUI
Documentation=https://github.com/your-repo/ipv6-token-webui
After=network.target NetworkManager.service

[Service]
Type=simple
User=root
WorkingDirectory=/opt/ipv6-token-webui
ExecStart=/opt/ipv6-token-webui/ipv6-token-webui --port 5000 --host 0.0.0.0
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

#### 3. 启用并启动服务

```bash
# 重新加载 systemd 配置
sudo systemctl daemon-reload

# 启用服务（开机自启）
sudo systemctl enable ipv6-token-webui

# 启动服务
sudo systemctl start ipv6-token-webui

# 查看状态
sudo systemctl status ipv6-token-webui
```

#### 4. 管理服务

```bash
# 停止服务
sudo systemctl stop ipv6-token-webui

# 重启服务
sudo systemctl restart ipv6-token-webui

# 查看日志
sudo journalctl -u ipv6-token-webui -f

# 查看最近 100 行日志
sudo journalctl -u ipv6-token-webui -n 100

# 禁用服务
sudo systemctl disable ipv6-token-webui
```

### 方法三：使用 Docker

#### 1. 构建 Docker 镜像

```bash
cd code

# 构建镜像
docker build -t ipv6-token-webui:latest .
```

#### 2. 运行容器

```bash
# 使用 host 网络模式（推荐）
docker run -d \
  --name ipv6-token-webui \
  --network host \
  --cap-add=NET_ADMIN \
  --restart unless-stopped \
  ipv6-token-webui:latest
```

#### 3. 管理容器

```bash
# 查看日志
docker logs -f ipv6-token-webui

# 停止容器
docker stop ipv6-token-webui

# 启动容器
docker start ipv6-token-webui

# 重启容器
docker restart ipv6-token-webui

# 删除容器
docker rm -f ipv6-token-webui
```

### 方法四：使用 Docker Compose

#### 1. 启动服务

```bash
cd code

# 构建并启动
docker compose up -d

# 查看日志
docker compose logs -f

# 停止服务
docker compose down

# 重启服务
docker compose restart
```

## 🔧 配置选项

### 命令行参数

```bash
ipv6-token-webui [选项]

选项:
  -p, --port <PORT>            服务器监听端口 [默认: 5000]
  -H, --host <HOST>            服务器监听地址 [默认: 0.0.0.0]
  -l, --log-level <LOG_LEVEL>  日志级别 [默认: info]
  -h, --help                   显示帮助信息
  -V, --version                显示版本信息
```

### 环境变量

```bash
# 设置环境变量
export SERVER_HOST=127.0.0.1
export SERVER_PORT=8080
export LOG_LEVEL=debug

# 运行程序
./ipv6-token-webui
```

## 🔒 安全配置

### 1. 防火墙配置

#### UFW (Ubuntu/Debian)

```bash
# 允许本地访问
sudo ufw allow from 127.0.0.1 to any port 5000

# 允许局域网访问
sudo ufw allow from 192.168.0.0/16 to any port 5000

# 允许所有访问（不推荐）
sudo ufw allow 5000
```

#### firewalld (CentOS/RHEL)

```bash
# 允许端口
sudo firewall-cmd --permanent --add-port=5000/tcp
sudo firewall-cmd --reload

# 或限制来源
sudo firewall-cmd --permanent --add-rich-rule='rule family="ipv4" source address="192.168.0.0/16" port port="5000" protocol="tcp" accept'
sudo firewall-cmd --reload
```

### 2. 使用 Nginx 反向代理

#### 安装 Nginx

```bash
# Debian/Ubuntu
sudo apt install nginx

# CentOS/RHEL
sudo yum install nginx
```

#### 配置反向代理

```bash
sudo nano /etc/nginx/sites-available/ipv6-token-webui
```

添加配置：

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://127.0.0.1:5000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

启用配置：

```bash
# 创建符号链接
sudo ln -s /etc/nginx/sites-available/ipv6-token-webui /etc/nginx/sites-enabled/

# 测试配置
sudo nginx -t

# 重启 Nginx
sudo systemctl restart nginx
```

### 3. 使用 HTTPS (Let's Encrypt)

```bash
# 安装 Certbot
sudo apt install certbot python3-certbot-nginx

# 获取证书
sudo certbot --nginx -d your-domain.com

# 自动续期
sudo certbot renew --dry-run
```

## 📊 监控和日志

### 查看实时日志

```bash
# systemd 服务
sudo journalctl -u ipv6-token-webui -f

# Docker
docker logs -f ipv6-token-webui

# 直接运行（输出到终端）
./ipv6-token-webui --log-level debug
```

### 日志级别

- `trace` - 最详细（调试用）
- `debug` - 调试信息
- `info` - 一般信息（默认）
- `warn` - 警告信息
- `error` - 错误信息

## 🧪 测试

### 测试 API

```bash
# 获取系统信息
curl http://localhost:5000/api/system/info

# 启用 Token
curl -X POST http://localhost:5000/api/ipv6/enable-token \
  -H "Content-Type: application/json" \
  -d '{
    "connection_name": "Wired connection 1",
    "interface": "eth0",
    "token": "::888",
    "privacy_mode": 2
  }'

# 检查状态
curl "http://localhost:5000/api/ipv6/check?connection_name=Wired%20connection%201&interface=eth0"
```

## 🐛 故障排查

### 问题 1: 端口被占用

```bash
# 查看端口占用
sudo netstat -tlnp | grep 5000
# 或
sudo ss -tlnp | grep 5000

# 解决方案：使用其他端口
./ipv6-token-webui --port 5001
```

### 问题 2: 权限不足

```bash
# 错误: Permission denied

# 解决方案：使用 sudo 运行
sudo ./ipv6-token-webui
```

### 问题 3: NetworkManager 未运行

```bash
# 检查 NetworkManager 状态
sudo systemctl status NetworkManager

# 启动 NetworkManager
sudo systemctl start NetworkManager

# 启用开机自启
sudo systemctl enable NetworkManager
```

### 问题 4: 无法访问 Web 界面

```bash
# 1. 检查服务是否运行
sudo systemctl status ipv6-token-webui

# 2. 检查端口监听
sudo netstat -tlnp | grep 5000

# 3. 检查防火墙
sudo ufw status
sudo firewall-cmd --list-all

# 4. 测试本地访问
curl http://localhost:5000
```

## 📦 卸载

### 停止并删除服务

```bash
# 停止服务
sudo systemctl stop ipv6-token-webui

# 禁用服务
sudo systemctl disable ipv6-token-webui

# 删除服务文件
sudo rm /etc/systemd/system/ipv6-token-webui.service

# 重新加载 systemd
sudo systemctl daemon-reload

# 删除程序文件
sudo rm -rf /opt/ipv6-token-webui
```

### 删除 Docker 容器和镜像

```bash
# 停止并删除容器
docker stop ipv6-token-webui
docker rm ipv6-token-webui

# 删除镜像
docker rmi ipv6-token-webui:latest
```

## 🎯 最佳实践

1. **使用 systemd 服务** - 自动启动和管理
2. **配置防火墙** - 限制访问来源
3. **使用 Nginx 反向代理** - 提供 HTTPS 和负载均衡
4. **定期查看日志** - 监控运行状态
5. **备份配置** - 定期备份重要配置

## 📚 相关文档

- [项目说明.md](项目说明.md) - 项目概述
- [CLI使用指南.md](CLI使用指南.md) - CLI 详细说明
- [部署指南.md](部署指南.md) - 多种部署方式

## 🎉 完成！

现在你可以在 Linux 上运行 IPv6 Token WebUI 了！

如有问题，请查看故障排查部分或提交 Issue。

---

**最后更新**: 2026-01-26
