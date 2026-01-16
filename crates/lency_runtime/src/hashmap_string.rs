//! HashMap<String, Int> 实现
//!
//! 为 Lency 提供字符串键的哈希表支持，用于符号表等核心数据结构

use std::collections::HashMap as StdHashMap;
use std::ffi::CStr;
use std::os::raw::c_char;

/// String → Int 的哈希表
pub struct LencyHashMapString {
    map: StdHashMap<String, i64>,
}

impl LencyHashMapString {
    fn new() -> Self {
        Self {
            map: StdHashMap::new(),
        }
    }

    fn insert(&mut self, key: String, value: i64) {
        self.map.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<i64> {
        self.map.get(key).copied()
    }

    fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    fn remove(&mut self, key: &str) -> bool {
        self.map.remove(key).is_some()
    }

    fn len(&self) -> usize {
        self.map.len()
    }
}

// ============== FFI 函数 ==============

/// 创建新的 HashMap<String, Int>
#[no_mangle]
pub extern "C" fn lency_hashmap_string_new() -> *mut LencyHashMapString {
    Box::into_raw(Box::new(LencyHashMapString::new()))
}

/// 插入键值对
///
/// # Safety
/// - map 必须是有效的指针
/// - key 必须是有效的 C 字符串
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_string_insert(
    map: *mut LencyHashMapString,
    key: *const c_char,
    value: i64,
) {
    if map.is_null() || key.is_null() {
        return;
    }

    let map = &mut *map;
    let key_str = CStr::from_ptr(key).to_string_lossy().into_owned();
    map.insert(key_str, value);
}

/// 获取值
///
/// 返回值：
/// - 如果找到：返回值
/// - 如果未找到：返回 0
///
/// # Safety
/// - map 必须是有效的指针
/// - key 必须是有效的 C 字符串
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_string_get(
    map: *const LencyHashMapString,
    key: *const c_char,
) -> i64 {
    if map.is_null() || key.is_null() {
        return 0;
    }

    let map = &*map;
    let key_str = CStr::from_ptr(key).to_string_lossy();
    map.get(&key_str).unwrap_or(0)
}

/// 检查键是否存在
///
/// # Safety
/// - map 必须是有效的指针
/// - key 必须是有效的 C 字符串
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_string_contains(
    map: *const LencyHashMapString,
    key: *const c_char,
) -> bool {
    if map.is_null() || key.is_null() {
        return false;
    }

    let map = &*map;
    let key_str = CStr::from_ptr(key).to_string_lossy();
    map.contains(&key_str)
}

/// 删除键值对
///
/// 返回值：
/// - true: 删除成功
/// - false: 键不存在
///
/// # Safety
/// - map 必须是有效的指针
/// - key 必须是有效的 C 字符串
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_string_remove(
    map: *mut LencyHashMapString,
    key: *const c_char,
) -> bool {
    if map.is_null() || key.is_null() {
        return false;
    }

    let map = &mut *map;
    let key_str = CStr::from_ptr(key).to_string_lossy();
    map.remove(&key_str)
}

/// 获取大小
///
/// # Safety
/// - map 必须是有效的指针
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_string_len(map: *const LencyHashMapString) -> i64 {
    if map.is_null() {
        return 0;
    }

    let map = &*map;
    map.len() as i64
}

/// 释放 HashMap
///
/// # Safety
/// - map 必须是有效的指针
/// - 释放后不能再使用该指针
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_string_free(map: *mut LencyHashMapString) {
    if !map.is_null() {
        let _ = Box::from_raw(map);
    }
}

// ============== 测试 ==============

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_hashmap_string_basic() {
        unsafe {
            let map = lency_hashmap_string_new();
            assert!(!map.is_null());

            // 插入
            let key1 = CString::new("hello").unwrap();
            let key2 = CString::new("world").unwrap();
            lency_hashmap_string_insert(map, key1.as_ptr(), 42);
            lency_hashmap_string_insert(map, key2.as_ptr(), 100);

            // 获取
            assert_eq!(lency_hashmap_string_get(map, key1.as_ptr()), 42);
            assert_eq!(lency_hashmap_string_get(map, key2.as_ptr()), 100);

            // 检查存在
            assert!(lency_hashmap_string_contains(map, key1.as_ptr()));

            let key3 = CString::new("notfound").unwrap();
            assert!(!lency_hashmap_string_contains(map, key3.as_ptr()));

            // 长度
            assert_eq!(lency_hashmap_string_len(map), 2);

            // 删除
            assert!(lency_hashmap_string_remove(map, key1.as_ptr()));
            assert!(!lency_hashmap_string_contains(map, key1.as_ptr()));
            assert_eq!(lency_hashmap_string_len(map), 1);

            // 清理
            lency_hashmap_string_free(map);
        }
    }

    #[test]
    fn test_hashmap_string_overwrite() {
        unsafe {
            let map = lency_hashmap_string_new();
            let key = CString::new("key").unwrap();

            lency_hashmap_string_insert(map, key.as_ptr(), 1);
            assert_eq!(lency_hashmap_string_get(map, key.as_ptr()), 1);

            lency_hashmap_string_insert(map, key.as_ptr(), 2);
            assert_eq!(lency_hashmap_string_get(map, key.as_ptr()), 2);
            assert_eq!(lency_hashmap_string_len(map), 1);

            lency_hashmap_string_free(map);
        }
    }

    #[test]
    fn test_hashmap_string_empty() {
        unsafe {
            let map = lency_hashmap_string_new();
            assert_eq!(lency_hashmap_string_len(map), 0);

            let key = CString::new("notexist").unwrap();
            assert_eq!(lency_hashmap_string_get(map, key.as_ptr()), 0);
            assert!(!lency_hashmap_string_remove(map, key.as_ptr()));

            lency_hashmap_string_free(map);
        }
    }
}
