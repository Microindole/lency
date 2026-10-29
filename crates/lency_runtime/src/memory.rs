//! 手动内存管理：记录释放方式，不追踪对象关系，也不自动释放对象。

use std::collections::HashMap;
use std::ffi::{c_char, c_void, CString};
use std::sync::{Mutex, OnceLock};

type Destructor = unsafe fn(*mut c_void);
static ALLOCATIONS: OnceLock<Mutex<HashMap<usize, Destructor>>> = OnceLock::new();

fn allocations() -> &'static Mutex<HashMap<usize, Destructor>> {
    ALLOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register(pointer: *mut c_void, destructor: Destructor) {
    if !pointer.is_null() {
        let previous = allocations()
            .lock()
            .unwrap()
            .insert(pointer as usize, destructor);
        assert!(previous.is_none(), "allocation registered twice");
    }
}

unsafe fn drop_box<T>(pointer: *mut c_void) {
    unsafe { drop(Box::from_raw(pointer.cast::<T>())) };
}

unsafe fn drop_string(pointer: *mut c_void) {
    unsafe { drop(CString::from_raw(pointer.cast::<c_char>())) };
}

unsafe fn drop_c(pointer: *mut c_void) {
    unsafe { libc::free(pointer) };
}

pub(crate) fn own_box<T>(value: Box<T>) -> *mut T {
    let pointer = Box::into_raw(value);
    register(pointer.cast(), drop_box::<T>);
    pointer
}

pub(crate) fn own_string(value: CString) -> *mut c_char {
    let pointer = value.into_raw();
    register(pointer.cast(), drop_string);
    pointer
}

/// 分配字节存储，返回值只能通过 Lency 释放接口释放。
#[no_mangle]
pub extern "C" fn lency_alloc(size: u64) -> *mut c_void {
    let size = usize::try_from(size).unwrap_or_else(|_| unsafe {
        crate::lency_panic_at(c"allocation size overflow".as_ptr(), 0)
    });
    let pointer = unsafe { libc::calloc(1, size.max(1)) };
    if pointer.is_null() {
        unsafe { crate::lency_panic_at(c"out of memory".as_ptr(), 0) }
    }
    register(pointer, drop_c);
    pointer
}

/// 诊断当前仍需手动释放的分配数量；不扫描内存，也不改变对象生命周期。
#[no_mangle]
pub extern "C" fn lency_memory_live_allocations() -> u64 {
    allocations().lock().unwrap().len() as u64
}

/// 释放单个对象；不递归释放字段或集合元素所引用的独立对象。空指针无操作。
/// 未登记地址和未复用地址上的重复释放会报告 panic。
///
/// # Safety
/// 非空 pointer 必须是当前存活的 Lency 分配首地址；释放后所有别名均不可再使用。
/// 地址复用后无法仅凭裸地址区分旧别名与新对象，因此该检查不构成内存安全保证。
#[no_mangle]
pub unsafe extern "C" fn lency_free(pointer: *mut c_void) {
    if pointer.is_null() {
        return;
    }
    // 移除记录后再析构，避免析构期间持有登记表锁。
    let destructor = allocations().lock().unwrap().remove(&(pointer as usize));
    match destructor {
        Some(destructor) => unsafe { destructor(pointer) },
        None => unsafe {
            crate::lency_panic_at(
                c"invalid free: allocation is not live or not owned".as_ptr(),
                0,
            )
        },
    }
}

#[cfg(test)]
mod tests;
