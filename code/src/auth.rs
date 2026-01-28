/*!
 * 认证管理模块
 * 
 * 负责会话管理、登录验证和失败追踪
 */

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use log::{info, warn};

use crate::config::AuthConfig;
use crate::crypto;

/// 会话信息
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub ip_address: String,
}

/// 登录尝试记录
#[derive(Debug, Clone)]
pub struct LoginAttempt {
    pub ip_address: String,
    pub failed_count: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_attempt: DateTime<Utc>,
}

/// 认证管理器
pub struct AuthManager {
    pub config: AuthConfig,
    password_hash: Arc<RwLock<Option<String>>>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    login_attempts: Arc<RwLock<HashMap<String, LoginAttempt>>>,
}

impl AuthManager {
    /// 创建认证管理器
    pub fn new(config: AuthConfig) -> Self {
        let password_hash = config.password_hash.clone();
        
        Self {
            config,
            password_hash: Arc::new(RwLock::new(password_hash)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            login_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 检查是否需要初始化密码
    pub fn needs_password_setup(&self) -> bool {
        let hash_guard = self.password_hash.read().unwrap();
        match hash_guard.as_ref() {
            None => true,
            Some(hash) => hash.trim().is_empty(),  // 空字符串也视为未设置
        }
    }
    
    /// 设置初始密码
    pub fn setup_password(&self, password: &str) -> Result<String, String> {
        // 验证密码强度
        if password.len() < 6 {
            return Err("密码长度至少为 6 位".to_string());
        }
        
        // 生成密码哈希
        let hash = crypto::hash_password(password)
            .map_err(|e| format!("密码哈希生成失败: {}", e))?;
        
        // 更新密码哈希
        *self.password_hash.write().unwrap() = Some(hash.clone());
        
        info!("初始密码已设置");
        Ok(hash)
    }
    
    /// 更新密码哈希
    pub fn update_password_hash(&self, hash: String) {
        *self.password_hash.write().unwrap() = Some(hash);
    }
    
    /// 验证密码
    pub fn verify_password(&self, password: &str) -> Result<bool, String> {
        let hash_guard = self.password_hash.read().unwrap();
        let hash = hash_guard.as_ref()
            .ok_or_else(|| "密码未设置".to_string())?;
        
        crypto::verify_password(password, hash)
            .map_err(|e| format!("密码验证失败: {}", e))
    }
    
    /// 创建会话
    pub fn create_session(&self, ip: &str) -> Result<String, String> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let session = Session {
            session_id: session_id.clone(),
            created_at: now,
            last_accessed: now,
            ip_address: ip.to_string(),
        };
        
        self.sessions.write().unwrap().insert(session_id.clone(), session);
        
        info!("会话已创建: {} (IP: {})", session_id, ip);
        Ok(session_id)
    }
    
    /// 验证会话
    pub fn validate_session(&self, session_id: &str) -> Result<bool, String> {
        let mut sessions = self.sessions.write().unwrap();
        
        if let Some(session) = sessions.get_mut(session_id) {
            let now = Utc::now();
            let timeout = Duration::minutes(self.config.session_timeout_minutes as i64);
            
            // 检查会话是否过期
            if now - session.last_accessed > timeout {
                sessions.remove(session_id);
                return Ok(false);
            }
            
            // 更新最后访问时间
            session.last_accessed = now;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// 删除会话（登出）
    pub fn delete_session(&self, session_id: &str) {
        if self.sessions.write().unwrap().remove(session_id).is_some() {
            info!("会话已删除: {}", session_id);
        }
    }
    
    /// 记录登录失败
    pub fn record_failed_login(&self, ip: &str) {
        let mut attempts = self.login_attempts.write().unwrap();
        let now = Utc::now();
        
        let attempt = attempts.entry(ip.to_string()).or_insert(LoginAttempt {
            ip_address: ip.to_string(),
            failed_count: 0,
            locked_until: None,
            last_attempt: now,
        });
        
        attempt.failed_count += 1;
        attempt.last_attempt = now;
        
        // 检查是否需要锁定
        if attempt.failed_count >= self.config.max_login_attempts {
            let lockout_duration = Duration::minutes(self.config.lockout_duration_minutes as i64);
            attempt.locked_until = Some(now + lockout_duration);
            warn!("IP {} 登录失败次数过多，已锁定 {} 分钟", 
                  ip, self.config.lockout_duration_minutes);
        }
    }
    
    /// 重置登录失败计数（成功登录后）
    pub fn reset_failed_login(&self, ip: &str) {
        self.login_attempts.write().unwrap().remove(ip);
    }
    
    /// 检查 IP 是否被锁定
    pub fn is_ip_locked(&self, ip: &str) -> bool {
        let attempts = self.login_attempts.read().unwrap();
        
        if let Some(attempt) = attempts.get(ip) {
            if let Some(locked_until) = attempt.locked_until {
                let now = Utc::now();
                if now < locked_until {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// 获取剩余尝试次数
    pub fn get_remaining_attempts(&self, ip: &str) -> u32 {
        let attempts = self.login_attempts.read().unwrap();
        
        if let Some(attempt) = attempts.get(ip) {
            self.config.max_login_attempts.saturating_sub(attempt.failed_count)
        } else {
            self.config.max_login_attempts
        }
    }
    
    /// 清理过期会话和解锁记录
    pub async fn cleanup_expired(&self) {
        let now = Utc::now();
        let timeout = Duration::minutes(self.config.session_timeout_minutes as i64);
        
        // 清理过期会话
        {
            let mut sessions = self.sessions.write().unwrap();
            let before_count = sessions.len();
            sessions.retain(|_, session| {
                now - session.last_accessed <= timeout
            });
            let removed = before_count - sessions.len();
            if removed > 0 {
                info!("清理了 {} 个过期会话", removed);
            }
        }
        
        // 清理过期的锁定记录
        {
            let mut attempts = self.login_attempts.write().unwrap();
            attempts.retain(|_, attempt| {
                if let Some(locked_until) = attempt.locked_until {
                    now < locked_until
                } else {
                    // 保留未锁定的记录（用于计数）
                    true
                }
            });
        }
    }
    
    /// 获取会话统计信息
    pub fn get_stats(&self) -> (usize, usize) {
        let sessions_count = self.sessions.read().unwrap().len();
        let locked_ips_count = self.login_attempts.read().unwrap()
            .values()
            .filter(|a| {
                if let Some(locked_until) = a.locked_until {
                    Utc::now() < locked_until
                } else {
                    false
                }
            })
            .count();
        
        (sessions_count, locked_ips_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AuthConfig;

    fn test_config() -> AuthConfig {
        AuthConfig {
            password_hash: Some("$argon2id$v=19$m=19456,t=2,p=1$test$hash".to_string()),
            session_timeout_minutes: 30,
            max_login_attempts: 3,
            lockout_duration_minutes: 15,
        }
    }

    #[test]
    fn test_create_session() {
        let manager = AuthManager::new(test_config());
        let session_id = manager.create_session("127.0.0.1").unwrap();
        
        assert!(!session_id.is_empty());
        assert!(manager.validate_session(&session_id).unwrap());
    }

    #[test]
    fn test_delete_session() {
        let manager = AuthManager::new(test_config());
        let session_id = manager.create_session("127.0.0.1").unwrap();
        
        manager.delete_session(&session_id);
        assert!(!manager.validate_session(&session_id).unwrap());
    }

    #[test]
    fn test_login_failure_tracking() {
        let manager = AuthManager::new(test_config());
        let ip = "192.168.1.1";
        
        // 第一次失败
        manager.record_failed_login(ip);
        assert!(!manager.is_ip_locked(ip));
        assert_eq!(manager.get_remaining_attempts(ip), 2);
        
        // 第二次失败
        manager.record_failed_login(ip);
        assert!(!manager.is_ip_locked(ip));
        assert_eq!(manager.get_remaining_attempts(ip), 1);
        
        // 第三次失败 - 应该被锁定
        manager.record_failed_login(ip);
        assert!(manager.is_ip_locked(ip));
        assert_eq!(manager.get_remaining_attempts(ip), 0);
    }

    #[test]
    fn test_reset_failed_login() {
        let manager = AuthManager::new(test_config());
        let ip = "192.168.1.2";
        
        manager.record_failed_login(ip);
        manager.record_failed_login(ip);
        assert_eq!(manager.get_remaining_attempts(ip), 1);
        
        manager.reset_failed_login(ip);
        assert_eq!(manager.get_remaining_attempts(ip), 3);
    }
}
