//! Lency String Runtime
//!
//! 提供 Lency 语言的字符串处理运行时支持

use std::ffi::CStr;
use std::os::raw::c_char;

use crate::LencyVec;

fn is_obviously_invalid_c_string_ptr(ptr: *const c_char) -> bool {
    let addr = ptr as usize;
    addr != 0 && addr < 4096
}

/// 比较两个字符串是否内容相等
///
/// # Safety
/// `lhs` and `rhs` must be valid null-terminated C string pointers unless null
#[no_mangle]
pub unsafe extern "C" fn lency_string_eq(lhs: *const c_char, rhs: *const c_char) -> i64 {
    if lhs.is_null() || rhs.is_null() {
        return if lhs == rhs { 1 } else { 0 };
    }
    // FIXME: This low-address guard is a defensive stopgap for self-host runtime
    // crashes when a scalar payload is misused as a string handle on Linux.
    if is_obviously_invalid_c_string_ptr(lhs) || is_obviously_invalid_c_string_ptr(rhs) {
        return 0;
    }

    let lhs_str = unsafe { CStr::from_ptr(lhs) };
    let rhs_str = unsafe { CStr::from_ptr(rhs) };
    if lhs_str.to_bytes() == rhs_str.to_bytes() {
        1
    } else {
        0
    }
}

/// 获取字符串长度
///
/// # Safety
/// `ptr` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn lency_string_len(ptr: *const c_char) -> i64 {
    if ptr.is_null() {
        return 0;
    }
    let c_str = unsafe { CStr::from_ptr(ptr) };
    c_str.to_bytes().len() as i64
}

/// 去除字符串首尾空白
/// 返回新分配的字符串
///
/// # Safety
/// `ptr` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn lency_string_trim(ptr: *const c_char) -> *mut c_char {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(ptr) };
    let s = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let trimmed = s.trim();

    // 分配新内存并复制
    let len = trimmed.len();
    let result = unsafe { libc::malloc(len + 1) as *mut c_char };
    if result.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        std::ptr::copy_nonoverlapping(trimmed.as_ptr(), result as *mut u8, len);
        *result.add(len) = 0; // null terminator
    }

    result
}

/// 按分隔符拆分字符串
/// 返回 LencyVec (存储字符串指针)
///
/// # Safety
/// `str_ptr` and `delim_ptr` must be valid null-terminated C strings
#[no_mangle]
pub unsafe extern "C" fn lency_string_split(
    str_ptr: *const c_char,
    delim_ptr: *const c_char,
) -> *mut LencyVec {
    if str_ptr.is_null() || delim_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(str_ptr) };
    let s = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let c_delim = unsafe { CStr::from_ptr(delim_ptr) };
    let delim = match c_delim.to_str() {
        Ok(d) => d,
        Err(_) => return std::ptr::null_mut(),
    };

    let parts: Vec<&str> = s.split(delim).collect();
    let vec = Box::into_raw(LencyVec::new(parts.len() as i64));

    for part in parts {
        // 分配每个子串
        let len = part.len();
        let part_ptr = unsafe { libc::malloc(len + 1) as *mut c_char };
        if !part_ptr.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(part.as_ptr(), part_ptr as *mut u8, len);
                *part_ptr.add(len) = 0;
                // 将指针作为 i64 存储 (因为 LencyVec 存储 i64)
                (*vec).push(part_ptr as i64);
            }
        }
    }

    vec
}

/// 用分隔符连接字符串数组
/// 返回新分配的字符串
///
/// # Safety
/// `vec_ptr` must be a valid LencyVec containing string pointers
/// `sep_ptr` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn lency_string_join(
    vec_ptr: *const LencyVec,
    sep_ptr: *const c_char,
) -> *mut c_char {
    if vec_ptr.is_null() || sep_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let c_sep = unsafe { CStr::from_ptr(sep_ptr) };
    let sep = match c_sep.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let vec = unsafe { &*vec_ptr };
    let len = vec.len();

    if len == 0 {
        // 返回空字符串
        let result = unsafe { libc::malloc(1) as *mut c_char };
        if !result.is_null() {
            unsafe { *result = 0 };
        }
        return result;
    }

    // 收集所有字符串
    let mut parts: Vec<String> = Vec::new();
    for i in 0..len {
        let str_ptr = vec.get(i) as *const c_char;
        if !str_ptr.is_null() {
            let c_str = unsafe { CStr::from_ptr(str_ptr) };
            if let Ok(s) = c_str.to_str() {
                parts.push(s.to_string());
            }
        }
    }

    let joined = parts.join(sep);
    let result_len = joined.len();
    let result = unsafe { libc::malloc(result_len + 1) as *mut c_char };
    if result.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        std::ptr::copy_nonoverlapping(joined.as_ptr(), result as *mut u8, result_len);
        *result.add(result_len) = 0;
    }

    result
}

