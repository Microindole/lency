//! Diagnostic - 诊断信息
//!
//! 表示一个编译器诊断（错误、警告等）

use crate::level::DiagnosticLevel;
use crate::span::Span;

/// 修复建议
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// 建议消息
    pub message: String,
    /// 建议的替换内容（如果有）
    pub replacement: Option<String>,
}

impl Suggestion {
    /// 创建新的建议
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            replacement: None,
        }
    }

    /// 添加替换内容
    pub fn with_replacement(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }
}

/// 诊断信息
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// 诊断级别
    pub level: DiagnosticLevel,
    /// 主要消息
    pub message: String,
    /// 源码位置（可选）
    pub span: Option<Span>,
    /// 补充注释
    pub notes: Vec<String>,
    /// 修复建议
    pub suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    /// 创建新的诊断
    pub fn new(level: DiagnosticLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            span: None,
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// 创建错误诊断
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(DiagnosticLevel::Error, message)
    }

    /// 创建警告诊断
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(DiagnosticLevel::Warning, message)
    }

    /// 创建信息诊断
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(DiagnosticLevel::Info, message)
    }

    /// 创建注释诊断
    pub fn note(message: impl Into<String>) -> Self {
        Self::new(DiagnosticLevel::Note, message)
    }

    /// 设置位置信息
    pub fn span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// 添加注释
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// 添加建议
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// 添加简单建议（仅消息）
    pub fn suggest(self, message: impl Into<String>) -> Self {
        self.with_suggestion(Suggestion::new(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error("test error")
            .span(0..10)
            .with_note("test note")
            .suggest("try this");

        assert_eq!(diag.level, DiagnosticLevel::Error);
        assert_eq!(diag.message, "test error");
        assert_eq!(diag.span, Some(0..10));
        assert_eq!(diag.notes.len(), 1);
        assert_eq!(diag.notes[0], "test note");
        assert_eq!(diag.suggestions.len(), 1);
        assert_eq!(diag.suggestions[0].message, "try this");
    }

    #[test]
    fn test_different_levels() {
        let error = Diagnostic::error("error");
        let warning = Diagnostic::warning("warning");
        let info = Diagnostic::info("info");
        let note = Diagnostic::note("note");

        assert_eq!(error.level, DiagnosticLevel::Error);
        assert_eq!(warning.level, DiagnosticLevel::Warning);
        assert_eq!(info.level, DiagnosticLevel::Info);
        assert_eq!(note.level, DiagnosticLevel::Note);
    }

    #[test]
    fn test_builder_pattern() {
        let diag = Diagnostic::error("test")
            .span(5..15)
            .with_note("note 1")
            .with_note("note 2")
            .suggest("suggestion 1")
            .suggest("suggestion 2");

        assert_eq!(diag.notes.len(), 2);
        assert_eq!(diag.suggestions.len(), 2);
    }
}
