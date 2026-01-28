/*!
 * 脚本执行器模块
 * 
 * 提供安全的 Shell 脚本执行功能
 * - 脚本白名单验证
 * - 命令注入防护
 * - 超时控制
 * - 输出捕获
 */

use crate::models::CommandResult;
use log::{info, warn, error};
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tokio::sync::Semaphore;
use lazy_static::lazy_static;

lazy_static! {
    /// 全局命令执行信号量（最多 5 个并发）
    static ref COMMAND_SEMAPHORE: Semaphore = Semaphore::new(5);
}

/// 脚本执行器
pub struct ScriptExecutor {
    /// 脚本目录路径
    script_dir: PathBuf,
    /// 允许执行的脚本白名单
    allowed_scripts: Vec<String>,
}

/// 执行器错误
#[derive(Debug)]
pub enum ExecutorError {
    /// 脚本不在白名单中
    ScriptNotAllowed(String),
    /// 脚本文件不存在
    ScriptNotFound(String),
    /// 执行超时
    Timeout,
    /// IO 错误
    IoError(std::io::Error),
    /// 请求过多
    TooManyRequests,
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScriptNotAllowed(script) => write!(f, "脚本不允许执行: {}", script),
            Self::ScriptNotFound(script) => write!(f, "脚本文件不存在: {}", script),
            Self::Timeout => write!(f, "脚本执行超时（60秒）"),
            Self::IoError(e) => write!(f, "IO 错误: {}", e),
            Self::TooManyRequests => write!(f, "请求过多，请稍后重试"),
        }
    }
}

impl std::error::Error for ExecutorError {}

impl From<std::io::Error> for ExecutorError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

impl ScriptExecutor {
    /// 创建新的脚本执行器
    /// 
    /// # 参数
    /// - `script_dir`: 脚本目录路径
    pub fn new(script_dir: &str) -> Self {
        Self {
            script_dir: PathBuf::from(script_dir),
            allowed_scripts: vec![
                "enable-token-101.sh".to_string(),
                "disable-token.sh".to_string(),
                "check-token.sh".to_string(),
            ],
        }
    }
    
    /// 验证脚本是否在白名单中
    fn is_script_allowed(&self, script_name: &str) -> bool {
        self.allowed_scripts.contains(&script_name.to_string())
    }
    
    /// 安全执行脚本
    /// 
    /// # 安全机制
    /// 1. 脚本白名单验证
    /// 2. 路径规范化验证（防止路径遍历）
    /// 3. 参数数组传递（防止命令注入）
    /// 4. 超时控制（60秒）
    /// 5. 并发限制（最多 5 个并发）
    /// 
    /// # 参数
    /// - `script_name`: 脚本文件名（必须在白名单中）
    /// - `args`: 脚本参数数组
    /// 
    /// # 返回
    /// - `Ok(CommandResult)`: 执行成功，包含输出和退出码
    /// - `Err(ExecutorError)`: 执行失败
    pub async fn execute_script(
        &self,
        script_name: &str,
        args: Vec<String>,
    ) -> Result<CommandResult, ExecutorError> {
        // 0. 获取信号量许可（限制并发）
        let _permit = COMMAND_SEMAPHORE.acquire().await
            .map_err(|e| {
                error!("获取执行许可失败: {}", e);
                ExecutorError::TooManyRequests
            })?;
        
        info!("获得执行许可，当前可用: {}", COMMAND_SEMAPHORE.available_permits());
        
        // 1. 验证脚本名称在白名单中
        if !self.is_script_allowed(script_name) {
            warn!("尝试执行不允许的脚本: {}", script_name);
            return Err(ExecutorError::ScriptNotAllowed(script_name.to_string()));
        }
        
        // 2. 构建完整脚本路径
        let script_path = self.script_dir.join(script_name);
        
        // 3. 规范化脚本目录路径
        let canonical_script_dir = self.script_dir.canonicalize()
            .map_err(|e| {
                error!("无法规范化脚本目录 {:?}: {}", self.script_dir, e);
                ExecutorError::IoError(e)
            })?;
        
        // 4. 规范化脚本路径（防止路径遍历）
        let canonical_script_path = script_path.canonicalize()
            .map_err(|e| {
                error!("脚本文件不存在或无法访问: {:?} - {}", script_path, e);
                ExecutorError::ScriptNotFound(script_path.to_string_lossy().to_string())
            })?;
        
        // 5. 确保规范化后的路径仍在脚本目录内（防止路径遍历攻击）
        if !canonical_script_path.starts_with(&canonical_script_dir) {
            error!("检测到路径遍历攻击: {:?} 不在 {:?} 内", 
                   canonical_script_path, canonical_script_dir);
            return Err(ExecutorError::ScriptNotAllowed(script_name.to_string()));
        }
        
        info!("执行脚本: {:?} 参数: {:?}", canonical_script_path, args);
        
        // 6. 构建命令（使用参数数组，防止注入）
        let mut cmd = Command::new("bash");
        cmd.arg(&canonical_script_path);
        cmd.args(&args);
        
        // 7. 执行命令（带超时控制）
        let result = timeout(
            Duration::from_secs(60),  // 60 秒超时
            cmd.output()
        ).await;
        
        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();
                let success = output.status.success();
                
                if success {
                    info!("脚本执行成功: {}", script_name);
                } else {
                    warn!("脚本执行失败: {} 退出码: {:?}", script_name, exit_code);
                }
                
                Ok(CommandResult {
                    success,
                    stdout,
                    stderr,
                    exit_code,
                })
            }
            Ok(Err(e)) => {
                error!("脚本执行 IO 错误: {}", e);
                Err(ExecutorError::IoError(e))
            }
            Err(_) => {
                error!("脚本执行超时: {}", script_name);
                Err(ExecutorError::Timeout)
            }
        }
    }
    
    /// 执行系统命令（用于获取系统信息）
    /// 
    /// # 安全说明
    /// 此方法仅用于执行预定义的系统命令（如 nmcli, ip）
    /// 不接受用户输入作为命令名称
    pub async fn execute_command(
        &self,
        command: &str,
        args: Vec<&str>,
    ) -> Result<CommandResult, ExecutorError> {
        info!("执行系统命令: {} {:?}", command, args);
        
        let mut cmd = Command::new(command);
        cmd.args(&args);
        
        let result = timeout(
            Duration::from_secs(10),  // 系统命令 10 秒超时
            cmd.output()
        ).await;
        
        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();
                let success = output.status.success();
                
                Ok(CommandResult {
                    success,
                    stdout,
                    stderr,
                    exit_code,
                })
            }
            Ok(Err(e)) => Err(ExecutorError::IoError(e)),
            Err(_) => Err(ExecutorError::Timeout),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_script_allowed() {
        let executor = ScriptExecutor::new("./scripts");
        
        // 允许的脚本
        assert!(executor.is_script_allowed("enable-token-101.sh"));
        assert!(executor.is_script_allowed("disable-token.sh"));
        assert!(executor.is_script_allowed("check-token.sh"));
        
        // 不允许的脚本
        assert!(!executor.is_script_allowed("malicious.sh"));
        assert!(!executor.is_script_allowed("../../../etc/passwd"));
    }
    
    #[tokio::test]
    async fn test_execute_script_not_allowed() {
        let executor = ScriptExecutor::new("./scripts");
        
        let result = executor.execute_script(
            "malicious.sh",
            vec![]
        ).await;
        
        assert!(matches!(result, Err(ExecutorError::ScriptNotAllowed(_))));
    }
}
