//! HashMap Runtime Implementation
//!
//! 使用开放寻址法 (Open Addressing) 实现的哈希表

use std::alloc::{alloc_zeroed, dealloc, Layout};

/// 哈希表条目状态
#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)] // Empty 通过 alloc_zeroed 隐式设置
enum EntryState {
    Empty = 0,
    Occupied = 1,
    Deleted = 2,
}

/// 哈希表条目
#[repr(C)]
struct Entry {
    key: i64,
    value: i64,
    state: EntryState,
}

/// Lency HashMap
#[repr(C)]
pub struct LencyHashMap {
    entries: *mut Entry,
    capacity: i64,
    len: i64,
    // 使用 0.7 的负载因子阈值
}

impl LencyHashMap {
    /// 创建新的 HashMap
    pub fn new(initial_capacity: i64) -> Box<Self> {
        let capacity = if initial_capacity > 0 {
            // 确保是 2 的幂次
            (initial_capacity as usize)
                .checked_next_power_of_two()
                .and_then(|value| i64::try_from(value).ok())
                .unwrap_or_else(|| unsafe {
                    crate::lency_panic_at(c"HashMap capacity overflow".as_ptr(), 0)
                })
        } else {
            8
        };

        let layout = Layout::array::<Entry>(capacity as usize).unwrap_or_else(|_| unsafe {
            crate::lency_panic_at(c"HashMap capacity overflow".as_ptr(), 0)
        });
        let entries = unsafe { alloc_zeroed(layout) as *mut Entry };

        if entries.is_null() {
            unsafe { crate::lency_panic_at(c"Failed to allocate HashMap".as_ptr(), 0) }
        }

        Box::new(LencyHashMap {
            entries,
            capacity,
            len: 0,
        })
    }

    /// 哈希函数 (简单的乘法哈希)
    fn hash(&self, key: i64) -> i64 {
        // 使用 FNV-1a 风格的哈希
        let mut h = key.wrapping_mul(0x517cc1b727220a95);
        h ^= h >> 32;
        h & (self.capacity - 1)
    }

    /// 扩容
    fn grow(&mut self) {
        let old_capacity = self.capacity;
        let old_entries = self.entries;

        let new_capacity = old_capacity.checked_mul(2).unwrap_or_else(|| unsafe {
            crate::lency_panic_at(c"HashMap capacity overflow".as_ptr(), 0)
        });
        let new_layout = Layout::array::<Entry>(new_capacity as usize).unwrap_or_else(|_| unsafe {
            crate::lency_panic_at(c"HashMap capacity overflow".as_ptr(), 0)
        });
        let new_entries = unsafe { alloc_zeroed(new_layout) as *mut Entry };

        if new_entries.is_null() {
            unsafe { crate::lency_panic_at(c"Failed to grow HashMap".as_ptr(), 0) }
        }

        self.entries = new_entries;
        self.capacity = new_capacity;
        self.len = 0;

        // 重新插入所有元素
        for i in 0..old_capacity {
            unsafe {
                let entry = &*old_entries.offset(i as isize);
                if entry.state == EntryState::Occupied {
                    self.insert_internal(entry.key, entry.value);
                }
            }
        }

        // 释放旧内存
        let old_layout = Layout::array::<Entry>(old_capacity as usize).unwrap();
        unsafe {
            dealloc(old_entries as *mut u8, old_layout);
        }
    }

    /// 内部插入 (不检查负载因子)
    fn insert_internal(&mut self, key: i64, value: i64) {
        let mut index = self.hash(key);
        let mut deleted = None;
        for _ in 0..self.capacity {
            let entry = unsafe { &mut *self.entries.offset(index as isize) };
            match entry.state {
                EntryState::Empty => {
                    self.write_entry(deleted.unwrap_or(index), key, value);
                    return;
                }
                EntryState::Deleted => {
                    deleted.get_or_insert(index);
                }
                EntryState::Occupied if entry.key == key => {
                    // 更新已有键的值
                    entry.value = value;
                    return;
                }
                EntryState::Occupied => {}
            }
            index = (index + 1) & (self.capacity - 1);
        }
        if let Some(index) = deleted {
            self.write_entry(index, key, value);
            return;
        }
        unsafe { crate::lency_panic_at(c"HashMap storage full".as_ptr(), 0) }
    }

    fn write_entry(&mut self, index: i64, key: i64, value: i64) {
        let entry = unsafe { &mut *self.entries.offset(index as isize) };
        entry.key = key;
        entry.value = value;
        entry.state = EntryState::Occupied;
        self.len += 1;
    }

    /// 插入键值对
    pub fn insert(&mut self, key: i64, value: i64) {
        // 检查负载因子 (0.7)
        if (self.len + 1) * 10 > self.capacity * 7 {
            self.grow();
        }
        self.insert_internal(key, value);
    }

    /// 查找键对应的值
    pub fn get(&self, key: i64) -> Option<i64> {
        let mut index = self.hash(key);
        let start = index;

        loop {
            let entry = unsafe { &*self.entries.offset(index as isize) };

            match entry.state {
                EntryState::Empty => return None,
                EntryState::Occupied if entry.key == key => return Some(entry.value),
                EntryState::Occupied | EntryState::Deleted => {
                    index = (index + 1) & (self.capacity - 1);
                    if index == start {
                        return None;
                    }
                }
            }
        }
    }

    /// 检查是否包含键
    pub fn contains(&self, key: i64) -> bool {
        self.get(key).is_some()
    }

