# Docker 部署说明

本项目提供三种 Docker Compose 配置文件，适用于不同的部署场景。

---

## ⚠️ 重要：监听地址配置

### 默认配置

- **二进制运行**: 默认监听 `127.0.0.1:5000`（仅本地访问）
- **Docker 容器**: 默认监听 `0.0.0.0:5000`（容器内），通过端口映射访问

### 安全建议

1. **本地开发测试**: 使用默认配置即可
2. **内网部署**: 配合防火墙使用
3. **公网部署**: 使用反向代理（Nginx + HTTPS）

**详细配置说明**: 查看 [网络监听地址配置说明.md](网络监听地址配置说明.md)

---

## 📁 配置文件说明

### 1. `docker-compose.yml` - 默认配置（本地构建）

**用途**: 默认配置，适合开发和测试

**特点**:
- 从本地 Dockerfile 构建镜像
- 适合需要修改代码的场景

**使用方法**:
```bash
cd code
docker compose up -d
```

---

### 2. `docker-compose.local.yml` - 本地构建版本

**用途**: 明确指定本地构建，适合开发环境

**特点**:
- 从本地 Dockerfile 构建镜像
- 可以自定义修改代码
- 构建时间较长（首次约 5-10 分钟）

**使用方法**:
```bash
cd code
docker compose -f docker-compose.local.yml up -d

# 查看日志
docker compose -f docker-compose.local.yml logs -f

# 停止
docker compose -f docker-compose.local.yml down
```

**适用场景**:
- ✅ 开发和调试
- ✅ 需要修改源代码
- ✅ 测试新功能
- ✅ 离线环境（无法访问 GitHub）

---

### 3. `docker-compose.remote.yml` - 远程镜像版本（推荐）

**用途**: 从 GitHub Container Registry 拉取预构建镜像

**特点**:
- 直接拉取已构建好的镜像
- 启动速度快（约 10-30 秒）
- 无需本地编译环境
- 镜像由 GitHub Actions 自动构建
- 配置文件通过卷挂载持久化

**使用方法**:
```bash
cd code
docker compose -f docker-compose.remote.yml up -d

# 首次访问需要设置密码
# 浏览器打开 http://localhost:5000

# 查看日志
docker compose -f docker-compose.remote.yml logs -f

# 停止
docker compose -f docker-compose.remote.yml down

# 更新到最新版本
docker compose -f docker-compose.remote.yml pull
docker compose -f docker-compose.remote.yml up -d
```

**适用场景**:
- ✅ 生产环境部署（推荐）
- ✅ 快速部署
- ✅ 不需要修改代码
- ✅ 自动获取最新版本
- ✅ 配置持久化（密码不丢失）

---

## 🚀 快速开始

### 方式一：使用远程镜像（推荐，最快）

```bash
cd code
docker compose -f docker-compose.remote.yml up -d
```

访问: http://localhost:5000

**首次访问**: 系统会要求设置初始密码（至少 6 位）

### 方式二：本地构建

```bash
cd code
docker compose up -d
```

首次构建需要 5-10 分钟，后续启动很快。

**首次访问**: 系统会要求设置初始密码（至少 6 位）

---

## 🔐 认证配置

### 首次启动

1. **启动容器**
   ```bash
   docker compose up -d
   ```

2. **访问 Web 界面**
   - 浏览器打开 `http://localhost:5000`
   - 系统检测到未设置密码，显示"首次设置"页面

3. **设置初始密码**
   - 输入密码（至少 6 位）
   - 确认密码
   - 系统自动登录

### 配置持久化

所有 Docker Compose 文件已配置卷挂载：

```yaml
volumes:
  - ./config:/app/config
```

这确保：
- ✅ 配置文件持久化（`config/config.yaml`）
- ✅ 重启容器不丢失密码
- ✅ 可以在宿主机直接编辑配置

### 修改密码

**方法 1: 使用 CLI（推荐）**
```bash
docker exec -it ipv6-token-webui ipv6-token-webui --set-password
```

**方法 2: 重置密码**
```bash
# 停止容器
docker compose down

# 删除配置文件
rm config/config.yaml

# 重启容器
docker compose up -d

# 访问 Web 界面重新设置密码
```

### 认证配置文件

配置文件位置: `./config/config.yaml`

```yaml
auth:
  password_hash: "$argon2id$v=19$m=19456,t=2,p=1$..."
  session_timeout_minutes: 30
  max_login_attempts: 5
  lockout_duration_minutes: 15
```

**详细认证配置**: 查看 [认证配置指南.md](认证配置指南.md)

---

## 📊 对比

