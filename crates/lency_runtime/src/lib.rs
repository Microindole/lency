//! Lency Runtime Library
//!
//! 提供 Lency 语言的运行时支持，包括动态数组、哈希表、文件 I/O 和字符串处理

pub mod enum_value;
pub mod file;
pub mod hashmap;
pub mod hashmap_string;
pub mod string;
use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ffi::{CStr, CString};
use std::io::{self, Write};
use std::os::raw::c_char;

// ============== Panic Runtime ==============

/// Panic 函数 - 打印错误并退出程序
///
/// # Safety
/// `msg` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn lency_panic(msg: *const c_char) {
    if msg.is_null() {
        eprintln!("panic: (null message)");
    } else {
        let c_str = unsafe { CStr::from_ptr(msg) };
        match c_str.to_str() {
            Ok(s) => eprintln!("panic: {}", s),
            Err(_) => eprintln!("panic: (invalid UTF-8)"),
        }
    }
    std::process::exit(1);
}

/// Print a UTF-8 C string without adding a newline.
///
/// # Safety
/// `text` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn lency_print(text: *const c_char) -> i64 {
    if text.is_null() {
        return -1;
    }
    let bytes = unsafe { CStr::from_ptr(text) }.to_bytes();
    let mut stdout = io::stdout().lock();
    if stdout.write_all(bytes).is_err() || stdout.flush().is_err() {
        return -1;
    }
    0
}

/// Lency 动态数组
#[repr(C)]
pub struct LencyVec {
    data: *mut i64,
    len: i64,
    capacity: i64,
}

impl LencyVec {
    /// 创建新的 Vec
    pub fn new(initial_capacity: i64) -> Box<Self> {
        let capacity = if initial_capacity > 0 {
            initial_capacity
        } else {
            4
        };
        let layout = Layout::array::<i64>(capacity as usize).unwrap();

        let data = unsafe { alloc(layout) as *mut i64 };
        if data.is_null() {
            panic!("Failed to allocate Vec");
        }

        Box::new(LencyVec {
            data,
            len: 0,
            capacity,
        })
    }

    /// 扩容
    fn grow(&mut self) {
        let new_capacity = self.capacity * 2;
        let old_layout = Layout::array::<i64>(self.capacity as usize).unwrap();
        let new_layout = Layout::array::<i64>(new_capacity as usize).unwrap();

        let new_data =
            unsafe { realloc(self.data as *mut u8, old_layout, new_layout.size()) as *mut i64 };

        if new_data.is_null() {
            panic!("Failed to grow Vec");
        }

        self.data = new_data;
        self.capacity = new_capacity;
    }

    /// 添加元素
    pub fn push(&mut self, element: i64) {
        if self.len >= self.capacity {
            self.grow();
        }
        unsafe {
            *self.data.offset(self.len as isize) = element;
        }
        self.len += 1;
    }

    /// 弹出元素
    pub fn pop(&mut self) -> i64 {
        if self.len == 0 {
            eprintln!("Cannot pop from empty Vec");
            return 0;
        }
        self.len -= 1;
        unsafe { *self.data.offset(self.len as isize) }
    }

    /// 获取长度
    pub fn len(&self) -> i64 {
        self.len
    }

    /// 检查是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 获取元素
    pub fn get(&self, index: i64) -> i64 {
        if index < 0 || index >= self.len {
            panic!("Vec index out of bounds: {} (len: {})", index, self.len);
        }
        unsafe { *self.data.offset(index as isize) }
    }

    /// 设置元素
    pub fn set(&mut self, index: i64, value: i64) {
        if index < 0 || index >= self.len {
            panic!("Vec index out of bounds: {} (len: {})", index, self.len);
        }
        unsafe {
            *self.data.offset(index as isize) = value;
        }
    }
}

impl Drop for LencyVec {
    fn drop(&mut self) {
        if !self.data.is_null() {
            let layout = Layout::array::<i64>(self.capacity as usize).unwrap();
            unsafe {
                dealloc(self.data as *mut u8, layout);
            }
        }
    }
}

// C-compatible FFI functions

/// Create a new Vec
#[no_mangle]
pub extern "C" fn lency_vec_new(initial_capacity: i64) -> *mut LencyVec {
    Box::into_raw(LencyVec::new(initial_capacity))
}

/// Push an element to the Vec
///
/// # Safety
/// `vec` must be a valid pointer returned by `lency_vec_new`
#[no_mangle]
pub unsafe extern "C" fn lency_vec_push(vec: *mut LencyVec, element: i64) {
    unsafe {
        if !vec.is_null() {
            (*vec).push(element);
        }
    }
}

/// Pop an element from the Vec
///
/// # Safety  
/// `vec` must be a valid pointer returned by `lency_vec_new`
#[no_mangle]
pub unsafe extern "C" fn lency_vec_pop(vec: *mut LencyVec) -> i64 {
    unsafe {
        if vec.is_null() {
            return 0;
        }
        (*vec).pop()
    }
}

/// Get the length of the Vec
///
/// # Safety
/// `vec` must be a valid pointer returned by `lency_vec_new`
#[no_mangle]
pub unsafe extern "C" fn lency_vec_len(vec: *const LencyVec) -> i64 {
    unsafe {
        if vec.is_null() {
            return 0;
        }
        (*vec).len()
    }
}

