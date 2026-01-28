/*!
 * IPv6 Token 操作模块
 * 
 * 使用 Rust 原生实现 Token 的启用、禁用和检查功能
 * 替代原有的 Shell 脚本
 */

use std::time::Duration;
use tokio::time::{timeout, sleep};
use log::{info, warn};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

/// Token 操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenOperationResult {
    /// 是否成功
    pub success: bool,
    /// 操作消息
    pub message: String,
    /// 详细日志
    pub logs: Vec<String>,
}

/// Token 操作器
pub struct TokenOperator;

impl TokenOperator {
    /// 启用 IPv6 Token
    pub async fn enable_token(
        connection_name: &str,
        interface: &str,
        token: &str,
        privacy_mode: i32,
    ) -> Result<TokenOperationResult> {
        info!("启用 Token: 连接={}, 接口={}, Token={}, 隐私模式={}", 
              connection_name, interface, token, privacy_mode);
        
        let mut logs = Vec::new();
        logs.push(format!("=== 启用 IPv6 Token {} ===", token));
        logs.push(String::new());
        
        // 检查 root 权限（通过尝试执行需要权限的命令）
        if !Self::check_root_permission().await {
            return Ok(TokenOperationResult {
                success: false,
                message: "需要 root 权限".to_string(),
                logs,
            });
        }
        
        // 1. 获取配置前的地址
        logs.push("配置前地址:".to_string());
        if let Ok(addrs) = Self::get_ipv6_addresses(interface).await {
            if addrs.is_empty() {
                logs.push("  无".to_string());
            } else {
                for addr in addrs {
                    logs.push(format!("  {}", addr));
                }
            }
        }
        logs.push(String::new());
        
        // 2. 配置 IPv6 方法
        logs.push("1. 配置 IPv6 方法...".to_string());
        Self::nmcli_modify(connection_name, "ipv6.method", "auto").await?;
        
        // 3. 禁用 DHCPv6（关键）
        logs.push("2. 禁用 DHCPv6（关键）...".to_string());
        Self::nmcli_modify(connection_name, "ipv6.dhcp-timeout", "0").await?;
        
        // 4. 设置地址生成模式（必须在 Token 之前）
        logs.push("3. 设置地址生成模式（必须在 Token 之前）...".to_string());
        Self::nmcli_modify(connection_name, "ipv6.addr-gen-mode", "eui64").await?;
        
        // 5. 设置 Token
        logs.push("4. 设置 Token...".to_string());
        Self::nmcli_modify(connection_name, "ipv6.token", token).await?;
        
        // 6. 设置隐私扩展
        logs.push(format!("5. 设置隐私扩展模式为 {}...", privacy_mode));
        Self::nmcli_modify(connection_name, "ipv6.ip6-privacy", &privacy_mode.to_string()).await?;
        
        // 7. 清除旧地址
        logs.push("6. 清除旧地址...".to_string());
        let _ = Self::flush_ipv6_addresses(interface).await; // 忽略错误
        
        // 8. 重启连接
        logs.push("7. 重启连接...".to_string());
        Self::restart_connection(connection_name).await?;
        
        // 9. 等待地址生成
        logs.push("等待 10 秒让地址生成...".to_string());
        sleep(Duration::from_secs(10)).await;
        logs.push(String::new());
        
        // 10. 获取新地址
        logs.push("=== 配置完成 ===".to_string());
        logs.push(String::new());
        logs.push("新地址:".to_string());
        
        let mut token_found = false;
        if let Ok(addrs) = Self::get_ipv6_addresses(interface).await {
            for addr in &addrs {
                logs.push(format!("  {}", addr));
                // 检查是否包含 Token（去掉前导 ::）
                let token_suffix = token.trim_start_matches(':');
                if addr.contains(token_suffix) {
                    token_found = true;
                }
            }
        }
        logs.push(String::new());
        
        // 11. 验证结果
        if token_found {
            logs.push(format!("✅ Token {} 已生效！", token));
            Ok(TokenOperationResult {
                success: true,
                message: format!("Token {} 已成功启用", token),
                logs,
            })
        } else {
            logs.push(format!("⚠️  Token {} 可能未生效", token));
            logs.push("请检查路由器是否发送 RA 通告".to_string());
            Ok(TokenOperationResult {
                success: false,
                message: "Token 配置完成但未检测到对应地址".to_string(),
                logs,
            })
        }
    }
    
