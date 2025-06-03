use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt;
use sysinfo::{CpuExt, System, SystemExt};
use uuid::Uuid;

#[derive(Debug)]
pub enum MachineIdError {
    SystemInfoError(String),
    HashError(String),
}

impl fmt::Display for MachineIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MachineIdError::SystemInfoError(e) => write!(f, "获取系统信息错误: {}", e),
            MachineIdError::HashError(e) => write!(f, "计算哈希值错误: {}", e),
        }
    }
}

impl Error for MachineIdError {}

/// 获取当前机器的唯一标识符
pub fn get_machine_id() -> Result<String, MachineIdError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // 收集系统信息
    let hostname = sys.host_name().unwrap_or_else(|| "unknown".to_string());
    let os_name = sys.name().unwrap_or_else(|| "unknown".to_string());
    let os_version = sys.os_version().unwrap_or_else(|| "unknown".to_string());
    let kernel_version = sys
        .kernel_version()
        .unwrap_or_else(|| "unknown".to_string());

    // 收集硬件信息
    let cpu_brand = sys.global_cpu_info().brand().to_string();
    let cpu_cores = sys.physical_core_count().unwrap_or(0).to_string();

    // 获取系统UUID（如果可用）
    let system_uuid = match Uuid::parse_str(&sys.host_name().unwrap_or_default()) {
        Ok(uuid) => uuid.to_string(),
        Err(_) => "unknown".to_string(),
    };

    // 组合所有信息
    let machine_info = format!(
        "{}:{}:{}:{}:{}:{}:{}",
        hostname, os_name, os_version, kernel_version, cpu_brand, cpu_cores, system_uuid
    );

    // 计算SHA-256哈希值
    let mut hasher = Sha256::new();
    hasher.update(machine_info.as_bytes());
    let result = hasher.finalize();

    // 转换为十六进制字符串，取前32个字符作为机器码
    let hex_string = format!("{:x}", result);
    let machine_id = hex_string.chars().take(32).collect::<String>();

    Ok(machine_id)
}
