# 多阶段构建 Dockerfile
# 阶段 1: 构建
# 使用 Rust 1.93 匹配本地环境
FROM rust:1.93-bookworm as builder

WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 复制源代码
COPY src ./src
COPY frontend ./frontend

# 编译发布版本
RUN cargo build --release

# 阶段 2: 运行时
# 使用 Debian 12 (Bookworm) 匹配宿主机环境 (GLIBC 2.36)
FROM debian:12-slim

# 添加标签
LABEL org.opencontainers.image.title="IPv6 Token WebUI"
LABEL org.opencontainers.image.description="Web UI for managing IPv6 Token configuration on Linux NetworkManager"
LABEL org.opencontainers.image.authors="clearlove.ymg"
LABEL org.opencontainers.image.source="https://github.com/MG5921MY/token64-eui-webui-public"
LABEL org.opencontainers.image.licenses="MIT"

# 安装必要的系统工具
RUN apt-get update && apt-get install -y \
    iproute2 \
    network-manager \
    && rm -rf /var/lib/apt/lists/*

# 从构建阶段复制二进制文件（已包含嵌入的前端资源）
COPY --from=builder /app/target/release/ipv6-token-webui /usr/local/bin/

# 创建配置目录
RUN mkdir -p /app/config && chmod 755 /app/config

# 设置工作目录
WORKDIR /app

# 设置权限
RUN chmod +x /usr/local/bin/ipv6-token-webui

# 暴露端口
EXPOSE 5000

# 启动命令
# 使用 ENTRYPOINT 允许 docker-compose 传递参数
ENTRYPOINT ["ipv6-token-webui"]

# 默认参数: 监听 0.0.0.0（Docker 容器中需要）
CMD ["-H", "0.0.0.0"]
