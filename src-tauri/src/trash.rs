//! 移至废纸篓封装。
//!
//! 统一使用跨平台 `trash` crate:
//! - macOS:`trash` crate 内部通过 Finder (`osascript`) 将文件移至系统废纸篓,可恢复。
//! - 其他平台:走各自桌面环境的回收站实现。
//!
//! 公共入口:`move_to_trash(&[PathBuf]) -> Result<(), AppError>`。

use crate::types::AppError;
use std::path::PathBuf;

/// 将给定路径移至废纸篓。空输入直接返回成功。
pub fn move_to_trash(paths: &[PathBuf]) -> Result<(), AppError> {
    if paths.is_empty() {
        return Ok(());
    }
    for path in paths {
        if let Err(e) = ::trash::delete(path) {
            return Err(AppError::Trash(format!(
                "failed to move {} to trash: {e}",
                path.display()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn empty_input_is_ok() {
        assert!(move_to_trash(&[]).is_ok());
    }

    #[test]
    fn file_disappears_after_move_to_trash() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("to_trash.txt");
        fs::write(&f, "hello").unwrap();
        assert!(f.exists());
        match move_to_trash(std::slice::from_ref(&f)) {
            Ok(()) => assert!(!f.exists(), "文件应在移至废纸篓后从原位置消失"),
            Err(e) => panic!("move_to_trash 失败(沙箱可能不支持 Trash):{e}"),
        }
    }
}
