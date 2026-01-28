# IPv6 Token WebUI

🌐 基于 Rust 的 IPv6 Token 配置管理 Web 界面

## 📖 项目简介

为使用 NetworkManager 的 Linux 系统提供简单易用的 Web 界面，用于配置和管理 IPv6 Token（固定 IPv6 后缀），解决 IPv6 地址频繁变化导致的远程访问问题。

**适用系统**: Debian、Ubuntu、CentOS、Fedora、Arch Linux、飞牛NAS 等所有使用 NetworkManager 的 Linux 发行版。

## ✨ 核心特性

- ✅ **单一可执行文件** - 无需外部依赖，开箱即用
- ✅ **跨架构支持** - 支持 x86_64, ARM64, ARM32
- ✅ **密码认证保护** - 首次启动设置密码，会话管理，IP 锁定防护
- ✅ **多层安全防护** - 输入验证、命令注入防护、Argon2id 密码哈希
- ✅ **低资源占用** - 内存占用 < 10MB，启动时间 < 1秒
- ✅ **现代化界面** - 响应式设计，支持深色/浅色主题
- ✅ **实时日志** - 操作日志实时显示

## ⚠️ 重要说明

**本程序仅支持 Linux 系统运行**

- ✅ **支持**: Debian 12, Ubuntu 20.04+, CentOS 8+, Arch Linux 等
- ❌ **不支持**: Windows, macOS
- 📋 **依赖**: NetworkManager, iproute2, sysctl

## 🚀 快速开始

### 编译并运行

```bash
# 进入代码目录
cd code

# 编译发布版本
cargo build --release

# 运行程序
./target/release/ipv6-token-webui

# 访问 Web 界面
# 浏览器打开 http://localhost:5000
# 首次访问需要设置密码（至少 6 位）
```

**⚠️ 监听地址说明**:
- **默认**: `127.0.0.1:5000`（仅本地访问，最安全）
- **远程访问**: 使用 `-H 0.0.0.0` 参数（需配合防火墙）
- **详细配置**: 查看 [code/网络监听地址配置说明.md](code/网络监听地址配置说明.md)

### 🔐 认证系统

首次启动时，系统会要求设置初始密码：

1. **首次访问**: 浏览器打开 Web 界面，设置密码（至少 6 位）
2. **后续访问**: 使用密码登录
3. **修改密码**: 使用 CLI 命令 `./ipv6-token-webui --set-password`

**安全特性**:
- ✅ Argon2id 密码哈希（行业标准）
- ✅ 会话超时（默认 30 分钟，可配置）
- ✅ 登录失败追踪（5 次失败后锁定 15 分钟）
- ✅ 配置文件持久化（`config/config.yaml`）

**配置文件位置**: `config/config.yaml`（程序同目录）

```yaml
# 示例配置
auth:
  password_hash: "$argon2id$v=19$m=19456,t=2,p=1$..."
  session_timeout_minutes: 30
  max_login_attempts: 5
  lockout_duration_minutes: 15
```

### 使用 Docker

```bash
cd code
docker compose up -d

# 访问 http://localhost:5000
# 首次访问需要设置密码
```

**⚠️ Docker 监听地址**:
- **默认**: 容器内监听 `0.0.0.0`，宿主机通过端口映射访问
- **安全配置**: 修改 `docker-compose.local.yml` 中的 `ports` 配置
- **详细说明**: 查看 [code/网络监听地址配置说明.md](code/网络监听地址配置说明.md)

**⚠️ Docker 配置持久化**:
- 配置文件通过卷挂载持久化：`./config:/app/config`
- 重启容器不会丢失密码和配置
- 首次启动会自动创建 `config` 目录

## 📁 项目结构

