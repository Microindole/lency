//! 独立于本机优化和调试符号的 Lency 源码调用帧。

use std::cell::RefCell;
use std::ffi::CStr;
use std::io::{self, Write};
use std::os::raw::c_char;

#[derive(Clone, Copy)]
struct Frame {
    function: *const c_char,
    file: *const c_char,
    line: u32,
    column: u32,
}

thread_local! { static FRAMES: RefCell<Vec<Frame>> = const { RefCell::new(Vec::new()) }; }

/// # Safety
/// 名称和路径必须是有效 C 字符串，且在对应帧退出前保持有效。
#[no_mangle]
pub unsafe extern "C" fn lency_trace_enter(
    function: *const c_char,
    file: *const c_char,
    line: u32,
    column: u32,
) {
    FRAMES.with(|frames| {
        frames.borrow_mut().push(Frame {
            function,
            file,
            line,
            column,
        })
    });
}

/// # Safety
/// 路径必须是有效 C 字符串，且在当前帧退出前保持有效。
#[no_mangle]
pub unsafe extern "C" fn lency_trace_location(file: *const c_char, line: u32, column: u32) {
    FRAMES.with(|frames| {
        if let Some(frame) = frames.borrow_mut().last_mut() {
            frame.file = file;
            frame.line = line;
            frame.column = column;
        }
    });
}

#[no_mangle]
pub extern "C" fn lency_trace_leave() {
    let valid = FRAMES.with(|frames| frames.borrow_mut().pop().is_some());
    if !valid {
        unsafe { crate::lency_panic_at(c"invalid Lency frame exit".as_ptr(), 0) }
    }
}

unsafe fn text<'a>(pointer: *const c_char) -> &'a str {
    if pointer.is_null() {
        "<unknown>"
    } else {
        unsafe { CStr::from_ptr(pointer) }
            .to_str()
            .unwrap_or("<invalid UTF-8>")
    }
}

pub(crate) fn report(output: &mut impl Write, panic_line: u32) -> io::Result<()> {
    let frames = FRAMES.with(|frames| frames.borrow().clone());
    if !frames.is_empty() {
        writeln!(output, "Lency stack:")?;
    }
    for (index, frame) in frames.iter().rev().enumerate() {
        let line = if index == 0 && panic_line != 0 {
            panic_line
        } else {
            frame.line
        };
        unsafe {
            writeln!(
                output,
                "  at {} ({}:{}:{})",
                text(frame.function),
                text(frame.file),
                line,
                frame.column
            )?;
        }
    }
    Ok(())
}
