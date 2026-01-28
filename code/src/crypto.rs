/*!
 * 密码哈希模块
 * 
 * 使用 Argon2id 算法进行密码哈希和验证
 */

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("密码哈希生成失败: {0}")]
    HashError(String),
    
    #[error("密码验证失败: {0}")]
    VerifyError(String),
    
    #[error("密码格式无效")]
    InvalidFormat,
}

/// 使用 Argon2id 生成密码哈希
/// 
/// # 参数
/// - `password`: 明文密码
/// 
/// # 返回
/// - `Ok(String)`: PHC 格式的密码哈希字符串
/// - `Err(CryptoError)`: 哈希生成失败
pub fn hash_password(password: &str) -> Result<String, CryptoError> {
    // 生成随机盐
    let salt = SaltString::generate(&mut OsRng);
    
    // 使用 Argon2id 算法（推荐用于密码哈希）
    let argon2 = Argon2::default();
    
    // 生成密码哈希
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| CryptoError::HashError(e.to_string()))?;
    
    Ok(password_hash.to_string())
}

/// 验证密码是否匹配哈希值
/// 
/// # 参数
/// - `password`: 明文密码
/// - `hash`: PHC 格式的密码哈希字符串
/// 
/// # 返回
/// - `Ok(true)`: 密码匹配
/// - `Ok(false)`: 密码不匹配
/// - `Err(CryptoError)`: 验证过程出错
pub fn verify_password(password: &str, hash: &str) -> Result<bool, CryptoError> {
    // 解析哈希字符串
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| CryptoError::VerifyError(e.to_string()))?;
    
    // 验证密码
    let argon2 = Argon2::default();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        
        // 验证哈希格式
        assert!(hash.starts_with("$argon2"));
        assert!(hash.len() > 50);
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();
        
        // 验证正确密码
        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "correct_password";
        let hash = hash_password(password).unwrap();
        
        // 验证错误密码
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("any_password", "invalid_hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let password = "same_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        
        // 由于使用随机盐，相同密码的哈希应该不同
        assert_ne!(hash1, hash2);
        
        // 但都应该能验证通过
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
}
