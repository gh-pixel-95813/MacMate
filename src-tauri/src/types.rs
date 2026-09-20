//! Tauri 命令响应格式与共享错误类型。
//!
//! 这些结构通过 serde 序列化后与前端共享,前端 TypeScript 可按相同形状解析。
//! `AppError` 实现了 `Into<tauri::ipc::InvokeError>`,因此 Tauri 命令可直接返回
//! `Result<T, AppError>`,错误信息会以字符串形式回传给前端。

use std::io;
use thiserror::Error;

/// 扫描项的安全等级,用于 Smart Selection 默认勾选策略。
///
/// - `safe`:缓存、临时文件、废纸篓等,默认勾选可清理。
/// - `caution`:日志、Application Support(卸载场景)等,默认不勾选,需手动展开。
/// - `danger`:Preferences / plist / Saved State 等,仅展示,不提供自动清理入口。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SafetyLevel {
    Safe,
    Caution,
    Danger,
}

/// 单个扫描结果项。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanItem {
    /// 稳定唯一标识,用于前端勾选与清理回调对齐。
    pub id: String,
    /// 文件 / 目录绝对路径。
    pub path: String,
    /// 占用字节数(目录则为递归累加)。
    pub size_bytes: u64,
    /// 最后修改时间(unix 秒)。
    pub modified_at: i64,
    /// 安全等级标签,驱动 Smart Selection。
    pub safety: SafetyLevel,
    /// 分类标识,如 `system_junk` / `uninstaller` / `large_files` / `privacy`。
    pub category: String,
    /// 可选的人类可读展示标签(如应用名、文件类型)。
    pub label: Option<String>,
}

/// 一次扫描的汇总结果。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub items: Vec<ScanItem>,
    /// 全部项的字节总和。
    pub total_size_bytes: u64,
    /// 实际遍历到的路径数(用于前端进度展示)。
    pub scanned_paths: u64,
}

/// 重复文件组:相同 SHA-256 的文件集合(由 Task 6 的 `scan_duplicates` 产生)。
///
/// `files` 按 mtime 升序排列,前端默认保留第一个(最早修改),其余勾选可清理。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    /// 组内共同的 SHA-256 哈希(十六进制)。
    pub hash: String,
    /// 组内每个文件的字节数(所有文件相同)。
    pub size_bytes: u64,
    /// 组内文件列表(按 mtime 升序)。
    pub files: Vec<ScanItem>,
}

/// 应用卸载器:单个关联路径(配置 / 缓存 / 日志等)。
///
/// 由 Task 5 的 `find_app_related` 产生,描述 .app bundle 在用户 Library 下的
/// 8 个标准关联位置(Preferences / Caches / Logs 等)之一。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelatedPath {
    /// 关联路径(绝对路径)。
    pub path: String,
    /// 占用字节数(目录则递归累加,文件则为文件大小,不存在则为 0)。
    pub size_bytes: u64,
    /// 路径是否存在(供前端灰显不存在的项)。
    pub exists: bool,
    /// 安全等级(由 `safety::classify` 标注,驱动卸载勾选策略)。
    pub safety: SafetyLevel,
}

/// 应用卸载器:扫描到的单个 .app bundle。
///
/// 由 Task 5 的 `scan_applications` 产生,聚合 .app 本体大小、bundle id、
/// 8 个关联位置清单、是否系统应用、是否运行中等信息,供前端展示与卸载。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppEntry {
    /// 应用名(.app 文件名去扩展名,如 `Safari`)。
    pub name: String,
    /// .app bundle 绝对路径(如 `/Applications/Safari.app`)。
    pub path: String,
    /// `CFBundleIdentifier`(解析 `Contents/Info.plist` 得到;失败则为 None)。
    pub bundle_id: Option<String>,
    /// 应用本体大小(.app bundle 递归累加字节数)。
    pub size_bytes: u64,
    /// 关联文件清单(8 个标准位置,每项含 exists / size / safety)。
    pub related: Vec<RelatedPath>,
    /// 是否为系统应用(`/System/Applications` 下,只读列出,不提供卸载)。
    pub is_system: bool,
    /// 应用是否正在运行(供前端禁用卸载按钮)。
    pub is_running: bool,
}

/// 清理成功的单项结果。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanedItem {
    pub id: String,
    pub path: String,
    pub size_bytes: u64,
}

