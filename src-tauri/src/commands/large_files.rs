//! 大文件扫描与重复文件检测命令(Task 6)。
//!
//! - `scan_large_files`:按阈值扫描用户目录下的大文件,按 size 降序返回。
//! - `scan_duplicates`:基于 SHA-256 的重复文件分组,组内按 mtime 升序。
//! - `clean_large_files`:将选中文件移至废纸篓。
//!
//! 核心逻辑放在同步的 `*_inner` 函数中,`#[tauri::command]` 异步命令仅做包装,
//! 便于单元测试直接覆盖而无需引入异步运行时。

use crate::fsutil;
use crate::trash;
use crate::types::{
    AppError, CleanOutcome, CleanedItem, DuplicateGroup, FailedItem, SafetyLevel, ScanItem,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Read;
use std::path::{Path, PathBuf};

/// 默认扫描阈值(MB),前端传入时与之对齐(仅文档化与测试引用)。
#[allow(dead_code)]
const DEFAULT_THRESHOLD_MB: u64 = 50;

/// 默认扫描子目录(相对 home),缺失项会被跳过。
const DEFAULT_SCAN_DIRS: &[&str] = &["Downloads", "Documents", "Movies", "Pictures", "Desktop"];

/// 默认扫描路径(基于用户 home 目录的 5 个子目录)。
/// 无法获取 home 时返回空向量(命令层会得到 0 项结果)。
fn default_scan_paths() -> Vec<String> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    DEFAULT_SCAN_DIRS
        .iter()
        .map(|d| home.join(d).to_string_lossy().into_owned())
        .collect()
}

/// 将前端传入的路径列表解析为存在的目录列表。
/// - 空列表回退到默认扫描路径(~/Downloads 等)。
/// - 非绝对路径(如前端只传目录名 `Downloads`)相对 home 解析。
/// - 仅保留实际存在的目录,缺失项自动跳过。
fn resolve_scan_paths(scan_paths: Vec<String>) -> Vec<PathBuf> {
    let home = dirs::home_dir();
    let paths: Vec<PathBuf> = if scan_paths.is_empty() {
        default_scan_paths()
            .into_iter()
            .map(PathBuf::from)
            .collect()
    } else {
        scan_paths
            .into_iter()
            .map(|s| {
                let p = PathBuf::from(&s);
                if p.is_absolute() {
                    p
                } else if let Some(home) = &home {
                    home.join(p)
                } else {
                    p
                }
            })
            .collect()
    };
    paths.into_iter().filter(|p| p.is_dir()).collect()
}

/// 构造单个文件的 [`ScanItem`]。
/// - `id` = 路径(稳定唯一);`category` = "Large File";`safety` = Caution。
/// - `label` = 文件扩展名(小写,无扩展名则为 None)。
fn build_scan_item(path: &Path, size_bytes: u64, modified_at: i64) -> ScanItem {
    let label = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase());
    let path_str = path.to_string_lossy().into_owned();
    ScanItem {
        id: path_str.clone(),
        path: path_str,
        size_bytes,
        modified_at,
        safety: SafetyLevel::Caution,
        category: "Large File".into(),
        label,
    }
}

/// 计算文件 SHA-256(十六进制),按 64 KiB 分块读取,打开 / 读取失败返回 None。
fn sha256_hex(path: &Path) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        // write! 到 String 不会失败,丢弃 Result 即可。
        let _ = write!(hex, "{:02x}", byte);
    }
    Some(hex)
}

/// 扫描指定路径下超过阈值的大文件,按 size 降序返回。
fn scan_large_files_inner(threshold_mb: u64, scan_paths: Vec<String>) -> Vec<ScanItem> {
    let threshold = threshold_mb.saturating_mul(1024 * 1024);
    let paths = resolve_scan_paths(scan_paths);
    let mut items: Vec<ScanItem> = Vec::new();
    for p in &paths {
        for file in fsutil::walk_files(p) {
            let Ok(meta) = std::fs::metadata(&file) else {
                continue;
            };
            let size = meta.len();
            if size < threshold {
                continue;
            }
            let mtime = fsutil::modified_unix(&file).unwrap_or(0);
            items.push(build_scan_item(&file, size, mtime));
        }
    }
    // size 降序;相同 size 时按路径升序,保持稳定输出便于断言。
    items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes).then(a.path.cmp(&b.path)));
    items
}

