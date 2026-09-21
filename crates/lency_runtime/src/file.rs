//! Lency File I/O Runtime
//!
//! 提供 Lency 语言的文件 I/O 运行时支持

use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{Read, Write};
use std::os::raw::c_char;

/// Lency 文件句柄
#[repr(C)]
pub struct LencyFile {
    file: Option<File>,
    error_code: i64,
}

impl LencyFile {
    /// 创建一个新的文件句柄
    pub fn new(file: File) -> Box<Self> {
        Box::new(LencyFile {
            file: Some(file),
            error_code: 0,
        })
    }

    /// 创建一个错误文件句柄
    pub fn error(code: i64) -> Box<Self> {
        Box::new(LencyFile {
            file: None,
            error_code: code,
        })
    }
}

// FFI Functions

/// 打开文件
/// path: 文件路径 (C 字符串)
/// mode: 模式 - 0=读, 1=写, 2=追加
/// 返回: 文件句柄指针, 如果失败返回 NULL
///
/// # Safety
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn lency_file_open(path: *const c_char, mode: i64) -> *mut LencyFile {
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let file_result = match mode {
        0 => File::open(path_str),   // 读
        1 => File::create(path_str), // 写 (覆盖)
        2 => std::fs::OpenOptions::new() // 追加
            .append(true)
            .create(true)
            .open(path_str),
        _ => return std::ptr::null_mut(),
    };

    match file_result {
        Ok(f) => Box::into_raw(LencyFile::new(f)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// 将整个文件读取为新分配的 C 字符串；失败或内容包含 NUL 时返回 NULL。
///
/// # Safety
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn lency_file_read_string(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(path) => path,
        Err(_) => return std::ptr::null_mut(),
    };
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(_) => return std::ptr::null_mut(),
    };
    match CString::new(bytes) {
        Ok(content) => content.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// 关闭文件
///
/// # Safety
/// `handle` must be a valid pointer returned by `lency_file_open` or NULL.
#[no_mangle]
pub unsafe extern "C" fn lency_file_close(handle: *mut LencyFile) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

/// 读取文件全部内容到缓冲区
/// 返回: 已读取字节数, -1 表示错误
///
/// # Safety
/// - `handle` must be a valid pointer returned by `lency_file_open`.
/// - `buffer` must be a valid writable buffer of at least `buffer_size` bytes.
#[no_mangle]
pub unsafe extern "C" fn lency_file_read_all(
    handle: *mut LencyFile,
    buffer: *mut c_char,
    buffer_size: i64,
) -> i64 {
    if handle.is_null() || buffer.is_null() {
        return -1;
    }

    let lency_file = unsafe { &mut *handle };

    if let Some(ref mut file) = lency_file.file {
        let mut contents = Vec::new();
        match file.read_to_end(&mut contents) {
            Ok(bytes_read) => {
                let copy_len = std::cmp::min(bytes_read, (buffer_size - 1) as usize);
                unsafe {
                    std::ptr::copy_nonoverlapping(contents.as_ptr(), buffer as *mut u8, copy_len);
                    *buffer.add(copy_len) = 0; // Null terminate
                }
                copy_len as i64
            }
            Err(_) => -1,
        }
    } else {
        -1
    }
}

/// 写入字符串到文件
/// 返回: 已写入字节数, -1 表示错误
///
/// # Safety
/// - `handle` must be a valid pointer returned by `lency_file_open`.
/// - `data` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn lency_file_write(handle: *mut LencyFile, data: *const c_char) -> i64 {
    if handle.is_null() || data.is_null() {
        return -1;
    }

    let lency_file = unsafe { &mut *handle };

    if let Some(ref mut file) = lency_file.file {
        let c_str = unsafe { CStr::from_ptr(data) };
        let bytes = c_str.to_bytes();

        match file.write_all(bytes) {
            Ok(_) => bytes.len() as i64,
            Err(_) => -1,
        }
    } else {
        -1
    }
}

/// 检查文件是否有效
///
/// # Safety
/// `handle` must be a valid pointer returned by `lency_file_open` or NULL.
#[no_mangle]
pub unsafe extern "C" fn lency_file_is_valid(handle: *const LencyFile) -> i64 {
    if handle.is_null() {
        return 0;
    }

    let lency_file = unsafe { &*handle };
    if lency_file.file.is_some() {
        1
    } else {
        0
    }
}

/// Check if a file or directory exists
///
/// # Safety
/// `path` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn lency_file_exists(path: *const c_char) -> i64 {
    if path.is_null() {
        return 0;
    }

    let c_str = unsafe { CStr::from_ptr(path) };
    if let Ok(path_str) = c_str.to_str() {
        if std::path::Path::new(path_str).exists() {
            return 1;
        }
    }
    0
}

/// Check if a path is a directory
///
/// # Safety
/// `path` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn lency_file_is_dir(path: *const c_char) -> i64 {
    if path.is_null() {
        return 0;
    }

    let c_str = unsafe { CStr::from_ptr(path) };
    if let Ok(path_str) = c_str.to_str() {
        if std::path::Path::new(path_str).is_dir() {
            return 1;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_file_write_read() {
        // 写入（跨平台临时目录）
        let test_file = std::env::temp_dir().join("lency_test.txt");
        let path = CString::new(test_file.to_string_lossy().as_bytes()).unwrap();
        let write_handle = unsafe { lency_file_open(path.as_ptr(), 1) };
        assert!(!write_handle.is_null());

        let content = CString::new("Hello, Lency!").unwrap();
        let written = unsafe { lency_file_write(write_handle, content.as_ptr()) };
        assert!(written > 0);

        unsafe { lency_file_close(write_handle) };

        // 读取
        let read_handle = unsafe { lency_file_open(path.as_ptr(), 0) };
        assert!(!read_handle.is_null());

        let mut buffer = vec![0i8; 100];
        let read_bytes = unsafe { lency_file_read_all(read_handle, buffer.as_mut_ptr(), 100) };
        assert!(read_bytes > 0);

        unsafe { lency_file_close(read_handle) };

        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_read_string_is_not_limited_to_fixed_buffer() {
        let test_file = std::env::temp_dir().join("lency_large_read_test.txt");
        let content = "x".repeat(16 * 1024);
        std::fs::write(&test_file, &content).unwrap();
        let path = CString::new(test_file.to_string_lossy().as_bytes()).unwrap();

        let result = unsafe { lency_file_read_string(path.as_ptr()) };
        assert!(!result.is_null());
        assert_eq!(
            unsafe { CStr::from_ptr(result) }.to_bytes(),
            content.as_bytes()
        );
        unsafe { drop(CString::from_raw(result)) };

        let _ = std::fs::remove_file(test_file);
    }
}
