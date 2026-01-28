/*!
 * 网络管理模块
 * 
 * 使用 Rust 原生实现 NetworkManager 和 ip 命令的功能
 * 不依赖外部 Shell 脚本
 */

use std::time::Duration;
use tokio::time::timeout;
use log::info;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};

/// 网络接口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// 接口名称（如 eth0, ens33）
    pub name: String,
    /// MAC 地址
    pub mac: Option<String>,
    /// 是否启用
    pub is_up: bool,
}

/// NetworkManager 连接信息
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// IPv6 地址信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPv6Address {
    /// 地址
    pub address: String,
    /// 前缀长度
    pub prefix_len: u8,
    /// 作用域（global, link, host）
    pub scope: String,
    /// 是否是临时地址
    pub is_temporary: bool,
}

/// Token 配置状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStatus {
    /// NetworkManager 中配置的 Token
    pub nm_token: Option<String>,
    /// 系统内核中的 Token
    pub kernel_token: Option<String>,
    /// IPv6 地址列表
    pub addresses: Vec<IPv6Address>,
    /// 地址生成模式
    pub addr_gen_mode: Option<String>,
    /// 隐私模式
    pub privacy_mode: Option<i32>,
}

/// 网络管理器
pub struct NetworkManager;

impl NetworkManager {
    /// 获取所有网络接口
    pub async fn get_interfaces() -> Result<Vec<NetworkInterface>> {
        info!("获取网络接口列表");
        
        let output = timeout(
            Duration::from_secs(10),
            tokio::process::Command::new("ip")
                .args(&["link", "show"])
                .output()
        )
        .await
        .context("获取接口列表超时")?
        .context("执行 ip link show 失败")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ip link show 失败: {}", stderr);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let interfaces = Self::parse_interfaces(&stdout)?;
        
        info!("找到 {} 个网络接口", interfaces.len());
        Ok(interfaces)
    }
    
    /// 解析 ip link show 输出
    fn parse_interfaces(output: &str) -> Result<Vec<NetworkInterface>> {
        let mut interfaces = Vec::new();
        let lines: Vec<&str> = output.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            
            // 匹配接口行: "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> ..."
            if let Some(colon_pos) = line.find(':') {
                if let Some(second_colon) = line[colon_pos + 1..].find(':') {
                    let name_part = &line[colon_pos + 1..colon_pos + 1 + second_colon];
                    let name = name_part.trim().to_string();
                    
                    // 跳过 lo (loopback)
                    if name == "lo" {
                        i += 1;
                        continue;
                    }
                    
                    let is_up = line.contains("UP");
                    
                    // 查找下一行的 MAC 地址
                    let mut mac = None;
                    if i + 1 < lines.len() {
                        let next_line = lines[i + 1].trim();
                        if next_line.starts_with("link/ether") {
                            let parts: Vec<&str> = next_line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                mac = Some(parts[1].to_string());
                            }
                        }
                    }
                    
                    interfaces.push(NetworkInterface {
                        name,
                        mac,
                        is_up,
                    });
                }
            }
            
