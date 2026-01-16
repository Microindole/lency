//! Lency Diagnostics
//!
//! 统一的诊断系统，为 Lency 编译器提供清晰、美观的错误报告。
//!
//! # 核心类型
//!
//! - [`Diagnostic`] - 诊断信息主体
//! - [`DiagnosticLevel`] - 诊断级别（Error/Warning/Info/Note）
//! - [`DiagnosticSink`] - 诊断收集器
//! - [`Emitter`] - 诊断输出器
//! - [`Span`] - 源码位置信息
//!
//! # 示例
//!
//! ```rust
//! use lency_diagnostics::{Diagnostic, DiagnosticSink, Emitter};
//!
//! let mut sink = DiagnosticSink::new();
//!
//! // 添加错误
//! sink.add(
//!     Diagnostic::error("type mismatch")
//!         .span(10..20)
//!         .with_note("expected 'int', found 'string'")
//!         .suggest("try using int_to_string()")
//! );
//!
//! // 检查是否有错误
//! if sink.has_errors() {
//!     let emitter = Emitter::new();
//!     emitter.emit_all(sink.diagnostics());
//! }
//! ```

pub mod diagnostic;
pub mod emitter;
pub mod level;
pub mod sink;
pub mod span;

// 重新导出核心类型
pub use diagnostic::{Diagnostic, Suggestion};
pub use emitter::Emitter;
pub use level::DiagnosticLevel;
pub use sink::DiagnosticSink;
pub use span::{Span, SpanExt};