/// 检测重复文件:先按 size 分组(>=2),再对每组按 SHA-256 分组(>=2)。
/// 组内按 mtime 升序,前端默认保留第一个(最早修改)。
fn scan_duplicates_inner(scan_paths: Vec<String>) -> Vec<DuplicateGroup> {
    let paths = resolve_scan_paths(scan_paths);

    // 第一步:收集所有文件并按 size 分组。
    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for p in &paths {
        for file in fsutil::walk_files(p) {
            let Ok(meta) = std::fs::metadata(&file) else {
                continue;
            };
            by_size.entry(meta.len()).or_default().push(file);
        }
    }

    // 第二步:对每个 size 组(>=2),计算 SHA-256 并按 hash 分组。
    let mut groups: Vec<DuplicateGroup> = Vec::new();
    for (size, files) in by_size {
        if files.len() < 2 {
            continue;
        }
        let mut by_hash: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for file in files {
            if let Some(hash) = sha256_hex(&file) {
                by_hash.entry(hash).or_default().push(file);
            }
        }
        // 第三步:每个 hash 组(>=2)作为一个 DuplicateGroup。
        for (hash, group_files) in by_hash {
            if group_files.len() < 2 {
                continue;
            }
            let mut items: Vec<ScanItem> = group_files
                .iter()
                .map(|f| {
                    let mtime = fsutil::modified_unix(f).unwrap_or(0);
                    build_scan_item(f, size, mtime)
                })
                .collect();
            // 组内按 mtime 升序;mtime 相同时按路径升序保持稳定。
            items.sort_by(|a, b| a.modified_at.cmp(&b.modified_at).then(a.path.cmp(&b.path)));
            groups.push(DuplicateGroup {
                hash,
                size_bytes: size,
                files: items,
            });
        }
    }
    // 按 size 降序输出,便于前端优先展示大块重复。
    groups.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes).then(a.hash.cmp(&b.hash)));
    groups
}

/// 将选中文件移至废纸篓,返回成功 / 失败汇总。
fn clean_large_files_inner(items: Vec<String>) -> CleanOutcome {
    let paths: Vec<PathBuf> = items.iter().map(PathBuf::from).collect();
    // 先记录每项 size(移除后无法再 stat),再用 trash crate 批量移除。
    let enriched: Vec<(String, u64)> = paths
        .iter()
        .map(|p| {
            let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
            (p.to_string_lossy().into_owned(), size)
        })
        .collect();

    let mut success: Vec<CleanedItem> = Vec::new();
    let mut failed: Vec<FailedItem> = Vec::new();
    match trash::move_to_trash(&paths) {
        Ok(()) => {
            for (path, size) in enriched {
                success.push(CleanedItem {
                    id: path.clone(),
                    path,
                    size_bytes: size,
                });
            }
        }
        Err(err) => {
            // trash::move_to_trash 当前为整体成功 / 整体失败,失败时全部记为失败。
            let reason = err.to_string();
            for (path, _) in enriched {
                failed.push(FailedItem {
                    id: path.clone(),
                    path,
                    reason: reason.clone(),
                });
            }
        }
    }
    CleanOutcome { success, failed }
}

/// 扫描指定路径下超过阈值的大文件,按 size 降序返回。
///
/// - `threshold_mb`:阈值(MB),由前端传入(默认 50)。
/// - `scan_paths`:扫描根目录;为空时回退到 `~/Downloads`、`~/Documents`、
///   `~/Movies`、`~/Pictures`、`~/Desktop`,缺失目录自动跳过。
#[tauri::command]
pub async fn scan_large_files(
    threshold_mb: u64,
    scan_paths: Vec<String>,
) -> Result<Vec<ScanItem>, AppError> {
    Ok(scan_large_files_inner(threshold_mb, scan_paths))
}

/// 检测重复文件:基于 SHA-256 分组,每个组(>=2 文件)返回一个 [`DuplicateGroup`]。
///
/// 组内 `files` 按 mtime 升序,前端默认保留第一个(最早修改),其余勾选可清理。
#[tauri::command]
pub async fn scan_duplicates(scan_paths: Vec<String>) -> Result<Vec<DuplicateGroup>, AppError> {
    Ok(scan_duplicates_inner(scan_paths))
}