/// 清理失败的单项结果。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedItem {
    pub id: String,
    pub path: String,
    pub reason: String,
}

/// 一次清理操作的汇总结果(成功与失败分离)。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanOutcome {
    pub success: Vec<CleanedItem>,
    pub failed: Vec<FailedItem>,
}

/// 隐私清理扫描项:浏览器隐私数据(Safari / Chrome / Firefox)或系统最近文档。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyTarget {
    /// 稳定唯一标识,如 `safari-history`、`chrome-default-cache`。
    pub id: String,
    /// 所属浏览器:`Safari` / `Chrome` / `Firefox` / `System`。
    pub browser: String,
    /// 人类可读标签,如 `History` / `Cookies` / `Cache`。
    pub label: String,
    /// 文件 / 目录绝对路径。
    pub path: String,
    /// 占用字节数。
    pub size_bytes: u64,
    /// 浏览器是否已安装(基于 .app bundle 路径检测)。
    pub is_installed: bool,
    /// 浏览器是否正在运行(sysinfo 进程检测)。
    pub is_running: bool,
    /// 安全等级。
    pub safety: SafetyLevel,
}

/// 浏览器运行状态。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserStatus {
    /// 浏览器名:`Safari` / `Chrome` / `Firefox`。
    pub browser: String,
    /// 是否正在运行。
    pub is_running: bool,
}

/// 磁盘使用概览,供仪表盘 DiskUsageWidget 调用。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskUsage {
    /// 磁盘总容量(字节)。
    pub total_bytes: u64,
    /// 已用容量(字节)。
    pub used_bytes: u64,
    /// 可用容量(字节)。
    pub available_bytes: u64,
    /// 挂载点路径,如 `/`。
    pub mount_point: String,
}

/// 清理历史记录单条(Task 8.4):一次清理操作落盘到 `~/.macmate/history.json`。
///
/// `module` 取值如 `system_junk` / `uninstaller` / `large_files` / `privacy`,
/// 用于在历史列表中区分来源模块。`freed_bytes` 为本次实际释放字节数
/// (success 项 size 之和),`success_count` / `failed_count` 记录成败计数。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanHistoryEntry {
    /// 发生时间(unix 秒)。
    pub timestamp: i64,
    /// 来源模块名,如 `system_junk`。
    pub module: String,
    /// 释放字节数。
    pub freed_bytes: u64,
    /// 成功项数。
    pub success_count: u32,
    /// 失败项数。
    pub failed_count: u32,
}

/// 应用配置(Task 9.3):落盘到 `~/.macmate/config.json`。
///
/// 记录用户在前端 Settings 页面中可调整的语言、主题、深度清理开关与
/// 大文件扫描阈值,启动时由前端 `get_config` 加载到 store,变更时由
/// `save_config` 持久化。文件不存在时 [`Default::default`] 返回安全默认值。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// 界面语言:`zh-CN` / `en-US`。
    pub locale: String,
    /// 主题模式:`system` / `light` / `dark`。
    pub theme: String,
    /// 是否开启深度清理(扫描系统级目录,清理需管理员授权)。
    pub deep_clean: bool,
    /// 大文件扫描阈值(MB),低于此大小的文件不进入大文件列表。
    pub large_file_threshold_mb: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            locale: "zh-CN".into(),
            theme: "system".into(),
            deep_clean: false,
            large_file_threshold_mb: 50,
        }
    }
}

/// 系统环境信息(供 Settings 关于区与日志上报 Issue 正文头使用)。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    /// macOS 产品版本号(如 `14.5`);非 macOS 平台为 `unknown`。
    pub os_version: String,
    /// CPU 架构(`aarch64` / `x86_64` 等)。
    pub arch: String,
    /// 主机名。
    pub hostname: String,
    /// 根挂载点磁盘总容量(字节)。
    pub disk_total_bytes: u64,
    /// 根挂载点磁盘可用容量(字节)。
    pub disk_available_bytes: u64,
}

/// Tauri 命令统一错误类型。
///
/// 注意:不实现 `serde::Serialize`,而是显式实现 `Into<InvokeError>`,
/// 这样既满足 Tauri 2 `Result<T, E: Into<InvokeError>>` 的命令返回约束,
/// 又避免与 `impl<T: Serialize> From<T> for InvokeError` 这条 blanket impl 冲突。
#[derive(Debug, Error)]
pub enum AppError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("directory walk error")]
    Walk,
    #[error("plist parse error")]
    Plist,
    #[error("sudo error: {0}")]
    Sudo(String),
    #[error("trash error: {0}")]
    Trash(String),
    #[error("not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(String),
}

