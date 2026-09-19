//! 文件系统工具:目录递归大小计算、文件遍历、人类可读格式化、修改时间。

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

/// 递归累加 `path` 下所有普通文件的总字节数。
///
/// - 不跟随符号链接(`follow_links(false)`),避免环路或越权统计。
/// - 遇到无权限目录或读取错误时跳过该项,不中断整体遍历。
/// - 入口为文件时返回该文件大小;不存在则返回 0。
pub fn dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    for entry in WalkDir::new(path).follow_links(false).into_iter() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // 跳过无权限目录、符号链接错误等
        };
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

/// 返回 `path` 下所有普通文件的路径(不跟随符号链接,跳过读取错误)。
pub fn walk_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path).follow_links(false).into_iter() {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    files
}

/// 将字节数转换为人类可读字符串,1 KB = 1024。
///
/// 规则:`>=1 TB` 两位小数;`>=1 GB` 两位小数;`>=1 MB` 整数;`>=1 KB` 整数;否则原样加 `B`。
/// 例:`1.23 GB` / `456 MB` / `789 KB` / `123 B`。
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 返回文件 / 目录最后修改时间(unix 秒),无法获取时返回 `None`。
pub fn modified_unix(path: &Path) -> Option<i64> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta.modified().ok()?;
    let dur = mtime.duration_since(UNIX_EPOCH).ok()?;
    Some(dur.as_secs() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn dir_size_aggregates_all_files() {
        let dir = tempfile::tempdir().unwrap();
        let f1 = dir.path().join("a.txt");
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        let f2 = sub.join("b.bin");
        fs::write(&f1, "hello").unwrap(); // 5 字节
        let mut f = fs::File::create(&f2).unwrap();
        f.write_all(&[0u8; 100]).unwrap(); // 100 字节
        assert_eq!(dir_size(dir.path()), 5 + 100);
    }

    #[test]
    fn dir_size_skips_unreadable_without_panic() {
        // 不存在的路径返回 0 而非 panic。
        assert_eq!(dir_size(Path::new("/nonexistent-macmate-test/zzz")), 0);
    }

    #[test]
    fn walk_files_returns_regular_files_only() {
        let dir = tempfile::tempdir().unwrap();
        let f1 = dir.path().join("a.txt");
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        let f2 = sub.join("b.bin");
        fs::write(&f1, "x").unwrap();
        fs::write(&f2, "yy").unwrap();
        let mut files = walk_files(dir.path());
        files.sort();
        assert_eq!(files.len(), 2, "应只返回 2 个普通文件(不含目录)");
        assert!(files.contains(&f1));
        assert!(files.contains(&f2));
    }

    #[test]
    fn format_size_units_match_spec() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(123), "123 B");
        assert_eq!(format_size(1024), "1 KB");
        assert_eq!(format_size(789 * 1024), "789 KB");
        assert_eq!(format_size(456 * 1024 * 1024), "456 MB");
        // 恰好 1 GiB(1024^3)= 1.00 GB(二进制单位,1 GB = 1024^3 字节)
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        // spec 示例 "1.23 GB":1.23 * 1024^3 ≈ 1_320_702_444 字节
        assert_eq!(format_size(1_320_702_444), "1.23 GB");
        assert_eq!(format_size(2 * 1024 * 1024 * 1024), "2.00 GB");
    }

    #[test]
    fn modified_unix_returns_positive_for_existing() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("a.txt");
        fs::write(&f, "x").unwrap();
        let ts = modified_unix(&f).expect("应能读取现有文件的 mtime");
        assert!(ts > 0, "mtime 应为正数(unix 秒)");
    }

    #[test]
    fn modified_unix_none_for_missing() {
        assert!(modified_unix(Path::new("/nonexistent-macmate-test/none")).is_none());
    }
}
