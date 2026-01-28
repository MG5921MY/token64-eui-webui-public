/*!
 * API 处理函数模块
 * 
 * 提供所有 HTTP API 端点的处理逻辑
 * 使用 Rust 原生实现，不依赖外部脚本
 */

use actix_web::{web, HttpResponse, Error};
use log::{info, warn, error};

use crate::models::{ApiResponse, EnableTokenRequest, DisableTokenRequest, CheckStatusRequest};
use crate::network::NetworkManager;
use crate::token_ops::TokenOperator;
use crate::validator;

/// GET /api/system/info - 获取系统信息
/// 
/// 返回网络接口和 NetworkManager 连接列表
pub async fn get_system_info() -> Result<HttpResponse, Error> {
    info!("API: 获取系统信息");
    
    // 获取网络接口列表
    let interfaces = match NetworkManager::get_interfaces().await {
        Ok(ifaces) => ifaces,
        Err(e) => {
            warn!("获取接口列表失败: {}", e);
            vec![]
        }
    };
    
    // 获取 NetworkManager 连接列表
    let connections = match NetworkManager::get_connections().await {
        Ok(conns) => conns,
        Err(e) => {
            warn!("获取连接列表失败: {}", e);
            vec![]
        }
    };
    
    let system_info = serde_json::json!({
        "interfaces": interfaces,
        "connections": connections,
    });
    
    Ok(HttpResponse::Ok().json(ApiResponse::success(system_info)))
}

/// POST /api/ipv6/enable-token - 启用 IPv6 Token
/// 
/// # 请求体
/// ```json
/// {
///   "connection_name": "Wired connection 1",
///   "interface": "eth0",
///   "token": "::888",
///   "privacy_mode": 2
/// }
/// ```
pub async fn enable_token(
    req: web::Json<EnableTokenRequest>,
) -> Result<HttpResponse, Error> {
    info!("API: 启用 Token - {:?}", req);
    
    // 验证所有输入
    if let Err(e) = validator::validate_token(&req.token) {
        warn!("Token 验证失败: {}", e);
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error(e.to_string())
        ));
    }
    
    if let Err(e) = validator::validate_interface(&req.interface) {
        warn!("接口验证失败: {}", e);
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error(e.to_string())
        ));
    }
    
    if let Err(e) = validator::validate_connection_name(&req.connection_name) {
        warn!("连接名称验证失败: {}", e);
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error(e.to_string())
        ));
    }
    
    if let Err(e) = validator::validate_privacy_mode(req.privacy_mode as u8) {
        warn!("隐私模式验证失败: {}", e);
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error(e.to_string())
        ));
    }
    
    // 执行 Token 启用操作
    match TokenOperator::enable_token(
        &req.connection_name,
        &req.interface,
        &req.token,
        req.privacy_mode,
    ).await {
        Ok(result) => {
            if result.success {
                info!("Token 启用成功");
            } else {
                warn!("Token 启用失败: {}", result.message);
            }
            Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
        }
        Err(e) => {
            error!("Token 启用操作错误: {}", e);  // 详细日志
            Ok(HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("Token 启用失败，请检查日志".to_string())  // 通用消息
            ))
        }
    }
}

/// POST /api/ipv6/disable-token - 禁用 IPv6 Token
/// 
/// # 请求体
/// ```json
/// {
///   "connection_name": "Wired connection 1",
///   "interface": "eth0"
/// }
/// ```
pub async fn disable_token(
    req: web::Json<DisableTokenRequest>,
) -> Result<HttpResponse, Error> {
    info!("API: 禁用 Token - {:?}", req);
    
    // 验证输入
    if let Err(e) = validator::validate_interface(&req.interface) {
        warn!("接口验证失败: {}", e);
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error(e.to_string())
        ));
    }
    
    if let Err(e) = validator::validate_connection_name(&req.connection_name) {
        warn!("连接名称验证失败: {}", e);
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error(e.to_string())
        ));
    }
    
    // 执行 Token 禁用操作
    match TokenOperator::disable_token(
        &req.connection_name,
        &req.interface,
    ).await {
        Ok(result) => {
            if result.success {
                info!("Token 禁用成功");
            } else {
                warn!("Token 禁用失败: {}", result.message);
            }
            Ok(HttpResponse::Ok().json(ApiResponse::success(result)))
        }
        Err(e) => {
            error!("Token 禁用操作错误: {}", e);  // 详细日志
            Ok(HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("Token 禁用失败，请检查日志".to_string())  // 通用消息
            ))
        }
    }
}

