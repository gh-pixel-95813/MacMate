//! 移至废纸篓封装。
//!
//! - macOS:优先通过 `objc` + `cocoa` 调用 `[NSWorkspace recycleURLs:completionHandler:]`,
//!   走系统 Trash API(可恢复);若调用失败则回退到 `trash` crate。
//! - 非 macOS:直接使用 `trash` crate(跨平台 fallback),用于 CI / 单元测试环境。
//!
//! 公共入口:`move_to_trash(&[PathBuf]) -> Result<(), AppError>`。

use crate::types::AppError;
use std::path::PathBuf;

/// macOS 实现:走 `NSWorkspace.recycleURLs:completionHandler:`,失败回退 `trash` crate。
#[cfg(target_os = "macos")]
mod platform {
    use super::AppError;
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSArray;
    use objc::rc::autoreleasepool;
    use objc::{class, msg_send};
    use std::ffi::CString;
    use std::path::PathBuf;
    use std::ptr;

    /// 调用 `[NSWorkspace sharedWorkspace] recycleURLs:completionHandler:`。
    ///
    /// 注:`completionHandler` 传 nil 表示 fire-and-forget,回收仍会执行,
    /// 只是不回调完成通知;此处以"未抛 ObjC 异常即视为成功"为最佳努力判定,
    /// 失败由上层 `move_to_trash` 回退到 `trash` crate。
    fn recycle_via_workspace(paths: &[PathBuf]) -> Result<(), String> {
        if paths.is_empty() {
            return Ok(());
        }
        autoreleasepool(|| unsafe {
            let mut urls: Vec<id> = Vec::with_capacity(paths.len());
            for path in paths {
                let path_str = path.to_string_lossy();
                let c_str = CString::new(path_str.as_ref())
                    .map_err(|e| format!("path contains NUL byte: {e}"))?;
                let ns_str: id = msg_send![class!(NSString), stringWithUTF8String: c_str.as_ptr()];
                let url: id = msg_send![class!(NSURL), fileURLWithPath: ns_str];
                if url.is_null() {
                    return Err(format!("NSURL is null for path: {}", path_str));
                }
                urls.push(url);
            }
            let array: id = NSArray::arrayWithObjects(nil, &urls);
            let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
            let () = msg_send![
                workspace,
                recycleURLs: array
                completionHandler: ptr::null::<objc::runtime::Object>()
            ];
            Ok(())
        })
    }

    pub fn move_to_trash(paths: &[PathBuf]) -> Result<(), AppError> {
        if paths.is_empty() {
            return Ok(());
        }
        if let Err(ws_err) = recycle_via_workspace(paths) {
            // 回退到跨平台 trash crate。
            for path in paths {
                if let Err(e) = ::trash::delete(path) {
                    return Err(AppError::Trash(format!(
                        "NSWorkspace failed ({ws_err}); trash fallback failed ({e})"
                    )));
                }
            }
        }
        Ok(())
    }
}

/// 非 macOS 实现:使用跨平台 `trash` crate。
#[cfg(not(target_os = "macos"))]
mod platform {
    use super::AppError;
    use std::path::PathBuf;

    pub fn move_to_trash(paths: &[PathBuf]) -> Result<(), AppError> {
        for path in paths {
            if let Err(e) = ::trash::delete(path) {
                return Err(AppError::Trash(e.to_string()));
            }
        }
        Ok(())
    }
}

/// 将给定路径移至废纸篓。空输入直接返回成功。
pub fn move_to_trash(paths: &[PathBuf]) -> Result<(), AppError> {
    platform::move_to_trash(paths)
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
