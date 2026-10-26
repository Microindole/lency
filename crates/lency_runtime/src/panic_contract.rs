use std::ffi::CString;
use std::process::Command;

#[test]
fn panic_child() {
    if std::env::var_os("LENCY_TEST_PANIC_CHILD").is_some() {
        let message = CString::new("native contract failure").unwrap();
        unsafe { crate::lency_panic_at(message.as_ptr(), 27) }
    }
}

#[test]
fn panic_reports_to_stderr_and_exits_with_failure() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "panic_contract::panic_child", "--nocapture"])
        .env("LENCY_TEST_PANIC_CHILD", "1")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("panic at line 27:"), "{diagnostic}");
    assert!(
        diagnostic.contains("native contract failure"),
        "{diagnostic}"
    );
}
