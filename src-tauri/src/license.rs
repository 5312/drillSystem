use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use base64::{engine::general_purpose, Engine as _};
use uuid::Uuid;
use rsa::{Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
use rand::rngs::OsRng;
use pkcs8::{DecodePrivateKey, DecodePublicKey};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseInfo {
    pub license_id: String,
    pub customer_name: String,
    pub customer_email: String,
    pub issue_date: DateTime<Utc>,
    pub expiry_date: DateTime<Utc>,
    pub features: Vec<String>,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseValidationResult {
    pub is_valid: bool,
    pub info: Option<LicenseInfo>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseDatabase {
    pub licenses: Vec<LicenseInfo>,
}

#[derive(Debug)]
pub enum LicenseError {
    SerializationError(String),
    ValidationError(String),
    ExpiredLicense,
    InvalidSignature,
    FileError(String),
}

impl fmt::Display for LicenseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LicenseError::SerializationError(e) => write!(f, "序列化错误: {}", e),
            LicenseError::ValidationError(e) => write!(f, "验证错误: {}", e),
            LicenseError::ExpiredLicense => write!(f, "许可证已过期"),
            LicenseError::InvalidSignature => write!(f, "无效的许可证签名"),
            LicenseError::FileError(e) => write!(f, "文件操作错误: {}", e),
        }
    }
}

impl Error for LicenseError {}

// RSA密钥 - 实际应用中，私钥应存储在安全的地方，不应硬编码
// 这里使用的是测试密钥，生产环境请生成新的密钥对
const PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDEgxwQMKHnYVxB
2ZkF5awrWTCLGNB8E6bx+KKHT2RGJG7avIg2VCPmW/+e5gbf1FIJFowS07cWSM0e
o6rQdLyfP6J7/D1RNIxVy8/f636P1kXMqtpQeGjQ2gimDG4yhZMVZjF1iI3vZQUC
/V4O5VFdOXKCLzNcD0qK9a5cO6jgQrLLKL9Ryc8Dh6SQJLtNnwjMDmB5D7hgPcS1
R9UCSMUQpeFWJzQOJiKCZtA2m+BG4KMqtS1Bw6hevNwTWVVdnJBCZqcbQCKFTHwl
2UfcFGkW+hwRKgBL0FbzHrBDRMVAVYJRWiWtRMdeqPImgeGU48Aaa4kOyvMpywWj
wMG+9XETAgMBAAECggEATBtPhAo0vhF1nJMtPL6hnDJpBKS0qkYCIsoNVYlE9QkQ
SWUHWGbCgKYXbejw2K/pkZXITVygvNVIeN0sCbLfMxQYGqt8XCfZnFKYMPpYzYTK
LnrvFELktbVK29dn2PSYUy1Kjteb4JeHqcLAyJiQPQip5JKNZXj+jYOVKNeEgYdA
vk9UoJxvO4H8XtU7j0pHxRcBPzN/laIYU9yDjWWBKRmRbVtLCwEWZ/K5FMZvHWH5
x60YrDYS/vCHd8AxiSmS8ewWg4xkVRYHHFHXJaGXcHjKv3BL4HMUfzf/5Smm/OBd
rLVxmrYJ6ykU8/1zYmqxKOLsHa0KF2mB+ZsM8IbqoQKBgQDncyVdqIPXGfxA/fWK
2qFfQBvxT7CERWDuTR+NPl4qFzKEVJgk7jN77vHEYnVUS1DAEZ5fMUmi0XlsurFV
9I4Bv9Zkxe9GO43/QUHnxQIPcZA59sNTOLOcDsyjQE6AKkwBi9NaqUWmKW+StH02
xdHLPOVSHfCGDv6yBv8LB3MZkQKBgQDZM6We+Wi7DAiQGpShzEiIvMWVAuEwbLft
Kd/XuExnL5x0ScYORjzNGytMY0XQFggwFfxXZQyYfLgER+UxT8kU9zOc9Yhvq2ZN
fDWdK5iA6q0MlqTihUNW3TohMFzpyXO7I96l5z+wIppHRbDMCenBVw+aCaVlINXQ
6o7WIm9+owKBgQCLRv/Y9Y+BR4U1ZgEUGnVRYTZ0vvKYPYn4QK9rMDSG1sBDQk5g
1qSsY1xDgWsUB6QSGjjtHCyxYX1I3vwFCPsxNc5SK6FUbUTzI16FMUu9oV7X3ILc
qU7Vngb+bn7Ai5n6EgbND9ITrP2Z55c5JK5ttvZYbvMK6X3z3JvvuAzGsQKBgE7g
A+lz3XGhL8eY9Pt9Z9YFPxJKYtRbKxMKx0p5B/4y3IOyEwfW1BCDHdpR1xyVQzWm
Bkqf5XWcAAOltExlRYNACLbeA1KHvRrEJW7XCCwxPYO+y4n7dPvX0wVT7PKgIcDw
IWpUgUsHLrRPOe4l4F2spxmD3eV8sSFXx4ZdFdwrAoGBANRCj6TZhHeduZGAy9ej
J9mIlZOQlkZ+usEdAajLjdGLcfE68wwjRKp3Q11Akh0p8+MFWeVYRmXArJdnkSbF
2JIoFGEDKrpEe9iDGQ0UgjRlNy3Zic4NnFRNSVdwZa4X5hxMK5qDbryGXSXnxGAL
GVFiJuZTKLEE3r/8zB8fpU+i
-----END PRIVATE KEY-----"#;

const PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAxIMcEDCh52FcQdmZBeWs
K1kwixjQfBOm8fiih09kRiRu2ryINlQj5lv/nuYG39RSCRaMEtO3FkjNHqOq0HS8
nz+ie/w9UTSMVcvP3+t+j9ZFzKraUHho0NoIpgxuMoWTFWYxdYiN72UFAv1eDuVR
XTlygi8zXA9KivWuXDuo4EKyyyi/UcnPA4ekkCS7TZ8IzA5geQ+4YD3EtUfVAkjF
EKXhVic0DiYigmbQNpvgRuCjKrUtQcOoXrzcE1lVXZyQQmanG0AihUx8JdlH3BRp
FvocESoAS9BW8x6wQ0TFQFWCUVolrUTHXqjyJoHhlOPAGmuJDsrzKcsFo8DBvvVx
EwIDAQAB
-----END PUBLIC KEY-----"#;

// 许可证数据库文件路径
fn get_license_db_path() -> PathBuf {
    let app_dir = if cfg!(target_os = "windows") {
        let app_data = std::env::var("APPDATA").expect("无法获取APPDATA环境变量");
        PathBuf::from(app_data).join("drilling-system")
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").expect("无法获取HOME环境变量");
        PathBuf::from(home).join("Library").join("Application Support").join("drilling-system")
    } else {
        // Linux
        let home = std::env::var("HOME").expect("无法获取HOME环境变量");
        PathBuf::from(home).join(".config").join("drilling-system")
    };
    
    // 确保目录存在
    fs::create_dir_all(&app_dir).expect("无法创建应用数据目录");
    
    app_dir.join("licenses.json")
}

// 加载许可证数据库
fn load_license_db() -> Result<LicenseDatabase, LicenseError> {
    let db_path = get_license_db_path();
    
    if !db_path.exists() {
        return Ok(LicenseDatabase { licenses: vec![] });
    }
    
    let mut file = File::open(&db_path)
        .map_err(|e| LicenseError::FileError(format!("打开数据库文件失败: {}", e)))?;
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| LicenseError::FileError(format!("读取数据库文件失败: {}", e)))?;
    
    serde_json::from_str(&contents)
        .map_err(|e| LicenseError::SerializationError(format!("解析数据库失败: {}", e)))
}

// 保存许可证数据库
fn save_license_db(db: &LicenseDatabase) -> Result<(), LicenseError> {
    let db_path = get_license_db_path();
    
    let json = serde_json::to_string_pretty(db)
        .map_err(|e| LicenseError::SerializationError(format!("序列化数据库失败: {}", e)))?;
    
    let mut file = File::create(&db_path)
        .map_err(|e| LicenseError::FileError(format!("创建数据库文件失败: {}", e)))?;
    
    file.write_all(json.as_bytes())
        .map_err(|e| LicenseError::FileError(format!("写入数据库失败: {}", e)))?;
    
    Ok(())
}

// 生成RSA签名
fn generate_signature(data: &str) -> Result<String, LicenseError> {
    // 从PEM格式解析私钥
    let private_key = RsaPrivateKey::from_pkcs8_pem(PRIVATE_KEY_PEM)
        .map_err(|e| LicenseError::ValidationError(format!("解析私钥失败: {}", e)))?;
    
    // 计算数据的SHA-256哈希值
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let hashed = hasher.finalize();
    
    // 使用私钥对哈希值进行签名
    let signature = private_key.sign_with_rng(&mut OsRng, Pkcs1v15Sign::new::<Sha256>(), &hashed)
        .map_err(|e| LicenseError::ValidationError(format!("签名失败: {}", e)))?;
    
    // 返回Base64编码的签名
    Ok(general_purpose::STANDARD.encode(&signature))
}