    /// 禁用 IPv6 Token
    pub async fn disable_token(
        connection_name: &str,
        interface: &str,
    ) -> Result<TokenOperationResult> {
        info!("禁用 Token: 连接={}, 接口={}", connection_name, interface);
        
        let mut logs = Vec::new();
        logs.push("=== 禁用 Token ===".to_string());
        logs.push(String::new());
        
        // 检查 root 权限
        if !Self::check_root_permission().await {
            return Ok(TokenOperationResult {
                success: false,
                message: "需要 root 权限".to_string(),
                logs,
            });
        }
        
        // 1. 获取当前地址
        logs.push("当前地址:".to_string());
        if let Ok(addrs) = Self::get_ipv6_addresses(interface).await {
            if addrs.is_empty() {
                logs.push("  无".to_string());
            } else {
                for addr in addrs {
                    logs.push(format!("  {}", addr));
                }
            }
        }
        logs.push(String::new());
        
        // 2. 清除 Token
        logs.push("1. 清除 Token...".to_string());
        Self::nmcli_modify(connection_name, "ipv6.token", "").await?;
        
        // 3. 恢复 DHCPv6
        logs.push("2. 恢复 DHCPv6...".to_string());
        Self::nmcli_modify(connection_name, "ipv6.dhcp-timeout", "").await?;
        
        // 4. 恢复地址生成模式
        logs.push("3. 恢复地址生成模式...".to_string());
        Self::nmcli_modify(connection_name, "ipv6.addr-gen-mode", "stable-privacy").await?;
        
        // 5. 启用隐私扩展
        logs.push("4. 启用隐私扩展...".to_string());
        Self::nmcli_modify(connection_name, "ipv6.ip6-privacy", "2").await?;
        
        // 6. 清除旧地址
        logs.push("5. 清除旧地址...".to_string());
        let _ = Self::flush_ipv6_addresses(interface).await;
        
        // 7. 重启连接
        logs.push("6. 重启连接...".to_string());
        Self::restart_connection(connection_name).await?;
        
        // 8. 等待
        logs.push("等待 10 秒...".to_string());
        sleep(Duration::from_secs(10)).await;
        logs.push(String::new());
        
        // 9. 获取新地址
        logs.push("=== 恢复完成 ===".to_string());
        logs.push(String::new());
        logs.push("新地址:".to_string());
        if let Ok(addrs) = Self::get_ipv6_addresses(interface).await {
            if addrs.is_empty() {
                logs.push("  无".to_string());
            } else {
                for addr in addrs {
                    logs.push(format!("  {}", addr));
                }
            }
        }
        logs.push(String::new());
        logs.push("✅ 已恢复默认配置（DHCPv6 或 SLAAC）".to_string());
        
        Ok(TokenOperationResult {
            success: true,
            message: "Token 已成功禁用".to_string(),
            logs,
        })
    }
    
    /// 检查 root 权限
    async fn check_root_permission() -> bool {
        // 尝试执行一个需要权限的命令
        let result = timeout(
            Duration::from_secs(5),
            tokio::process::Command::new("nmcli")
                .args(&["general", "permissions"])
                .output()
        )
        .await;
        
        match result {
            Ok(Ok(output)) => output.status.success(),
            _ => false,
        }
    }
    
    /// 修改 NetworkManager 连接配置
    async fn nmcli_modify(connection_name: &str, key: &str, value: &str) -> Result<()> {
        let output = timeout(
            Duration::from_secs(30),
            tokio::process::Command::new("nmcli")
                .args(&["connection", "modify", connection_name, key, value])
                .output()
        )
        .await
        .context("nmcli modify 超时")?
        .context("执行 nmcli modify 失败")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("nmcli modify 失败: {}", stderr);
        }
        
        Ok(())
    }
    
    /// 重启网络连接
    async fn restart_connection(connection_name: &str) -> Result<()> {
        // 先 down
        let _ = timeout(
            Duration::from_secs(10),
            tokio::process::Command::new("nmcli")
                .args(&["connection", "down", connection_name])
                .output()
        )
        .await; // 忽略错误，可能连接已经是 down 状态
        
        // 等待 2 秒
        sleep(Duration::from_secs(2)).await;
        
        // 再 up
        let output = timeout(
            Duration::from_secs(30),
            tokio::process::Command::new("nmcli")
                .args(&["connection", "up", connection_name])
                .output()
        )
        .await
        .context("nmcli up 超时")?
        .context("执行 nmcli up 失败")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("nmcli up 失败: {}", stderr);
        }
        
        Ok(())
    }
    
    /// 清除 IPv6 地址
    async fn flush_ipv6_addresses(interface: &str) -> Result<()> {
        let output = timeout(
            Duration::from_secs(10),
            tokio::process::Command::new("ip")
                .args(&["-6", "addr", "flush", "dev", interface, "scope", "global"])
                .output()
        )
        .await
        .context("ip addr flush 超时")?
        .context("执行 ip addr flush 失败")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("ip addr flush 警告: {}", stderr);
        }
        
        Ok(())
    }
    
    /// 获取 IPv6 地址（仅全局地址）
    async fn get_ipv6_addresses(interface: &str) -> Result<Vec<String>> {
        let output = timeout(
            Duration::from_secs(10),
            tokio::process::Command::new("ip")
                .args(&["-6", "addr", "show", interface])
                .output()
        )
        .await
        .context("获取 IPv6 地址超时")?
        .context("执行 ip -6 addr show 失败")?;
        
        if !output.status.success() {
            anyhow::bail!("ip -6 addr show 失败");
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut addresses = Vec::new();
        
        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("inet6") && line.contains("scope global") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    addresses.push(parts[1].to_string());
                }
            }
        }
        
        Ok(addresses)
    }
}