/// 让 Tauri 命令可以直接返回 `Result<T, AppError>`:
/// 错误会以字符串(`error.to_string()`)形式回传给前端 reject 回调。
impl From<AppError> for tauri::ipc::InvokeError {
    fn from(err: AppError) -> Self {
        Self::from_error(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_level_serde_kebab_case() {
        let json = serde_json::to_string(&SafetyLevel::Caution).unwrap();
        assert_eq!(json, "\"caution\"");
        let parsed: SafetyLevel = serde_json::from_str("\"danger\"").unwrap();
        assert_eq!(parsed, SafetyLevel::Danger);
        let parsed_safe: SafetyLevel = serde_json::from_str("\"safe\"").unwrap();
        assert_eq!(parsed_safe, SafetyLevel::Safe);
    }

    #[test]
    fn scan_item_round_trip() {
        let item = ScanItem {
            id: "abc".into(),
            path: "/tmp/x".into(),
            size_bytes: 1024,
            modified_at: 1_700_000_000,
            safety: SafetyLevel::Safe,
            category: "system_junk".into(),
            label: Some("cache".into()),
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: ScanItem = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "abc");
        assert_eq!(back.size_bytes, 1024);
        assert_eq!(back.safety, SafetyLevel::Safe);
        assert!(back.label.is_some());
    }

    #[test]
    fn clean_outcome_default_empty() {
        let outcome = CleanOutcome {
            success: vec![],
            failed: vec![],
        };
        let json = serde_json::to_string(&outcome).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("failed"));
    }

    #[test]
    fn disk_usage_round_trip() {
        let du = DiskUsage {
            total_bytes: 500_000_000_000,
            used_bytes: 250_000_000_000,
            available_bytes: 250_000_000_000,
            mount_point: "/".into(),
        };
        let json = serde_json::to_string(&du).unwrap();
        let back: DiskUsage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_bytes, 500_000_000_000);
        assert_eq!(back.mount_point, "/");
    }

    #[test]
    fn app_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "missing");
        let app_err = AppError::from(io_err);
        match app_err {
            AppError::Io(_) => {}
            other => panic!("expected Io, got {other:?}"),
        }
    }

    #[test]
    fn app_error_displays_message() {
        let err = AppError::Sudo("denied".into());
        assert!(err.to_string().contains("sudo error"));
        assert!(err.to_string().contains("denied"));
    }

    #[test]
    fn duplicate_group_serde_camel_case() {
        let group = DuplicateGroup {
            hash: "abc123".into(),
            size_bytes: 4096,
            files: vec![ScanItem {
                id: "id1".into(),
                path: "/tmp/x".into(),
                size_bytes: 4096,
                modified_at: 1_700_000_000,
                safety: SafetyLevel::Caution,
                category: "Large File".into(),
                label: Some("bin".into()),
            }],
        };
        let json = serde_json::to_string(&group).unwrap();
        // camelCase 字段
        assert!(json.contains("\"sizeBytes\""));
        assert!(!json.contains("\"size_bytes\""));
        assert!(json.contains("\"hash\""));
        assert!(json.contains("\"files\""));
        let back: DuplicateGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(back.hash, "abc123");
        assert_eq!(back.size_bytes, 4096);
        assert_eq!(back.files.len(), 1);
    }

    #[test]
    fn app_config_default_matches_spec() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.locale, "zh-CN");
        assert_eq!(cfg.theme, "system");
        assert!(!cfg.deep_clean);
        assert_eq!(cfg.large_file_threshold_mb, 50);
    }

    #[test]
    fn app_config_serde_camel_case_round_trip() {
        let cfg = AppConfig {
            locale: "en-US".into(),
            theme: "dark".into(),
            deep_clean: true,
            large_file_threshold_mb: 128,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        // camelCase 字段
        assert!(json.contains("\"deepClean\""));
        assert!(json.contains("\"largeFileThresholdMb\""));
        assert!(!json.contains("\"deep_clean\""));
        let back: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.locale, "en-US");
        assert_eq!(back.theme, "dark");
        assert!(back.deep_clean);
        assert_eq!(back.large_file_threshold_mb, 128);
    }
}