            i += 1;
        }
        
        Ok(interfaces)
    }
    
    /// 获取所有 NetworkManager 连接
    pub async fn get_connections() -> Result<Vec<NetworkConnection>> {
        info!("获取 NetworkManager 连接列表");
        
        let output = timeout(
            Duration::from_secs(10),
            tokio::process::Command::new("nmcli")
                .args(&["-t", "-f", "NAME,UUID,TYPE,DEVICE", "connection", "show"])
                .output()
        )
        .await
        .context("获取连接列表超时")?
        .context("执行 nmcli 失败")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("nmcli 失败: {}", stderr);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let connections = Self::parse_connections(&stdout)?;
        
        info!("找到 {} 个连接", connections.len());
        Ok(connections)
    }
    
    /// 解析 nmcli 连接输出
    fn parse_connections(output: &str) -> Result<Vec<NetworkConnection>> {
        let mut connections = Vec::new();
        
        for line in output.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 4 {
                let device = if parts[3].is_empty() || parts[3] == "--" {
                    None
                } else {
                    Some(parts[3].to_string())
                };
                
                connections.push(NetworkConnection {
                    name: parts[0].to_string(),
                    uuid: parts[1].to_string(),
                    conn_type: parts[2].to_string(),
                    device,
                });
            }
        }
        
        Ok(connections)
    }
    
    /// 获取接口的 IPv6 地址
    pub async fn get_ipv6_addresses(interface: &str) -> Result<Vec<IPv6Address>> {
        info!("获取接口 {} 的 IPv6 地址", interface);
        
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
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ip -6 addr show 失败: {}", stderr);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let addresses = Self::parse_ipv6_addresses(&stdout)?;
        
        info!("找到 {} 个 IPv6 地址", addresses.len());
        Ok(addresses)
    }
    
    /// 解析 IPv6 地址输出
    fn parse_ipv6_addresses(output: &str) -> Result<Vec<IPv6Address>> {
        let mut addresses = Vec::new();
        
        for line in output.lines() {
            let line = line.trim();
            if line.starts_with("inet6") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    // 解析地址和前缀长度
                    let addr_parts: Vec<&str> = parts[1].split('/').collect();
                    if addr_parts.len() == 2 {
                        let address = addr_parts[0].to_string();
                        let prefix_len = addr_parts[1].parse::<u8>().unwrap_or(64);
                        
                        // 解析作用域
                        let scope = parts[2].to_string();
                        
                        // 检查是否是临时地址
                        let is_temporary = line.contains("temporary") || line.contains("deprecated");
                        
                        addresses.push(IPv6Address {
                            address,
                            prefix_len,
                            scope,
                            is_temporary,
                        });
                    }
                }
            }
        }
        
        Ok(addresses)
    }
    
    /// 检查 Token 状态
    pub async fn check_token_status(connection_name: &str, interface: &str) -> Result<TokenStatus> {
        info!("检查 Token 状态: 连接={}, 接口={}", connection_name, interface);
        
        // 获取 NetworkManager 配置
        let nm_output = timeout(
            Duration::from_secs(10),
            tokio::process::Command::new("nmcli")
                .args(&["connection", "show", connection_name])
                .output()
        )
        .await
        .context("获取连接配置超时")?
        .context("执行 nmcli 失败")?;
        
        let nm_stdout = String::from_utf8_lossy(&nm_output.stdout);
        
        // 解析 Token
        let nm_token = Self::parse_nm_token(&nm_stdout);
        let addr_gen_mode = Self::parse_nm_field(&nm_stdout, "ipv6.addr-gen-mode");
        let privacy_mode = Self::parse_nm_field(&nm_stdout, "ipv6.ip6-privacy")
            .and_then(|s| s.parse::<i32>().ok());
        
        // 获取系统内核 Token
        let kernel_token = Self::get_kernel_token(interface).await.ok();
        
        // 获取 IPv6 地址
        let addresses = Self::get_ipv6_addresses(interface).await?;
        
        Ok(TokenStatus {
            nm_token,
            kernel_token,
            addresses,
            addr_gen_mode,
            privacy_mode,
        })
    }
    
    /// 解析 NetworkManager Token
    fn parse_nm_token(output: &str) -> Option<String> {
        for line in output.lines() {
            if line.contains("ipv6.token:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let token = parts[1].trim();
                    if token != "--" && !token.is_empty() {
                        return Some(token.to_string());
                    }
                }
            }
        }
        None
    }
    
    /// 解析 NetworkManager 字段
    fn parse_nm_field(output: &str, field: &str) -> Option<String> {
        for line in output.lines() {
            if line.contains(&format!("{}:", field)) {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let value = parts[1].trim();
                    if value != "--" && !value.is_empty() {
                        return Some(value.to_string());
                    }
                }
            }
        }
        None
    }
    
    /// 获取内核 Token
    async fn get_kernel_token(interface: &str) -> Result<String> {
        let output = timeout(
            Duration::from_secs(5),
            tokio::process::Command::new("ip")
                .args(&["token", "show", "dev", interface])
                .output()
        )
        .await
        .context("获取内核 Token 超时")?
        .context("执行 ip token show 失败")?;
        
        if !output.status.success() {
            anyhow::bail!("ip token show 失败");
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let token = stdout.trim().to_string();
        
        if token.is_empty() {
            anyhow::bail!("未找到内核 Token");
        }
        
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_interfaces() {
        let output = r#"
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel state UP mode DEFAULT group default qlen 1000
    link/ether 00:0c:29:3a:2b:1c brd ff:ff:ff:ff:ff:ff
3: wlan0: <BROADCAST,MULTICAST> mtu 1500 qdisc noop state DOWN mode DEFAULT group default qlen 1000
    link/ether 00:11:22:33:44:55 brd ff:ff:ff:ff:ff:ff
"#;
        
        let interfaces = NetworkManager::parse_interfaces(output).unwrap();
        assert_eq!(interfaces.len(), 2); // lo 被跳过
        assert_eq!(interfaces[0].name, "eth0");
        assert_eq!(interfaces[0].mac, Some("00:0c:29:3a:2b:1c".to_string()));
        assert!(interfaces[0].is_up);
        assert_eq!(interfaces[1].name, "wlan0");
        assert!(!interfaces[1].is_up);
    }
    
    #[test]
    fn test_parse_connections() {
        let output = "Wired connection 1:uuid-123:ethernet:eth0\nWiFi:uuid-456:wifi:wlan0\nVPN:uuid-789:vpn:--\n";
        
        let connections = NetworkManager::parse_connections(output).unwrap();
        assert_eq!(connections.len(), 3);
        assert_eq!(connections[0].name, "Wired connection 1");
        assert_eq!(connections[0].device, Some("eth0".to_string()));
        assert_eq!(connections[2].device, None);
    }
    
    #[test]
    fn test_parse_ipv6_addresses() {
        let output = r#"
2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 state UP qlen 1000
    inet6 2001:db8::101/64 scope global 
       valid_lft forever preferred_lft forever
    inet6 fe80::20c:29ff:fe3a:2b1c/64 scope link 
       valid_lft forever preferred_lft forever
    inet6 2001:db8::abc:def/128 scope global temporary 
       valid_lft 86400sec preferred_lft 3600sec
"#;
        
        let addresses = NetworkManager::parse_ipv6_addresses(output).unwrap();
        assert_eq!(addresses.len(), 3);
        assert_eq!(addresses[0].address, "2001:db8::101");
        assert_eq!(addresses[0].prefix_len, 64);
        assert_eq!(addresses[0].scope, "global");
        assert!(!addresses[0].is_temporary);
        assert!(addresses[2].is_temporary);
    }
}