    /// 删除键
    pub fn remove(&mut self, key: i64) -> bool {
        let mut index = self.hash(key);
        let start = index;

        loop {
            let entry = unsafe { &mut *self.entries.offset(index as isize) };

            match entry.state {
                EntryState::Empty => return false,
                EntryState::Occupied if entry.key == key => {
                    entry.state = EntryState::Deleted;
                    self.len -= 1;
                    return true;
                }
                EntryState::Occupied | EntryState::Deleted => {
                    index = (index + 1) & (self.capacity - 1);
                    if index == start {
                        return false;
                    }
                }
            }
        }
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
}

impl Drop for LencyHashMap {
    fn drop(&mut self) {
        if !self.entries.is_null() {
            let layout = Layout::array::<Entry>(self.capacity as usize).unwrap();
            unsafe {
                dealloc(self.entries as *mut u8, layout);
            }
        }
    }
}

// ============== FFI Functions ==============

/// Create a new HashMap
#[no_mangle]
pub extern "C" fn lency_hashmap_new(initial_capacity: i64) -> *mut LencyHashMap {
    crate::memory::own_box(LencyHashMap::new(initial_capacity))
}

/// Insert a key-value pair
///
/// # Safety
/// `map` must be a valid pointer returned by `lency_hashmap_new`
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_insert(map: *mut LencyHashMap, key: i64, value: i64) {
    if !map.is_null() {
        (*map).insert(key, value);
    }
}

/// Get a value by key, returns 0 if not found
/// Use lency_hashmap_contains to check existence first
///
/// # Safety
/// `map` must be a valid pointer returned by `lency_hashmap_new`
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_get(map: *const LencyHashMap, key: i64) -> i64 {
    if map.is_null() {
        return 0;
    }
    (*map).get(key).unwrap_or(0)
}

/// Check if a key exists
///
/// # Safety
/// `map` must be a valid pointer returned by `lency_hashmap_new`
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_contains(map: *const LencyHashMap, key: i64) -> bool {
    if map.is_null() {
        return false;
    }
    (*map).contains(key)
}

/// Remove a key, returns true if key was present
///
/// # Safety
/// `map` must be a valid pointer returned by `lency_hashmap_new`
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_remove(map: *mut LencyHashMap, key: i64) -> bool {
    if map.is_null() {
        return false;
    }
    (*map).remove(key)
}

/// Get the number of entries
///
/// # Safety
/// `map` must be a valid pointer returned by `lency_hashmap_new`
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_len(map: *const LencyHashMap) -> i64 {
    if map.is_null() {
        return 0;
    }
    (*map).len()
}

/// Free a HashMap
///
/// # Safety
/// `map` must be a valid pointer returned by `lency_hashmap_new` and not already freed
#[no_mangle]
pub unsafe extern "C" fn lency_hashmap_free(map: *mut LencyHashMap) {
    unsafe { crate::memory::lency_free(map.cast()) };
}

// ============== Tests ==============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updating_a_collision_after_deletion_does_not_duplicate_the_key() {
        let mut map = LencyHashMap::new(8);
        let other = (1..10_000)
            .find(|key| map.hash(*key) == map.hash(0))
            .unwrap();
        map.insert(0, 1);
        map.insert(other, 2);
        assert!(map.remove(0));
        map.insert(other, 3);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(other), Some(3));
        assert!(map.remove(other));
        assert_eq!(map.get(other), None);
    }

    #[test]
    fn test_hashmap_new() {
        let map = LencyHashMap::new(10);
        assert_eq!(map.len(), 0);
        assert!(map.capacity >= 10);
    }

    #[test]
    fn test_hashmap_insert_get() {
        let mut map = LencyHashMap::new(8);
        map.insert(1, 100);
        map.insert(2, 200);
        map.insert(3, 300);

        assert_eq!(map.get(1), Some(100));
        assert_eq!(map.get(2), Some(200));
        assert_eq!(map.get(3), Some(300));
        assert_eq!(map.get(4), None);
    }

    #[test]
    fn test_hashmap_update() {
        let mut map = LencyHashMap::new(8);
        map.insert(1, 100);
        assert_eq!(map.get(1), Some(100));

        map.insert(1, 999);
        assert_eq!(map.get(1), Some(999));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_hashmap_contains() {
        let mut map = LencyHashMap::new(8);
        map.insert(42, 1);

        assert!(map.contains(42));
        assert!(!map.contains(43));
    }

    #[test]
    fn test_hashmap_remove() {
        let mut map = LencyHashMap::new(8);
        map.insert(1, 100);
        map.insert(2, 200);

        assert!(map.remove(1));
        assert!(!map.contains(1));
        assert!(map.contains(2));
        assert_eq!(map.len(), 1);

        assert!(!map.remove(1)); // Already removed
    }

    #[test]
    fn test_hashmap_grow() {
        let mut map = LencyHashMap::new(4);

        // 插入足够多的元素触发扩容
        for i in 0..20 {
            map.insert(i, i * 10);
        }

        // 验证所有元素仍然存在
        for i in 0..20 {
            assert_eq!(map.get(i), Some(i * 10), "Failed for key {}", i);
        }
    }

    #[test]
    fn test_hashmap_string_keys() {
        // 测试使用指针作为键 (模拟字符串)
        let mut map = LencyHashMap::new(8);
        let s1 = "hello".as_ptr() as i64;
        let s2 = "world".as_ptr() as i64;

        map.insert(s1, 1);
        map.insert(s2, 2);

        assert_eq!(map.get(s1), Some(1));
        assert_eq!(map.get(s2), Some(2));
    }
}