// 验证RSA签名
fn verify_signature(data: &str, signature_base64: &str) -> Result<bool, LicenseError> {
    // 从PEM格式解析公钥
    let public_key = RsaPublicKey::from_public_key_pem(PUBLIC_KEY_PEM)
        .map_err(|e| LicenseError::ValidationError(format!("解析公钥失败: {}", e)))?;
    
    // 计算数据的SHA-256哈希值
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let hashed = hasher.finalize();
    
    // 解码Base64签名
    let signature = general_purpose::STANDARD.decode(signature_base64)
        .map_err(|e| LicenseError::ValidationError(format!("解码签名失败: {}", e)))?;
    
    // 验证签名
    let result = public_key.verify(Pkcs1v15Sign::new::<Sha256>(), &hashed, &signature);
    
    // 返回验证结果
    match result {
        Ok(_) => Ok(true),
        Err(_) => Ok(false)
    }
}

pub fn generate_license(
    customer_name: &str,
    customer_email: &str,
    expiry_days: u32,
    features: Vec<String>,
) -> Result<String, LicenseError> {
    let now = Utc::now();
    let expiry = now + Duration::days(expiry_days as i64);
    
    let license_id = Uuid::new_v4().to_string();
    
    // 创建不包含签名的许可证信息
    let license_data = LicenseInfo {
        license_id,
        customer_name: customer_name.to_string(),
        customer_email: customer_email.to_string(),
        issue_date: now,
        expiry_date: expiry,
        features,
        signature: String::new(), // 暂时为空
    };
    
    // 序列化为JSON
    let json_data = serde_json::to_string(&license_data)
        .map_err(|e| LicenseError::SerializationError(e.to_string()))?;
    
    // 生成签名
    let signature = generate_signature(&json_data)?;
    
    // 更新许可证信息，包含签名
    let license_with_signature = LicenseInfo {
        signature,
        ..license_data
    };
    
    // 保存到数据库
    let mut db = load_license_db()?;
    db.licenses.push(license_with_signature.clone());
    save_license_db(&db)?;
    
    // 序列化并编码为Base64
    let final_json = serde_json::to_string(&license_with_signature)
        .map_err(|e| LicenseError::SerializationError(e.to_string()))?;
    
    Ok(general_purpose::STANDARD.encode(final_json))
}

pub fn validate_license(license_key: &str) -> Result<LicenseValidationResult, LicenseError> {
    // 解码Base64
    let decoded = general_purpose::STANDARD.decode(license_key)
        .map_err(|e| LicenseError::ValidationError(format!("Base64解码失败: {}", e)))?;
    
    // 解析JSON
    let license_data: LicenseInfo = serde_json::from_slice(&decoded)
        .map_err(|e| LicenseError::ValidationError(format!("JSON解析失败: {}", e)))?;
    
    // 验证签名
    let signature = license_data.signature.clone();
    let mut license_for_verification = license_data.clone();
    license_for_verification.signature = String::new();
    
    let json_data = serde_json::to_string(&license_for_verification)
        .map_err(|e| LicenseError::SerializationError(e.to_string()))?;
    
    let is_signature_valid = verify_signature(&json_data, &signature)?;
    
    if !is_signature_valid {
        return Ok(LicenseValidationResult {
            is_valid: false,
            info: None,
            message: "许可证签名无效".to_string(),
        });
    }
    
    // 检查过期时间
    let now = Utc::now();
    if license_data.expiry_date < now {
        return Ok(LicenseValidationResult {
            is_valid: false,
            info: Some(license_data),
            message: "许可证已过期".to_string(),
        });
    }
    
    // 有效许可证
    Ok(LicenseValidationResult {
        is_valid: true,
        info: Some(license_data),
        message: "许可证有效".to_string(),
    })
}

// 获取所有许可证
pub fn get_all_licenses() -> Result<Vec<LicenseInfo>, LicenseError> {
    let db = load_license_db()?;
    Ok(db.licenses)
}

// 导出公钥
pub fn export_public_key() -> String {
    PUBLIC_KEY_PEM.to_string()
} 