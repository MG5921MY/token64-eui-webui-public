/*!
 * IPv6 Token WebUI - 主程序入口
 * 
 * 这是一个基于 Rust + Actix-Web 的 Web 应用，用于管理 IPv6 Token 配置
 * 
 * 功能：
 * - 提供 RESTful API 接口
 * - 静态文件服务（前端页面）
 * - 安全的 Shell 脚本执行
 * - 多层输入验证和安全防护
 */

mod api;
mod api_auth;
mod auth;
mod config;
mod crypto;
mod embedded;
mod executor;
mod middleware_auth;
mod models;
mod network;
mod token_ops;
mod validator;

use actix_web::{middleware, web, App, HttpServer, HttpResponse};
use actix_governor::{Governor, GovernorConfigBuilder};
use clap::Parser;
use env_logger::Env;
use log::{info, error};
use std::sync::Arc;
use std::io::{self, Write};

#[cfg(target_os = "linux")]
use nix::unistd::Uid;

use crate::config::AppConfig;
use crate::executor::ScriptExecutor;
use crate::embedded::Assets;
use crate::auth::AuthManager;
use crate::middleware_auth::AuthMiddleware;

/// IPv6 Token WebUI - 为 Linux NetworkManager 配置自定义固定 IPv6 后缀
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 服务器监听端口
    #[arg(short, long, default_value = "5000")]
    port: u16,
    
    /// 服务器监听地址
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,
    
    /// 日志级别 (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// 跳过 root 权限检查（仅用于测试）
    #[arg(long, default_value = "false")]
    skip_root_check: bool,
    
    /// 设置密码（交互式）
    #[arg(long)]
    set_password: bool,
}

/// 检查是否具有 root 权限
fn check_root_privileges() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        if !Uid::effective().is_root() {
            return Err(
                "此程序需要 root 权限才能修改 NetworkManager 配置\n\
                 请使用 sudo 运行：sudo ./ipv6-token-webui\n\
                 或者使用 --skip-root-check 跳过检查（仅用于测试）".to_string()
            );
        }
        Ok(())
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        Err(
            "此程序仅支持 Linux 系统\n\
             需要 NetworkManager 和 iproute2 工具".to_string()
        )
    }
}