/// Get an element from the Vec
///
/// # Safety
/// `vec` must be a valid pointer returned by `lency_vec_new`
#[no_mangle]
pub unsafe extern "C" fn lency_vec_get(vec: *const LencyVec, index: i64) -> i64 {
    unsafe {
        if vec.is_null() {
            return 0;
        }
        (*vec).get(index)
    }
}

/// Set an element in the Vec
///
/// # Safety
/// `vec` must be a valid pointer returned by `lency_vec_new`
#[no_mangle]
pub unsafe extern "C" fn lency_vec_set(vec: *mut LencyVec, index: i64, value: i64) {
    unsafe {
        if !vec.is_null() {
            (*vec).set(index, value);
        }
    }
}

/// Free a Vec
///
/// # Safety
/// `vec` must be a valid pointer returned by `lency_vec_new` and not already freed
#[no_mangle]
pub unsafe extern "C" fn lency_vec_free(vec: *mut LencyVec) {
    if !vec.is_null() {
        unsafe {
            let _ = Box::from_raw(vec);
        }
    }
}

// ============== Type Conversion FFI ==============

/// Convert int to string
/// Returns a newly allocated C string that must be freed
#[no_mangle]
pub extern "C" fn lency_int_to_string(n: i64) -> *mut i8 {
    let s = n.to_string();
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert float to string
#[no_mangle]
pub extern "C" fn lency_float_to_string(f: f64) -> *mut i8 {
    let s = format!("{}", f);
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Parse int from string, returns 0 on error
/// Sets is_ok to 1 on success, 0 on failure
///
/// # Safety
/// `s` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn lency_parse_int(s: *const i8, is_ok: *mut i32) -> i64 {
    if s.is_null() {
        if !is_ok.is_null() {
            *is_ok = 0;
        }
        return 0;
    }

    let c_str = CStr::from_ptr(s);
    match c_str.to_str() {
        Ok(str_slice) => match str_slice.trim().parse::<i64>() {
            Ok(n) => {
                if !is_ok.is_null() {
                    *is_ok = 1;
                }
                n
            }
            Err(_) => {
                if !is_ok.is_null() {
                    *is_ok = 0;
                }
                0
            }
        },
        Err(_) => {
            if !is_ok.is_null() {
                *is_ok = 0;
            }
            0
        }
    }
}

/// Parse float from string, returns 0.0 on error
///
/// # Safety
/// `s` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn lency_parse_float(s: *const i8, is_ok: *mut i32) -> f64 {
    if s.is_null() {
        if !is_ok.is_null() {
            *is_ok = 0;
        }
        return 0.0;
    }

    let c_str = CStr::from_ptr(s);
    match c_str.to_str() {
        Ok(str_slice) => match str_slice.trim().parse::<f64>() {
            Ok(f) => {
                if !is_ok.is_null() {
                    *is_ok = 1;
                }
                f
            }
            Err(_) => {
                if !is_ok.is_null() {
                    *is_ok = 0;
                }
                0.0
            }
        },
        Err(_) => {
            if !is_ok.is_null() {
                *is_ok = 0;
            }
            0.0
        }
    }
}

/// Free a string allocated by lency_int_to_string or lency_float_to_string
///
/// # Safety
/// `s` must be a valid pointer returned by one of the above functions
#[no_mangle]
pub unsafe extern "C" fn lency_free_string(s: *mut i8) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

#[cfg(windows)]
fn process_args() -> Vec<String> {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCommandLineW() -> *const u16;
        fn LocalFree(memory: isize) -> isize;
    }
    #[link(name = "shell32")]
    extern "system" {
        fn CommandLineToArgvW(command_line: *const u16, argc: *mut i32) -> *mut *mut u16;
    }

    unsafe {
        let mut argc = 0i32;
        let argv = CommandLineToArgvW(GetCommandLineW(), &mut argc);
        if argv.is_null() || argc <= 0 {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(argc as usize);
        for index in 0..argc as usize {
            let ptr = *argv.add(index);
            let mut len = 0usize;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            result.push(String::from_utf16_lossy(std::slice::from_raw_parts(
                ptr, len,
            )));
        }
        let _ = LocalFree(argv as isize);
        result
    }
}

#[cfg(not(windows))]
fn process_args() -> Vec<String> {
    std::env::args().collect()
}

/// Return process argument count (including executable path).
#[no_mangle]
pub extern "C" fn lency_arg_count() -> i64 {
    process_args().len() as i64
}

/// Return process argument at index as newly allocated C string.
///
/// # Safety
/// Caller must eventually free returned string with `lency_free_string`.
#[no_mangle]
pub unsafe extern "C" fn lency_arg_at(index: i64) -> *mut i8 {
    if index < 0 {
        return std::ptr::null_mut();
    }
    match process_args().into_iter().nth(index as usize) {
        Some(s) => match CString::new(s) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        None => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_new() {
        let vec = LencyVec::new(10);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity, 10);
    }

    #[test]
    fn test_vec_push_pop() {
        let mut vec = LencyVec::new(2);
        vec.push(1);
        vec.push(2);
        vec.push(3); // 触发扩容

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.pop(), 3);
        assert_eq!(vec.pop(), 2);
        assert_eq!(vec.len(), 1);
    }

    #[test]
    fn test_vec_get_set() {
        let mut vec = LencyVec::new(5);
        vec.push(10);
        vec.push(20);

        assert_eq!(vec.get(0), 10);
        assert_eq!(vec.get(1), 20);

        vec.set(0, 99);
        assert_eq!(vec.get(0), 99);
    }
}
