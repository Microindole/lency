//! DiagnosticSink - 诊断收集器
//!
//! 收集编译过程中的所有诊断信息

use crate::diagnostic::Diagnostic;
use crate::level::DiagnosticLevel;

/// 诊断收集器
#[derive(Debug, Default)]
pub struct DiagnosticSink {
    /// 收集的诊断列表
    diagnostics: Vec<Diagnostic>,
    /// 是否有错误
    has_errors: bool,
}

impl DiagnosticSink {
    /// 创建新的诊断收集器
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            has_errors: false,
        }
    }

    /// 添加诊断
    pub fn add(&mut self, diagnostic: Diagnostic) {
        if diagnostic.level.is_error() {
            self.has_errors = true;
        }
        self.diagnostics.push(diagnostic);
    }

    /// 添加错误
    pub fn error(&mut self, message: impl Into<String>) {
        self.add(Diagnostic::error(message));
    }

    /// 添加警告
    pub fn warning(&mut self, message: impl Into<String>) {
        self.add(Diagnostic::warning(message));
    }

    /// 添加信息
    pub fn info(&mut self, message: impl Into<String>) {
        self.add(Diagnostic::info(message));
    }

    /// 是否有错误
    pub fn has_errors(&self) -> bool {
        self.has_errors
    }

    /// 获取所有诊断
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.level.is_error())
            .count()
    }

    /// 获取警告数量
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| matches!(d.level, DiagnosticLevel::Warning))
            .count()
    }

    /// 清空所有诊断
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.has_errors = false;
    }

    /// 获取诊断数量
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sink_creation() {
        let sink = DiagnosticSink::new();
        assert!(!sink.has_errors());
        assert_eq!(sink.len(), 0);
        assert!(sink.is_empty());
    }

    #[test]
    fn test_add_diagnostic() {
        let mut sink = DiagnosticSink::new();

        sink.add(Diagnostic::error("error 1"));
        assert!(sink.has_errors());
        assert_eq!(sink.error_count(), 1);
        assert_eq!(sink.len(), 1);

        sink.add(Diagnostic::warning("warning 1"));
        assert_eq!(sink.warning_count(), 1);
        assert_eq!(sink.len(), 2);
    }

    #[test]
    fn test_convenience_methods() {
        let mut sink = DiagnosticSink::new();

        sink.error("error");
        sink.warning("warning");
        sink.info("info");

        assert!(sink.has_errors());
        assert_eq!(sink.error_count(), 1);
        assert_eq!(sink.warning_count(), 1);
        assert_eq!(sink.len(), 3);
    }

    #[test]
    fn test_clear() {
        let mut sink = DiagnosticSink::new();
        sink.error("error");
        sink.warning("warning");

        assert_eq!(sink.len(), 2);
        assert!(sink.has_errors());

        sink.clear();

        assert_eq!(sink.len(), 0);
        assert!(!sink.has_errors());
    }

    #[test]
    fn test_counts() {
        let mut sink = DiagnosticSink::new();

        sink.error("e1");
        sink.error("e2");
        sink.warning("w1");
        sink.info("i1");

        assert_eq!(sink.error_count(), 2);
        assert_eq!(sink.warning_count(), 1);
        assert_eq!(sink.len(), 4);
    }
}
