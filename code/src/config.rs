/*!
 * 配置管理模块
 * 
 * 负责加载和管理应用程序配置
 * 支持 YAML 配置文件和环境变量
 */

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use log::info;

/// 应用程序配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// 服务器配置
    pub server: ServerConfig,
    /// 认证配置
    pub auth: AuthConfig,
    /// 路径配置
    pub paths: PathsConfig,
    /// 日志配置
    pub logging: LoggingConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
}

/// 认证配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// 密码哈希（Argon2id 格式）
    /// 如果为 None，表示首次启动，需要设置密码
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
    
    /// 会话超时时间（分钟）
    #[serde(default = "default_session_timeout")]
    pub session_timeout_minutes: u64,
    
    /// 最大登录尝试次数
    #[serde(default = "default_max_attempts")]
    pub max_login_attempts: u32,
    
    /// IP 锁定时间（分钟）
    #[serde(default = "default_lockout_duration")]
    pub lockout_duration_minutes: u64,
}

/// 路径配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathsConfig {
    /// Shell 脚本目录
    #[serde(default = "default_script_dir")]
    pub script_dir: String,
    
    /// 前端静态文件目录
    #[serde(default = "default_frontend_dir")]
    pub frontend_dir: String,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// 日志级别
    #[serde(default = "default_log_level")]
    pub level: String,
}

// 默认值函数
fn default_session_timeout() -> u64 { 30 }
fn default_max_attempts() -> u32 { 5 }
fn default_lockout_duration() -> u64 { 15 }
fn default_script_dir() -> String { "./sh/token-eui64".to_string() }
fn default_frontend_dir() -> String { "./frontend".to_string() }
fn default_log_level() -> String { "info".to_string() }

impl AppConfig {
    /// 配置文件路径
    pub fn config_path() -> PathBuf {
        PathBuf::from("config/config.yaml")
    }
    
    /// 加载配置
    /// 
    /// 优先级：环境变量 > 配置文件 > 默认值
    pub fn load() -> Result<Self, String> {
        let config_path = Self::config_path();
        
        // 如果配置文件不存在，创建默认配置
        if !config_path.exists() {
            info!("配置文件不存在，创建默认配置: {:?}", config_path);
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }
        
        // 读取配置文件
        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;
        
        // 解析 YAML
        let mut config: AppConfig = serde_yaml::from_str(&config_content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;
        
        // 环境变量覆盖
        if let Ok(host) = env::var("SERVER_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port_num) = port.parse() {
                config.server.port = port_num;
            }
        }
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.logging.level = log_level;
        }
        
        Ok(config)
    }
    
    /// 保存配置到文件
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path();
        
        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
        
        // 序列化为 YAML
        let yaml_content = serde_yaml::to_string(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        
        // 写入文件
        fs::write(&config_path, yaml_content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;
        
        info!("配置已保存到: {:?}", config_path);
        Ok(())
    }
    
    /// 更新密码哈希
    pub fn update_password_hash(&mut self, hash: String) -> Result<(), String> {
        self.auth.password_hash = Some(hash);
        self.save()
    }
    
    /// 检查是否需要初始化密码
    pub fn needs_password_setup(&self) -> bool {
        match &self.auth.password_hash {
            None => true,
            Some(hash) => hash.trim().is_empty(),  // 空字符串也视为未设置
        }
    }
    
    /// 获取默认配置
    pub fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 5000,
            },
            auth: AuthConfig {
                password_hash: None,  // 首次启动需要设置密码
                session_timeout_minutes: 30,
                max_login_attempts: 5,
                lockout_duration_minutes: 15,
            },
            paths: PathsConfig {
                script_dir: "./sh/token-eui64".to_string(),
                frontend_dir: "./frontend".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
            },
        }
    }
}

// 保留旧的字段名以便向后兼容
impl AppConfig {
    pub fn script_dir(&self) -> &str {
        &self.paths.script_dir
    }
    
    pub fn frontend_dir(&self) -> &str {
        &self.paths.frontend_dir
    }
    
    pub fn log_level(&self) -> &str {
        &self.logging.level
    }
}
