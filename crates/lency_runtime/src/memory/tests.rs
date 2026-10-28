use super::*;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn raw_allocations_are_zeroed_and_null_free_is_allowed() {
    let pointer = lency_alloc(32);
    assert!(
        unsafe { std::slice::from_raw_parts(pointer.cast::<u8>(), 32) }
            .iter()
            .all(|byte| *byte == 0)
    );
    unsafe {
        lency_free(pointer);
        lency_free(std::ptr::null_mut());
    }
    let empty = lency_alloc(0);
    assert!(!empty.is_null());
    unsafe { lency_free(empty) };
}

#[test]
fn registered_box_uses_its_rust_destructor() {
    struct Probe(Arc<AtomicUsize>);
    impl Drop for Probe {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let count = Arc::new(AtomicUsize::new(0));
    let value = own_box(Box::new(Probe(count.clone())));
    assert_eq!(count.load(Ordering::SeqCst), 0);
    unsafe { lency_free(value.cast()) };
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn both_string_allocators_use_the_matching_release() {
    let rust = own_string(CString::new("rust string").unwrap());
    let c = lency_alloc(8);
    unsafe {
        crate::lency_free_string(rust);
        crate::lency_free_string(c.cast());
    }
}

#[test]
fn container_release_preserves_independent_elements() {
    let value = own_string(CString::new("owned element").unwrap());
    let vector = crate::lency_vec_new(1);
    unsafe {
        crate::lency_vec_push(vector, value as i64);
        crate::lency_vec_free(vector);
    }
    assert!(allocations()
        .lock()
        .unwrap()
        .contains_key(&(value as usize)));
    assert_eq!(
        unsafe { std::ffi::CStr::from_ptr(value) }.to_bytes(),
        b"owned element"
    );
    unsafe { lency_free(value.cast()) };
}

#[test]
fn invalid_free_child() {
    match std::env::var("LENCY_TEST_INVALID_FREE").as_deref() {
        Ok("double") => {
            let pointer = lency_alloc(8);
            unsafe {
                lency_free(pointer);
                lency_free(pointer);
            }
        }
        Ok("literal") => unsafe { lency_free(c"static".as_ptr() as *mut c_void) },
        _ => {}
    }
}

#[test]
fn invalid_free_terminates_with_a_diagnostic() {
    for case in ["double", "literal"] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "memory::tests::invalid_free_child",
                "--nocapture",
            ])
            .env("LENCY_TEST_INVALID_FREE", case)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid free"));
    }
}