/// 处理 --set-password 命令
async fn handle_set_password() -> std::io::Result<()> {
    println!("🔐 密码设置工具");
    println!();
    
    // 加载配置
    let mut app_config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("❌ 加载配置失败: {}", e);
            std::process::exit(1);
        }
    };
    
    // 提示输入新密码
    print!("请输入新密码（至少 6 位）: ");
    io::stdout().flush()?;
    
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();
    
    // 验证密码长度
    if password.len() < 6 {
        eprintln!("❌ 密码长度至少为 6 位");
        std::process::exit(1);
    }
    
    // 确认密码
    print!("请再次输入密码: ");
    io::stdout().flush()?;
    
    let mut password_confirm = String::new();
    io::stdin().read_line(&mut password_confirm)?;
    let password_confirm = password_confirm.trim();
    
    if password != password_confirm {
        eprintln!("❌ 两次输入的密码不一致");
        std::process::exit(1);
    }
    
    // 生成密码哈希
    println!("⏳ 生成密码哈希...");
    let hash = match crypto::hash_password(password) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("❌ 密码哈希生成失败: {}", e);
            std::process::exit(1);
        }
    };
    
    // 更新配置
    if let Err(e) = app_config.update_password_hash(hash) {
        eprintln!("❌ 更新配置失败: {}", e);
        std::process::exit(1);
    }
    
    println!("✅ 密码已更新！");
    println!("📝 配置文件: config/config.yaml");
    
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 解析命令行参数
    let args = Args::parse();
    
    // 处理 --set-password 命令
    if args.set_password {
        return handle_set_password().await;
    }
    
    // 初始化日志系统（使用 CLI 参数或环境变量）
    env_logger::Builder::from_env(
        Env::default().default_filter_or(&args.log_level)
    ).init();
    
    info!("🚀 IPv6 Token WebUI 启动中...");
    info!("📌 版本: {}", env!("CARGO_PKG_VERSION"));
    
    // 检查 root 权限
    if !args.skip_root_check {
        if let Err(e) = check_root_privileges() {
            error!("❌ 权限错误: {}", e);
            eprintln!("\n❌ 权限错误:\n{}\n", e);
            std::process::exit(1);
        }
        info!("✅ 权限检查通过（root）");
    } else {
        info!("⚠️  跳过 root 权限检查（仅用于测试）");
    }
    
    // 加载配置（CLI 参数优先级高于配置文件）
    let mut app_config = match AppConfig::load() {
        Ok(config) => {
            info!("✅ 配置文件加载成功");
            config
        }
        Err(e) => {
            info!("⚠️  配置文件加载失败，使用默认配置: {}", e);
            AppConfig::default()
        }
    };
    
    // CLI 参数覆盖配置文件
    app_config.server.host = args.host;
    app_config.server.port = args.port;
    
    // 创建认证管理器
    let auth_manager = Arc::new(AuthManager::new(app_config.auth.clone()));
    
    // 检查是否需要初始化密码
    if auth_manager.needs_password_setup() {
        info!("⚠️  首次启动：需要设置初始密码");
        info!("📝 请访问 Web 界面设置密码");
    }
    
    // 启动会话清理任务
    let auth_manager_cleanup = auth_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 分钟
        loop {
            interval.tick().await;
            auth_manager_cleanup.cleanup_expired().await;
        }
    });
    
    // 创建脚本执行器（保留用于向后兼容，但新 API 不使用）
    let executor = Arc::new(ScriptExecutor::new(app_config.script_dir()));
    
    // 配置速率限制：每秒 2 个请求，突发 10 个
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(2)
        .burst_size(10)
        .finish()
        .unwrap();
    
    // 服务器地址
    let bind_address = format!("{}:{}", app_config.server.host, app_config.server.port);
    info!("📡 服务器监听地址: {}", bind_address);
    info!("🌐 静态文件: 已嵌入到二进制文件");
    info!("⚙️  使用 Rust 原生实现（不依赖外部脚本）");
    info!("🔒 安全特性: 输入验证、速率限制、并发控制、密钥认证");
    
    if auth_manager.needs_password_setup() {
        info!("⚠️  首次启动：请访问 Web 界面设置初始密码");
    }
    
    info!("");
    info!("🎉 服务器启动成功！");
    info!("🔗 访问地址: http://{}:{}", 
          if app_config.server.host == "0.0.0.0" || app_config.server.host == "127.0.0.1" { 
              "localhost" 
          } else { 
              &app_config.server.host 
          },
          app_config.server.port);
    info!("");
    
    // 启动 HTTP 服务器
    HttpServer::new(move || {
        App::new()
            // 共享状态
            .app_data(web::Data::new(executor.clone()))
            .app_data(web::Data::new(app_config.clone()))
            .app_data(web::Data::new(auth_manager.clone()))
            
            // 中间件
            .wrap(Governor::new(&governor_conf))  // 速率限制
            .wrap(middleware::Logger::default())  // 请求日志
            .wrap(middleware::Compress::default()) // 响应压缩
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            )
            .wrap(AuthMiddleware::new(auth_manager.clone()))  // 认证中间件
            
            // 认证 API 路由（不需要认证）
            .service(
                web::scope("/api/auth")
                    .route("/setup", web::post().to(api_auth::setup_password))
                    .route("/login", web::post().to(api_auth::login))
                    .route("/logout", web::post().to(api_auth::logout))
                    .route("/status", web::get().to(api_auth::check_auth_status))
            )
            
            // 受保护的 API 路由（需要认证）
            .service(
                web::scope("/api")
                    .route("/system/info", web::get().to(api::get_system_info))
                    .route("/ipv6/enable-token", web::post().to(api::enable_token))
                    .route("/ipv6/disable-token", web::post().to(api::disable_token))
                    .route("/ipv6/check", web::get().to(api::check_status))
                    .route("/ipv6/prefix-info", web::get().to(api::get_prefix_info))
            )
            
            // 静态文件服务（从嵌入的资源提供）
            .route("/", web::get().to(serve_index))
            .default_service(web::to(serve_embedded_file))
    })
    .bind(&bind_address)?
    .run()
    .await?;
    
    info!("👋 服务器已停止");
    Ok(())
}

/// 提供首页
async fn serve_index() -> HttpResponse {
    serve_static_file("index.html")
}

/// 提供嵌入的静态文件（从路由路径提取）
async fn serve_embedded_file(path: web::Path<String>) -> HttpResponse {
    let path = path.into_inner();
    let path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };
    
    serve_static_file(path)
}

/// 内部函数：从嵌入资源中提供文件
fn serve_static_file(path: &str) -> HttpResponse {
    match Assets::get(path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            
            HttpResponse::Ok()
                .content_type(mime_type)
                .body(content.data.into_owned())
        }
        None => {
            // 如果文件不存在，返回 index.html（用于 SPA 路由）
            match Assets::get("index.html") {
                Some(content) => HttpResponse::Ok()
                    .content_type("text/html")
                    .body(content.data.into_owned()),
                None => HttpResponse::NotFound().body("404 Not Found"),
            }
        }
    }
}
