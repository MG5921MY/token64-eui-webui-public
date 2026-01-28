/*!
 * 数据模型定义
 * 
 * 定义所有 API 请求和响应的数据结构
 */

use serde::{Deserialize, Serialize};

/// API 通用响应结构
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    /// 创建错误响应
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// 启用 Token 请求
#[derive(Debug, Deserialize)]
pub struct EnableTokenRequest {
    /// NetworkManager 连接名称（如 "Wired connection 1"）
    pub connection_name: String,
    /// 网络接口名称（如 "eth0", "ens33"）
    pub interface: String,
    /// Token 后缀（如 "::888"）
    pub token: String,
    /// 隐私模式：0=禁用临时IPv6, 2=启用临时IPv6优先出站
    pub privacy_mode: i32,
}

/// 禁用 Token 请求
#[derive(Debug, Deserialize)]
pub struct DisableTokenRequest {
    /// NetworkManager 连接名称
    pub connection_name: String,
    /// 网络接口名称
    pub interface: String,
}

/// 检查状态请求
#[derive(Debug, Deserialize)]
pub struct CheckStatusRequest {
    /// NetworkManager 连接名称
    pub connection_name: String,
    /// 网络接口名称
    pub interface: String,
}

/// 命令执行结果
#[derive(Debug, Serialize, Clone)]
pub struct CommandResult {
    /// 是否成功执行
    pub success: bool,
    /// 标准输出
    pub stdout: String,
    /// 标准错误输出
    pub stderr: String,
    /// 退出码
    pub exit_code: Option<i32>,
}

/// 系统信息
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    /// 网络接口列表
    pub interfaces: Vec<NetworkInterface>,
    /// NetworkManager 连接列表
    pub connections: Vec<NetworkConnection>,
}

/// 网络接口信息
#[derive(Debug, Serialize)]
pub struct NetworkInterface {
    /// 接口名称
    pub name: String,
    /// MAC 地址
    pub mac_address: Option<String>,
    /// IPv6 地址列表
    pub ipv6_addresses: Vec<String>,
    /// 接口状态
    pub status: String,
}

/// NetworkManager 连接信息
#[derive(Debug, Serialize)]
pub struct NetworkConnection {
    /// 连接名称
    pub name: String,
    /// UUID
    pub uuid: String,
    /// 类型（ethernet, wifi 等）
    pub conn_type: String,
    /// 关联的设备
    pub device: Option<String>,
}

/// IPv6 状态检查结果
#[derive(Debug, Serialize)]
pub struct IPv6Status {
    /// 原始输出
    pub raw_output: String,
    /// 是否配置了 Token
    pub token_configured: bool,
    /// 当前 Token 值
    pub current_token: Option<String>,
    /// IPv6 地址列表
    pub addresses: Vec<IPv6Address>,
}

/// IPv6 地址信息
#[derive(Debug, Serialize)]
pub struct IPv6Address {
    /// 地址
    pub address: String,
    /// 前缀长度
    pub prefix_length: u8,
    /// 地址类型（Token, DHCPv6, SLAAC 等）
    pub addr_type: String,
    /// 作用域
    pub scope: String,
}