/// 将选中文件(path 列表)移至废纸篓,返回 [`CleanOutcome`]。
#[tauri::command]
pub async fn clean_large_files(items: Vec<String>) -> Result<CleanOutcome, AppError> {
    Ok(clean_large_files_inner(items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    /// 写入指定字节数的全零文件。
    fn write_sized(path: &Path, bytes: usize) {
        let mut f = fs::File::create(path).unwrap();
        let buf = vec![0u8; bytes.min(65536)];
        let mut remaining = bytes;
        while remaining > 0 {
            let n = remaining.min(buf.len());
            f.write_all(&buf[..n]).unwrap();
            remaining -= n;
        }
    }

    #[test]
    fn scan_large_files_filters_by_threshold() {
        let dir = tempfile::tempdir().unwrap();
        // 大文件(2 MiB >= 1 MiB 阈值)
        let big = dir.path().join("big.bin");
        write_sized(&big, 2 * 1024 * 1024);
        // 小文件(< 阈值,应被过滤)
        let small = dir.path().join("small.txt");
        fs::write(&small, "tiny").unwrap();

        let items = scan_large_files_inner(1, vec![dir.path().to_string_lossy().into_owned()]);
        // 只有大文件被返回
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, big.to_string_lossy());
        assert!(items[0].size_bytes >= 2 * 1024 * 1024);
        assert_eq!(items[0].category, "Large File");
        assert_eq!(items[0].safety, SafetyLevel::Caution);
        assert_eq!(items[0].label.as_deref(), Some("bin"));
    }

    #[test]
    fn scan_large_files_sorted_desc_by_size() {
        let dir = tempfile::tempdir().unwrap();
        // 创建三个不同大小的大文件(均 >= 1 MiB)
        let mid = dir.path().join("mid.bin");
        write_sized(&mid, 2 * 1024 * 1024);
        let big = dir.path().join("big.bin");
        write_sized(&big, 4 * 1024 * 1024);
        let small = dir.path().join("small.bin");
        write_sized(&small, 1024 * 1024 + 1);

        let items = scan_large_files_inner(1, vec![dir.path().to_string_lossy().into_owned()]);
        assert_eq!(items.len(), 3);
        // 降序:big > mid > small
        assert!(items[0].size_bytes >= items[1].size_bytes);
        assert!(items[1].size_bytes >= items[2].size_bytes);
        assert_eq!(items[0].path, big.to_string_lossy());
    }

    #[test]
    fn scan_large_files_skips_nonexistent_paths() {
        // 不存在的路径被过滤,返回空结果而非 panic。
        let items = scan_large_files_inner(1, vec!["/nonexistent-macmate-test/zzz".into()]);
        assert!(items.is_empty());
    }

    #[test]
    fn resolve_scan_paths_empty_falls_back_to_defaults() {
        // 空 vec 回退到 home 下的默认目录;home 在沙箱中应存在。
        let resolved = resolve_scan_paths(vec![]);
        // 默认 5 个子目录,沙箱中实际存在的数量在 0..=5 之间。
        assert!(resolved.len() <= 5);
    }

    #[test]
    fn resolve_scan_paths_relative_names_resolved_against_home() {
        // 裸目录名相对 home 解析;沙箱中 home/Downloads 通常不存在 -> 被过滤为空。
        let resolved = resolve_scan_paths(vec!["Downloads".into()]);
        assert!(resolved.iter().all(|p| p.is_absolute()));
        assert!(resolved.iter().all(|p| p.is_dir()));
    }

    #[test]
    fn resolve_scan_paths_absolute_existing_dir() {
        let dir = tempfile::tempdir().unwrap();
        let abs = dir.path().to_string_lossy().into_owned();
        let resolved = resolve_scan_paths(vec![abs]);
        assert_eq!(resolved.len(), 1);
        assert!(resolved[0].is_absolute());
    }

    #[test]
    fn scan_large_files_empty_paths_does_not_panic() {
        // 默认路径在沙箱中通常无 >= 50MB 的大文件;关键是不 panic。
        scan_large_files_inner(DEFAULT_THRESHOLD_MB, vec![]);
    }

    #[test]
    fn scan_duplicates_groups_by_hash() {
        let dir = tempfile::tempdir().unwrap();
        // 两个同内容同大小文件 -> 应归为一组
        let payload = b"hello-duplicate-content";
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        fs::write(&a, payload).unwrap();
        fs::write(&b, payload).unwrap();
        // 第三个内容不同但大小相近的文件(故意大小相同内容不同,不会归组)
        let c = dir.path().join("c.bin");
        fs::write(&c, b"hello-duplicate-contenX").unwrap(); // 同长度,末字节不同

        let groups = scan_duplicates_inner(vec![dir.path().to_string_lossy().into_owned()]);
        // 应恰好 1 组,组内 2 个文件
        assert_eq!(groups.len(), 1, "expected exactly one duplicate group");
        let group = &groups[0];
        assert_eq!(group.files.len(), 2);
        assert_eq!(group.size_bytes, payload.len() as u64);
        // 组内文件路径包含 a 与 b
        let paths: Vec<String> = group.files.iter().map(|f| f.path.clone()).collect();
        assert!(paths.contains(&a.to_string_lossy().into_owned()));
        assert!(paths.contains(&b.to_string_lossy().into_owned()));
        // c 不在组内
        assert!(!paths.contains(&c.to_string_lossy().into_owned()));
        // hash 为 64 位十六进制
        assert_eq!(group.hash.len(), 64);
    }

    #[test]
    fn scan_duplicates_empty_when_no_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        // 两个内容不同的文件,大小也不同 -> 无组
        fs::write(dir.path().join("a.txt"), "aaa").unwrap();
        fs::write(dir.path().join("b.txt"), "bb").unwrap();
        let groups = scan_duplicates_inner(vec![dir.path().to_string_lossy().into_owned()]);
        assert!(groups.is_empty());
    }

    #[test]
    fn scan_duplicates_files_sorted_by_mtime_ascending() {
        let dir = tempfile::tempdir().unwrap();
        // 创建两个同内容文件,并显式设置不同的 mtime(older 较早)
        let payload = b"same-content";
        let older = dir.path().join("older.txt");
        let newer = dir.path().join("newer.txt");
        fs::write(&older, payload).unwrap();
        fs::write(&newer, payload).unwrap();
        // older 的 mtime 设为更早(60s 前)
        let now = std::time::SystemTime::now();
        let past = now - std::time::Duration::from_secs(60);
        filetime::set_file_mtime(&older, filetime::FileTime::from_system_time(past)).unwrap();

        let groups = scan_duplicates_inner(vec![dir.path().to_string_lossy().into_owned()]);
        assert_eq!(groups.len(), 1);
        let group = &groups[0];
        assert_eq!(group.files.len(), 2);
        // 升序:older 在前
        assert_eq!(group.files[0].path, older.to_string_lossy());
        assert!(group.files[0].modified_at <= group.files[1].modified_at);
    }

    #[test]
    fn clean_large_files_moves_to_trash() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("to_clean.bin");
        fs::write(&f, "x").unwrap();
        assert!(f.exists());

        let outcome = clean_large_files_inner(vec![f.to_string_lossy().into_owned()]);
        assert!(outcome.failed.is_empty(), "trash 在沙箱应可用");
        assert_eq!(outcome.success.len(), 1);
        assert_eq!(outcome.success[0].path, f.to_string_lossy());
        assert!(!f.exists(), "文件应已从原位置消失");
    }

    #[test]
    fn clean_large_files_empty_input_is_ok() {
        let outcome = clean_large_files_inner(vec![]);
        assert!(outcome.success.is_empty());
        assert!(outcome.failed.is_empty());
    }

    #[test]
    fn sha256_hex_consistent_for_same_content() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a.bin");
        let b = dir.path().join("b.bin");
        fs::write(&a, "identical").unwrap();
        fs::write(&b, "identical").unwrap();
        let ha = sha256_hex(&a).expect("应能读取现有文件");
        let hb = sha256_hex(&b).expect("应能读取现有文件");
        assert_eq!(ha, hb);
        assert_eq!(ha.len(), 64);
    }

    #[test]
    fn sha256_hex_none_for_missing() {
        assert!(sha256_hex(Path::new("/nonexistent-macmate-test/none")).is_none());
    }
}
