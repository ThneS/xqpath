//! # 配置管理系统
//!
//! 提供统一的配置文件管理，支持YAML和TOML格式，包含配置模板、配置文件版本控制等功能。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "config-management")]
use std::fs;

#[cfg(feature = "config-management")]
use dirs;

/// 配置管理器主要结构
#[derive(Debug, Clone)]
#[cfg(feature = "config-management")]
pub struct ConfigManager {
    config_dir: PathBuf,
    current_config: XQPathConfig,
    profiles: HashMap<String, XQPathConfig>,
    active_profile: String,
}

/// XQPath主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XQPathConfig {
    /// 调试相关配置
    pub debug: DebugConfig,
    /// 性能相关配置
    pub performance: PerformanceConfig,
    /// 路径相关配置
    pub paths: PathsConfig,
    /// 功能特性配置
    pub features: FeaturesConfig,
}

/// 调试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// 日志级别
    pub level: String,
    /// 输出目标
    pub output: String,
    /// 日志文件路径
    pub file: Option<PathBuf>,
    /// 是否启用计时
    pub timing: bool,
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 内存限制
    pub memory_limit: String,
    /// 超时时间
    pub timeout: String,
    /// 缓存大小
    pub cache_size: u32,
    /// 并行任务数
    pub parallel_jobs: u32,
}

/// 路径配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// 缓存目录
    pub cache_dir: PathBuf,
    /// 日志目录
    pub log_dir: PathBuf,
    /// 配置目录
    pub config_dir: PathBuf,
}

/// 功能特性配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    /// 彩色输出
    pub colored_output: bool,
    /// 交互模式
    pub interactive_mode: bool,
    /// 自动备份
    pub auto_backup: bool,
}

/// 配置操作结果
pub type ConfigResult<T> = Result<T, ConfigError>;

/// 配置错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("配置文件不存在: {0}")]
    FileNotFound(PathBuf),

    #[error("配置文件解析错误: {0}")]
    ParseError(String),

    #[error("配置写入错误: {0}")]
    WriteError(String),

    #[error("无效的配置值: {key} = {value}")]
    InvalidValue { key: String, value: String },

    #[error("配置目录创建失败: {0}")]
    DirectoryCreationFailed(String),

    #[error("权限不足: {0}")]
    PermissionDenied(String),
}

impl Default for XQPathConfig {
    fn default() -> Self {
        Self {
            debug: DebugConfig {
                level: "info".to_string(),
                output: "stderr".to_string(),
                file: None,
                timing: false,
            },
            performance: PerformanceConfig {
                memory_limit: "1GB".to_string(),
                timeout: "30s".to_string(),
                cache_size: 1000,
                parallel_jobs: 4,
            },
            paths: PathsConfig {
                cache_dir: PathBuf::from("~/.xqpath/cache"),
                log_dir: PathBuf::from("~/.xqpath/logs"),
                config_dir: PathBuf::from("~/.xqpath"),
            },
            features: FeaturesConfig {
                colored_output: true,
                interactive_mode: false,
                auto_backup: true,
            },
        }
    }
}