/// GET /api/ipv6/check - 检查 IPv6 Token 状态
/// 
/// # 查询参数
/// - connection_name: NetworkManager 连接名称
/// - interface: 网络接口名称
pub async fn check_status(
    query: web::Query<CheckStatusRequest>,
) -> Result<HttpResponse, Error> {
    info!("API: 检查状态 - {:?}", query);
    
    // 验证输入
    if let Err(e) = validator::validate_interface(&query.interface) {
        warn!("接口验证失败: {}", e);
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error(e.to_string())
        ));
    }
    
    if let Err(e) = validator::validate_connection_name(&query.connection_name) {
        warn!("连接名称验证失败: {}", e);
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error(e.to_string())
        ));
    }
    
    // 检查 Token 状态
    match NetworkManager::check_token_status(
        &query.connection_name,
        &query.interface,
    ).await {
        Ok(status) => {
            Ok(HttpResponse::Ok().json(ApiResponse::success(status)))
        }
        Err(e) => {
            error!("检查状态错误: {}", e);  // 详细日志
            Ok(HttpResponse::InternalServerError().json(
                ApiResponse::<()>::error("检查状态失败，请检查日志".to_string())  // 通用消息
            ))
        }
    }
}

/// GET /api/ipv6/prefix-info - 获取前缀信息
/// 
/// 检测指定接口的 IPv6 前缀长度，并提供后缀长度建议
/// 
/// # 查询参数
/// - interface: 网络接口名称
/// 
/// # 返回
/// ```json
/// {
///   "success": true,
///   "data": {
///     "detected_prefix_length": 64,
///     "suggested_suffix_length": 64,
///     "prefix_examples": {
///       "16": "::xxxx",
///       "32": "::xxxx:xxxx",
///       "64": "::xxxx:xxxx:xxxx:xxxx"
///     },
///     "current_addresses": [...]
///   }
/// }
/// ```
/// 
/// # 安全性
/// - 需要认证（通过认证中间件）
/// - 仅执行只读操作
/// - 严格的输入验证
/// - 不泄露敏感信息
pub async fn get_prefix_info(
    query: web::Query<PrefixInfoQuery>,
) -> Result<HttpResponse, Error> {
    info!("API: 获取前缀信息 - 接口={}", query.interface);
    
    // 1. 验证接口名称（防止命令注入）
    if let Err(e) = validator::validate_interface(&query.interface) {
        warn!("接口验证失败: {}", e);
        return Ok(HttpResponse::BadRequest().json(
            ApiResponse::<()>::error(e.to_string())
        ));
    }
    
    // 2. 获取 IPv6 地址（只读操作，安全）
    let addresses = match NetworkManager::get_ipv6_addresses(&query.interface).await {
        Ok(addrs) => addrs,
        Err(e) => {
            warn!("获取 IPv6 地址失败: {}", e);
            // 返回默认值而不是错误，提供更好的用户体验
            vec![]
        }
    };
    
    // 3. 检测前缀长度（从第一个全局地址）
    let detected_prefix = addresses.iter()
        .find(|addr| addr.scope == "global" && !addr.is_temporary)
        .map(|addr| addr.prefix_len)
        .unwrap_or(64); // 默认 /64
    
    // 4. 计算建议后缀长度
    let suggested_suffix = if detected_prefix <= 64 {
        128 - detected_prefix
    } else {
        64 // 如果前缀 > 64，建议使用 64 位后缀
    };
    
    // 5. 生成 Token 格式示例
    let mut prefix_examples = std::collections::HashMap::new();
    prefix_examples.insert("16".to_string(), "::xxxx (例如: ::888, ::1234)".to_string());
    prefix_examples.insert("32".to_string(), "::xxxx:xxxx (例如: ::1234:5678)".to_string());
    prefix_examples.insert("64".to_string(), "::xxxx:xxxx:xxxx:xxxx (例如: ::1234:5678:abcd:ef01)".to_string());
    
    // 6. 构建响应
    let warning = if detected_prefix == 64 && addresses.is_empty() {
        Some("未检测到 IPv6 地址，使用默认前缀长度 /64".to_string())
    } else {
        None
    };
    
    let prefix_info = PrefixInfo {
        detected_prefix_length: detected_prefix,
        suggested_suffix_length: suggested_suffix,
        prefix_examples,
        current_addresses: addresses,
        warning,
    };
    
    info!("前缀检测完成: 前缀={}, 建议后缀={}", detected_prefix, suggested_suffix);
    Ok(HttpResponse::Ok().json(ApiResponse::success(prefix_info)))
}

/// 前缀信息查询参数
#[derive(Debug, serde::Deserialize)]
pub struct PrefixInfoQuery {
    pub interface: String,
}

/// 前缀信息响应
#[derive(Debug, serde::Serialize)]
pub struct PrefixInfo {
    /// 检测到的前缀长度
    pub detected_prefix_length: u8,
    
    /// 建议的后缀长度
    pub suggested_suffix_length: u8,
    
    /// Token 格式示例
    pub prefix_examples: std::collections::HashMap<String, String>,
    
    /// 当前 IPv6 地址列表
    pub current_addresses: Vec<crate::network::IPv6Address>,
    
    /// 警告信息（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}
