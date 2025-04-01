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

// 生成签名的密钥 - 在实际应用中应该存储在安全的地方
const SIGNATURE_KEY: &str = "DrillSystemSecretKey2024";

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
    let signature = generate_signature(&json_data);
    
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
    
    let expected_signature = generate_signature(&json_data);
    
    if signature != expected_signature {
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

// 生成签名
fn generate_signature(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hasher.update(SIGNATURE_KEY.as_bytes());
    hex::encode(hasher.finalize())
} 