```
.
├── code/                      # 主代码目录
│   ├── src/                   # Rust 源代码
│   ├── frontend/              # Web 前端
│   ├── 项目说明.md            # 项目详细说明
│   ├── CLI使用指南.md         # CLI 参数说明
│   ├── Linux部署指南.md       # Linux 部署指南
│   └── 部署指南.md            # 多种部署方式
│
├── docs/                      # 文档目录
│   ├── README.md              # 文档入口
│   ├── WebUI使用指南.md       # WebUI 使用指南
│   └── 安全说明.md            # 安全说明
│
├── sh/                        # 参考脚本
│   └── token-eui64/
│
└── README.md                  # 本文件
```

## 📖 文档导航

### 用户文档
- [项目说明.md](code/项目说明.md) - 项目详细说明和快速开始
- [CLI使用指南.md](code/CLI使用指南.md) - 命令行参数详细说明
- [Linux部署指南.md](code/Linux部署指南.md) - Linux 系统完整部署指南
- [部署指南.md](code/部署指南.md) - 多种部署方式说明

## 🔧 技术栈

- **后端**: Rust + Actix-Web 4.x
- **前端**: HTML5 + Tailwind CSS + Vanilla JavaScript
- **部署**: Docker / Systemd / 直接运行
- **安全**: 多层输入验证 + 命令注入防护

## 📊 性能指标

| 指标 | 数值 |
|------|------|
| 二进制大小 | ~4MB |
| 内存占用 | <10MB |
| 启动时间 | <1秒 |
| 响应时间 | <50ms |

## 🔒 安全特性

### 认证与会话管理
1. **密码保护** - Argon2id 哈希算法（行业标准）
2. **会话管理** - UUID v4 会话 ID，可配置超时时间
3. **登录保护** - 失败次数追踪，IP 自动锁定
4. **CLI 密码管理** - 安全的密码修改工具

### 输入验证与防护
1. **输入验证** - 正则表达式严格验证所有输入，长度限制，特殊字符过滤
2. **命令注入防护** - 使用参数数组而非 shell 字符串，脚本白名单
3. **路径遍历防护** - 路径规范化验证，防止 `../` 攻击
4. **权限管理** - Root 权限检查，平台验证
5. **并发控制** - 最多 5 个并发命令，防止资源耗尽
6. **速率限制** - 每秒 2 个请求，突发 10 个
7. **超时控制** - 60 秒超时自动终止
8. **日志审计** - 记录所有操作，错误信息不泄露

**详细安全说明**: 请查看 [docs/安全说明.md](docs/安全说明.md)

## 🎯 使用场景

- ✅ 任何使用 NetworkManager 的 Linux 系统
- ✅ 家庭 NAS 服务器（飞牛NAS、群晖、威联通等运行 Linux 的 NAS）
- ✅ 云服务器和 VPS
- ✅ 需要固定 IPv6 地址的场景
- ✅ 远程访问和端口转发

## 📝 API 端点

### 认证 API
- `GET /api/auth/status` - 检查认证状态
- `POST /api/auth/setup` - 首次密码设置
- `POST /api/auth/login` - 用户登录
- `POST /api/auth/logout` - 用户登出

### 系统 API（需要认证）
- `GET /api/system/info` - 获取系统信息
- `POST /api/ipv6/enable-token` - 启用 Token
- `POST /api/ipv6/disable-token` - 禁用 Token
- `GET /api/ipv6/check` - 检查状态

## 🧪 测试

```bash
cd code

# 运行所有测试
cargo test

# 运行特定模块测试
cargo test validator
```

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

Copyright (c) 2026 clearlove.ymg

## 🙏 致谢

- Rust 社区
- Actix-Web 框架
- NetworkManager 项目
- Linux 内核开发者

## 🔗 相关链接

- [GitHub 仓库](https://github.com/MG5921MY/token64-eui-webui-public)
- [Rust 官网](https://www.rust-lang.org/)
- [Actix-Web 文档](https://actix.rs/)
- [NetworkManager 文档](https://networkmanager.dev/)

---

**现在就开始使用吧！** 🚀

```bash
cd code
cargo build --release
./target/release/ipv6-token-webui
# 访问 http://localhost:5000
```