/// 提取子串
/// 返回新分配的字符串
///
/// # Safety
/// `ptr` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn lency_string_substr(
    ptr: *const c_char,
    start: i64,
    len: i64,
) -> *mut c_char {
    if ptr.is_null() || start < 0 || len < 0 {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(ptr) };
    let s = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let start_idx = start as usize;
    let end_idx = std::cmp::min(start_idx + len as usize, s.len());

    if start_idx >= s.len() {
        // 返回空字符串
        let result = unsafe { libc::malloc(1) as *mut c_char };
        if !result.is_null() {
            unsafe { *result = 0 };
        }
        return result;
    }

    let substr = &s[start_idx..end_idx];
    let substr_len = substr.len();
    let result = unsafe { libc::malloc(substr_len + 1) as *mut c_char };
    if result.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        std::ptr::copy_nonoverlapping(substr.as_ptr(), result as *mut u8, substr_len);
        *result.add(substr_len) = 0;
    }

    result
}

/// 将字符码转换为单字符字符串
/// 返回新分配的字符串
///
/// # Safety
/// 调用者负责释放返回的内存
#[no_mangle]
pub unsafe extern "C" fn lency_char_to_string(char_code: i64) -> *mut c_char {
    // 分配2字节: 1个字符 + null terminator
    let result = unsafe { libc::malloc(2) as *mut c_char };
    if result.is_null() {
        return std::ptr::null_mut();
    }

    // 简单处理: 只支持 ASCII (0-127)
    // 对于 Unicode，需要更复杂的 UTF-8 编码
    if (0..=127).contains(&char_code) {
        unsafe {
            *result = char_code as c_char;
            *result.add(1) = 0; // null terminator
        }
    } else {
        // 非 ASCII: 返回空字符串或 '?'
        unsafe {
            *result = b'?' as c_char;
            *result.add(1) = 0;
        }
    }

    result
}