#[cfg(feature = "config-management")]
impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> ConfigResult<Self> {
        let config_dir = Self::get_config_directory()?;
        let default_config = XQPathConfig::default();

        let mut manager = ConfigManager {
            config_dir,
            current_config: default_config.clone(),
            profiles: HashMap::new(),
            active_profile: "default".to_string(),
        };

        // 插入默认配置文件
        manager
            .profiles
            .insert("default".to_string(), default_config);

        // 尝试加载现有配置
        if let Ok(config) = manager.load_config() {
            manager.current_config = config;
        }

        Ok(manager)
    }

    /// 获取配置目录
    fn get_config_directory() -> ConfigResult<PathBuf> {
        dirs::config_dir()
            .map(|dir| dir.join(".xqpath"))
            .ok_or_else(|| {
                ConfigError::DirectoryCreationFailed(
                    "无法确定配置目录".to_string(),
                )
            })
    }

    /// 加载配置文件
    pub fn load_config(&self) -> ConfigResult<XQPathConfig> {
        let config_file = self.config_dir.join("config.yaml");

        if !config_file.exists() {
            return Ok(XQPathConfig::default());
        }

        let content = fs::read_to_string(&config_file)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        let config: XQPathConfig = serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        Ok(config)
    }

    /// 保存配置文件
    pub fn save_config(&self) -> ConfigResult<()> {
        // 确保配置目录存在
        fs::create_dir_all(&self.config_dir)
            .map_err(|e| ConfigError::DirectoryCreationFailed(e.to_string()))?;

        let config_file = self.config_dir.join("config.yaml");

        let content = serde_yaml::to_string(&self.current_config)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        fs::write(&config_file, content)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        Ok(())
    }

    /// 设置配置项
    pub fn set_config_value(
        &mut self,
        key: &str,
        value: &str,
    ) -> ConfigResult<()> {
        match key {
            "debug.level" => {
                if !["trace", "debug", "info", "warn", "error"].contains(&value)
                {
                    return Err(ConfigError::InvalidValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    });
                }
                self.current_config.debug.level = value.to_string();
            }
            "debug.timing" => {
                self.current_config.debug.timing =
                    value.parse().map_err(|_| ConfigError::InvalidValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    })?;
            }
            "performance.cache_size" => {
                self.current_config.performance.cache_size =
                    value.parse().map_err(|_| ConfigError::InvalidValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    })?;
            }
            "features.colored_output" => {
                self.current_config.features.colored_output =
                    value.parse().map_err(|_| ConfigError::InvalidValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    })?;
            }
            _ => {
                return Err(ConfigError::InvalidValue {
                    key: key.to_string(),
                    value: value.to_string(),
                });
            }
        }

        Ok(())
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &XQPathConfig {
        &self.current_config
    }

    /// 重置配置为默认值
    pub fn reset_config(&mut self) -> ConfigResult<()> {
        self.current_config = XQPathConfig::default();
        self.save_config()
    }

    /// 创建配置模板
    pub fn create_template(&self, name: &str) -> ConfigResult<()> {
        let template_file =
            self.config_dir.join(format!("templates/{name}.yaml"));

        if let Some(parent) = template_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ConfigError::DirectoryCreationFailed(e.to_string())
            })?;
        }

        let content = serde_yaml::to_string(&self.current_config)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        fs::write(&template_file, content)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        Ok(())
    }

    /// 创建配置配置文件
    pub fn create_profile(&mut self, name: &str) -> ConfigResult<()> {
        self.profiles
            .insert(name.to_string(), self.current_config.clone());

        let profile_file =
            self.config_dir.join(format!("profiles/{name}.yaml"));

        if let Some(parent) = profile_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ConfigError::DirectoryCreationFailed(e.to_string())
            })?;
        }

        let content = serde_yaml::to_string(&self.current_config)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        fs::write(&profile_file, content)
            .map_err(|e| ConfigError::WriteError(e.to_string()))?;

        Ok(())
    }

    /// 切换配置配置文件
    pub fn switch_profile(&mut self, name: &str) -> ConfigResult<()> {
        if let Some(config) = self.profiles.get(name) {
            self.current_config = config.clone();
            self.active_profile = name.to_string();
            Ok(())
        } else {
            // 尝试从文件加载配置文件
            let profile_file =
                self.config_dir.join(format!("profiles/{name}.yaml"));

            if profile_file.exists() {
                let content = fs::read_to_string(&profile_file)
                    .map_err(|e| ConfigError::ParseError(e.to_string()))?;

                let config: XQPathConfig = serde_yaml::from_str(&content)
                    .map_err(|e| ConfigError::ParseError(e.to_string()))?;

                self.current_config = config.clone();
                self.profiles.insert(name.to_string(), config);
                self.active_profile = name.to_string();
                Ok(())
            } else {
                Err(ConfigError::FileNotFound(profile_file))
            }
        }
    }

    /// 获取当前活动的配置文件名
    pub fn get_active_profile(&self) -> &str {
        &self.active_profile
    }

    /// 列出所有可用的配置文件
    pub fn list_profiles(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }
}

#[cfg(not(feature = "config-management"))]
pub fn get_default_config() -> XQPathConfig {
    XQPathConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = XQPathConfig::default();
        assert_eq!(config.debug.level, "info");
        assert_eq!(config.performance.cache_size, 1000);
        assert!(config.features.colored_output);
    }

    #[cfg(feature = "config-management")]
    #[test]
    fn test_config_serialization() {
        let config = XQPathConfig::default();
        let serialized = serde_yaml::to_string(&config).unwrap();
        let deserialized: XQPathConfig =
            serde_yaml::from_str(&serialized).unwrap();

        assert_eq!(config.debug.level, deserialized.debug.level);
    }
}
