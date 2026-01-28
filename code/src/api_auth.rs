/*!
 * 认证 API 模块
 * 
 * 提供登录、登出和状态检查接口
 */

use actix_web::{web, HttpRequest, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use log::{info, warn};
use std::sync::Arc;

use crate::auth::AuthManager;
use crate::config::AppConfig;
use crate::models::ApiResponse;

/// 初始密码设置请求
#[derive(Debug, Deserialize)]
pub struct SetupPasswordRequest {
    pub password: String,
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub password: String,
}

/// 登录响应数据
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub session_id: String,
    pub expires_in: u64,  // 秒
}

/// 认证状态响应
#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub authenticated: bool,
    pub needs_setup: bool,
}

/// 提取客户端 IP 地址
fn get_client_ip(req: &HttpRequest) -> String {
    // 尝试从 X-Forwarded-For 获取真实 IP
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    
    // 尝试从 X-Real-IP 获取
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // 使用连接信息
    req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// POST /api/auth/setup - 初始密码设置
pub async fn setup_password(
    req: web::Json<SetupPasswordRequest>,
    auth_manager: web::Data<Arc<AuthManager>>,
    app_config: web::Data<AppConfig>,
    http_req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let ip = get_client_ip(&http_req);
    
    // 检查是否需要设置密码
    if !auth_manager.needs_password_setup() {
        warn!("尝试重复设置密码 (IP: {})", ip);
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error("密码已设置，请使用登录接口".to_string())
        ));
    }
    
    info!("初始密码设置请求 (IP: {})", ip);
    
    // 设置密码
    match auth_manager.setup_password(&req.password) {
        Ok(hash) => {
            info!("初始密码设置成功 (IP: {})", ip);
            
            // 保存密码哈希到配置文件
            let mut config = (*app_config.get_ref()).clone();
            if let Err(e) = config.update_password_hash(hash.clone()) {
                warn!("保存密码到配置文件失败: {} (IP: {})", e, ip);
                return Ok(HttpResponse::InternalServerError().json(
                    ApiResponse::<()>::error(format!("密码设置成功但保存失败: {}", e))
                ));
            }
            
            // 更新认证管理器中的密码哈希
            auth_manager.update_password_hash(hash);
            
            info!("密码已保存到配置文件 (IP: {})", ip);
            
            // 自动创建会话
            let session_id = auth_manager.create_session(&ip)
                .map_err(|e| {
                    actix_web::error::ErrorInternalServerError(e)
                })?;
            
            let response = LoginResponse {
                session_id,
                expires_in: auth_manager.config.session_timeout_minutes * 60,
            };
            
            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Err(e) => {
            warn!("初始密码设置失败: {} (IP: {})", e, ip);
            Ok(HttpResponse::BadRequest().json(
                ApiResponse::<()>::error(e)
            ))
        }
    }
}

/// POST /api/auth/login - 用户登录
pub async fn login(
    req: web::Json<LoginRequest>,
    auth_manager: web::Data<Arc<AuthManager>>,
    http_req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let ip = get_client_ip(&http_req);
    
    // 检查是否需要初始化密码
    if auth_manager.needs_password_setup() {
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error("请先设置初始密码".to_string())
        ));
    }
    
    // 检查 IP 是否被锁定
    if auth_manager.is_ip_locked(&ip) {
        warn!("IP {} 已被锁定，拒绝登录", ip);
        return Ok(HttpResponse::TooManyRequests().json(
            ApiResponse::<()>::error(format!(
                "登录失败次数过多，请在 {} 分钟后重试",
                auth_manager.config.lockout_duration_minutes
            ))
        ));
    }
    
    info!("登录请求 (IP: {})", ip);
    
    // 验证密码
    match auth_manager.verify_password(&req.password) {
        Ok(true) => {
            // 密码正确，重置失败计数
            auth_manager.reset_failed_login(&ip);
            
            // 创建会话
            let session_id = auth_manager.create_session(&ip)
                .map_err(|e| {
                    actix_web::error::ErrorInternalServerError(e)
                })?;
            
            info!("登录成功 (IP: {})", ip);
            
            let response = LoginResponse {
                session_id,
                expires_in: auth_manager.config.session_timeout_minutes * 60,
            };
            
            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Ok(false) => {
            // 密码错误，记录失败
            auth_manager.record_failed_login(&ip);
            let remaining = auth_manager.get_remaining_attempts(&ip);
            
            warn!("登录失败：密码错误 (IP: {}, 剩余尝试: {})", ip, remaining);
            
            Ok(HttpResponse::Unauthorized().json(
                ApiResponse::<()>::error(format!(
                    "密码错误，剩余尝试次数: {}",
                    remaining
                ))
            ))
        }
        Err(e) => {
            warn!("登录失败：{} (IP: {})", e, ip);
            Ok(HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("登录失败，请稍后重试".to_string())
            ))
        }
    }
}

/// POST /api/auth/logout - 用户登出
pub async fn logout(
    http_req: HttpRequest,
    auth_manager: web::Data<Arc<AuthManager>>,
) -> Result<HttpResponse, Error> {
    // 从 Authorization header 提取会话 ID
    if let Some(auth_header) = http_req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(session_id) = auth_str.strip_prefix("Bearer ") {
                auth_manager.delete_session(session_id);
                info!("用户登出: {}", session_id);
            }
        }
    }
    
    Ok(HttpResponse::Ok().json(ApiResponse::success("登出成功")))
}

/// GET /api/auth/status - 检查认证状态
pub async fn check_auth_status(
    http_req: HttpRequest,
    auth_manager: web::Data<Arc<AuthManager>>,
) -> Result<HttpResponse, Error> {
    let needs_setup = auth_manager.needs_password_setup();
    
    // 如果需要设置密码，直接返回
    if needs_setup {
        let response = AuthStatusResponse {
            authenticated: false,
            needs_setup: true,
        };
        return Ok(HttpResponse::Ok().json(ApiResponse::success(response)));
    }
    
    // 检查会话是否有效
    let authenticated = if let Some(auth_header) = http_req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(session_id) = auth_str.strip_prefix("Bearer ") {
                auth_manager.validate_session(session_id).unwrap_or(false)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    
    let response = AuthStatusResponse {
        authenticated,
        needs_setup: false,
    };
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
}