/// 字符串格式化：将模板中的 {} 占位符按顺序替换为 Vec 中的字符串
/// 返回新分配的字符串
///
/// # Safety
/// `template_ptr` must be a valid null-terminated C string
/// `vec_ptr` must be a valid LencyVec containing string pointers
#[no_mangle]
pub unsafe extern "C" fn lency_string_format(
    template_ptr: *const c_char,
    vec_ptr: *const LencyVec,
) -> *mut c_char {
    if template_ptr.is_null() || vec_ptr.is_null() {
        return std::ptr::null_mut();
    }

    let c_template = unsafe { CStr::from_ptr(template_ptr) };
    let template = match c_template.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let vec = unsafe { &*vec_ptr };
    let arg_count = vec.len();

    // 收集所有参数字符串
    let mut args: Vec<String> = Vec::new();
    for i in 0..arg_count {
        let str_ptr = vec.get(i) as *const c_char;
        if !str_ptr.is_null() {
            let c_str = unsafe { CStr::from_ptr(str_ptr) };
            if let Ok(s) = c_str.to_str() {
                args.push(s.to_string());
            }
        }
    }

    // 执行替换：逐字符扫描，遇到 {} 则替换
    let mut result = String::with_capacity(template.len() * 2);
    let mut arg_idx = 0;
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            if chars.peek() == Some(&'}') {
                chars.next(); // consume '}'
                if arg_idx < args.len() {
                    result.push_str(&args[arg_idx]);
                    arg_idx += 1;
                } else {
                    // 参数不足，保留 {}
                    result.push_str("{}");
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    let result_len = result.len();
    let buf = unsafe { libc::malloc(result_len + 1) as *mut c_char };
    if buf.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        std::ptr::copy_nonoverlapping(result.as_ptr(), buf as *mut u8, result_len);
        *buf.add(result_len) = 0;
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_string_len() {
        let s = CString::new("hello").unwrap();
        assert_eq!(unsafe { lency_string_len(s.as_ptr()) }, 5);

        let empty = CString::new("").unwrap();
        assert_eq!(unsafe { lency_string_len(empty.as_ptr()) }, 0);
    }

    #[test]
    fn test_string_trim() {
        let s = CString::new("  hello world  ").unwrap();
        let result = unsafe { lency_string_trim(s.as_ptr()) };
        assert!(!result.is_null());

        let trimmed = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        assert_eq!(trimmed, "hello world");

        unsafe { libc::free(result as *mut libc::c_void) };
    }

    #[test]
    fn test_string_split() {
        let s = CString::new("a,b,c").unwrap();
        let delim = CString::new(",").unwrap();

        let vec = unsafe { lency_string_split(s.as_ptr(), delim.as_ptr()) };
        assert!(!vec.is_null());

        unsafe {
            assert_eq!((*vec).len(), 3);

            let part0 = CStr::from_ptr((*vec).get(0) as *const c_char)
                .to_str()
                .unwrap();
            assert_eq!(part0, "a");

            let part1 = CStr::from_ptr((*vec).get(1) as *const c_char)
                .to_str()
                .unwrap();
            assert_eq!(part1, "b");

            let part2 = CStr::from_ptr((*vec).get(2) as *const c_char)
                .to_str()
                .unwrap();
            assert_eq!(part2, "c");

            // 清理
            for i in 0..(*vec).len() {
                libc::free((*vec).get(i) as *mut libc::c_void);
            }
            let _ = Box::from_raw(vec);
        }
    }

    #[test]
    fn test_string_join() {
        // 创建 vec 并填充
        let vec = Box::into_raw(LencyVec::new(3));
        let parts = ["hello", "world", "test"];

        unsafe {
            for part in &parts {
                let cs = CString::new(*part).unwrap();
                let ptr = libc::malloc(part.len() + 1) as *mut c_char;
                std::ptr::copy_nonoverlapping(cs.as_ptr(), ptr, part.len() + 1);
                (*vec).push(ptr as i64);
            }

            let sep = CString::new("-").unwrap();
            let result = lency_string_join(vec, sep.as_ptr());
            assert!(!result.is_null());

            let joined = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(joined, "hello-world-test");

            // 清理
            libc::free(result as *mut libc::c_void);
            for i in 0..(*vec).len() {
                libc::free((*vec).get(i) as *mut libc::c_void);
            }
            let _ = Box::from_raw(vec);
        }
    }

    #[test]
    fn test_string_substr() {
        let s = CString::new("hello world").unwrap();

        let result = unsafe { lency_string_substr(s.as_ptr(), 0, 5) };
        assert!(!result.is_null());
        let substr = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        assert_eq!(substr, "hello");
        unsafe { libc::free(result as *mut libc::c_void) };

        let result2 = unsafe { lency_string_substr(s.as_ptr(), 6, 5) };
        let substr2 = unsafe { CStr::from_ptr(result2) }.to_str().unwrap();
        assert_eq!(substr2, "world");
        unsafe { libc::free(result2 as *mut libc::c_void) };
    }

    #[test]
    fn test_string_eq() {
        let lhs = CString::new("hello").unwrap();
        let rhs = CString::new("hello").unwrap();
        let other = CString::new("world").unwrap();

        assert_eq!(unsafe { lency_string_eq(lhs.as_ptr(), rhs.as_ptr()) }, 1);
        assert_eq!(unsafe { lency_string_eq(lhs.as_ptr(), other.as_ptr()) }, 0);
        assert_eq!(
            unsafe { lency_string_eq(std::ptr::null(), std::ptr::null()) },
            1
        );
    }

    #[test]
    fn test_string_eq_rejects_obviously_invalid_small_pointers() {
        assert!(is_obviously_invalid_c_string_ptr(1usize as *const c_char));
        assert!(is_obviously_invalid_c_string_ptr(
            4095usize as *const c_char
        ));
        assert!(!is_obviously_invalid_c_string_ptr(std::ptr::null()));
        assert!(!is_obviously_invalid_c_string_ptr(
            4096usize as *const c_char
        ));

        let rhs = CString::new("hello").unwrap();
        assert_eq!(
            unsafe { lency_string_eq(8usize as *const c_char, rhs.as_ptr()) },
            0
        );
    }

    #[test]
    fn test_string_format() {
        use crate::LencyVec;

        // 测试基础替换
        let template = CString::new("hello {}!").unwrap();
        let arg = CString::new("world").unwrap();

        let mut vec = LencyVec::new(4);
        vec.push(arg.as_ptr() as i64);

        let result = unsafe { lency_string_format(template.as_ptr(), &*vec) };
        assert!(!result.is_null());
        let formatted = unsafe { CStr::from_ptr(result) }.to_str().unwrap();
        assert_eq!(formatted, "hello world!");
        unsafe { libc::free(result as *mut libc::c_void) };

        // 测试多参数替换
        let template2 = CString::new("{} + {} = {}").unwrap();
        let a1 = CString::new("1").unwrap();
        let a2 = CString::new("2").unwrap();
        let a3 = CString::new("3").unwrap();

        let mut vec2 = LencyVec::new(4);
        vec2.push(a1.as_ptr() as i64);
        vec2.push(a2.as_ptr() as i64);
        vec2.push(a3.as_ptr() as i64);

        let result2 = unsafe { lency_string_format(template2.as_ptr(), &*vec2) };
        assert!(!result2.is_null());
        let formatted2 = unsafe { CStr::from_ptr(result2) }.to_str().unwrap();
        assert_eq!(formatted2, "1 + 2 = 3");
        unsafe { libc::free(result2 as *mut libc::c_void) };

        // 测试无占位符
        let template3 = CString::new("no placeholders").unwrap();
        let vec3 = LencyVec::new(4);

        let result3 = unsafe { lency_string_format(template3.as_ptr(), &*vec3) };
        assert!(!result3.is_null());
        let formatted3 = unsafe { CStr::from_ptr(result3) }.to_str().unwrap();
        assert_eq!(formatted3, "no placeholders");
        unsafe { libc::free(result3 as *mut libc::c_void) };
    }
}