| 特性 | 本地构建 | 远程镜像 |
|------|---------|---------|
| 启动速度 | 慢（首次 5-10 分钟） | 快（10-30 秒） |
| 需要编译环境 | ✅ 需要 | ❌ 不需要 |
| 可修改代码 | ✅ 可以 | ❌ 不可以 |
| 网络要求 | 低 | 需要访问 GitHub |
| 适用场景 | 开发测试 | 生产环境 |
| 推荐度 | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## 🔧 常用命令

### 启动服务

```bash
# 使用默认配置
docker compose up -d

# 使用本地构建
docker compose -f docker-compose.local.yml up -d

# 使用远程镜像
docker compose -f docker-compose.remote.yml up -d
```

### 查看日志

```bash
# 实时查看日志
docker compose logs -f

# 查看最近 100 行日志
docker compose logs --tail=100
```

### 停止服务

```bash
# 停止并删除容器
docker compose down

# 停止但保留容器
docker compose stop
```

### 重启服务

```bash
docker compose restart
```

### 更新镜像（远程版本）

```bash
# 拉取最新镜像
docker compose -f docker-compose.remote.yml pull

# 重新创建容器
docker compose -f docker-compose.remote.yml up -d
```

### 重新构建（本地版本）

```bash
# 强制重新构建
docker compose build --no-cache

# 重新创建容器
docker compose up -d
```

---

## 🐛 故障排查

### 问题 1: 无法拉取远程镜像

**错误**: `Error response from daemon: manifest unknown`

**原因**: 镜像还未构建或网络问题

**解决方案**:
```bash
# 方案 1: 使用本地构建
docker compose -f docker-compose.local.yml up -d

# 方案 2: 检查网络连接
ping ghcr.io

# 方案 3: 等待 GitHub Actions 构建完成
# 访问: https://github.com/MG5921MY/token64-eui-webui-public/actions
```

### 问题 2: 本地构建失败

**错误**: `failed to solve: process "/bin/sh -c cargo build --release" did not complete successfully`

**解决方案**:
```bash
# 清理 Docker 缓存
docker system prune -a

# 重新构建
docker compose build --no-cache
```

### 问题 3: 容器启动后无法访问

**检查步骤**:
```bash
# 1. 检查容器是否运行
docker ps | grep ipv6-token-webui

# 2. 查看容器日志
docker logs ipv6-token-webui

# 3. 检查端口占用
sudo netstat -tlnp | grep 5000

# 4. 测试本地访问
curl http://localhost:5000
```

### 问题 4: 忘记密码

**解决方案**:
```bash
# 方案 1: 使用 CLI 修改密码
docker exec -it ipv6-token-webui ipv6-token-webui --set-password

# 方案 2: 重置密码（删除配置文件）
docker compose down
rm config/config.yaml
docker compose up -d
# 访问 Web 界面重新设置密码
```

### 问题 5: 配置文件丢失

**原因**: 未正确挂载卷或删除了 config 目录

**解决方案**:
```bash
# 检查卷挂载
docker inspect ipv6-token-webui | grep Mounts -A 10

# 确保 docker-compose.yml 中有卷配置
# volumes:
#   - ./config:/app/config

# 重新创建容器
docker compose down
docker compose up -d
```

---

## 📝 配置说明

### 网络模式

所有配置都使用 `network_mode: host`，这样容器可以直接访问主机的 NetworkManager。

### 权限

使用 `cap_add: NET_ADMIN` 而不是 `privileged: true`，提供最小必要权限。

### 环境变量

可以通过环境变量自定义配置：

```yaml
environment:
  - LOG_LEVEL=debug  # 日志级别: trace, debug, info, warn, error
  - SERVER_PORT=8080  # 自定义端口
  - SERVER_HOST=127.0.0.1  # 自定义监听地址
```

---

## 🎯 推荐部署方案

### 开发环境

```bash
# 使用本地构建，方便调试
docker compose -f docker-compose.local.yml up -d
```

### 生产环境

```bash
# 使用远程镜像，快速稳定
docker compose -f docker-compose.remote.yml up -d
```

### 测试环境

```bash
# 使用默认配置
docker compose up -d
```

---

## 📚 相关文档

- [项目说明.md](项目说明.md) - 项目概述
- [认证配置指南.md](认证配置指南.md) - 认证系统详细配置
- [Linux部署指南.md](Linux部署指南.md) - Linux 完整部署指南
- [部署指南.md](部署指南.md) - 多种部署方式
- [网络监听地址配置说明.md](网络监听地址配置说明.md) - 网络配置说明

---

**最后更新**: 2026-01-26
