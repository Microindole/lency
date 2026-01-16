//! DiagnosticLevel - 诊断级别
//!
//! 定义错误、警告、信息等不同级别的诊断

use colored::*;
use std::fmt;

/// 诊断级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// 错误 - 阻止编译
    Error,
    /// 警告 - 不阻止编译但应注意
    Warning,
    /// 信息 - 提示性信息
    Info,
    /// 注释 - 补充说明
    Note,
}

impl DiagnosticLevel {
    /// 获取级别名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Note => "note",
        }
    }

    /// 获取带颜色的级别名称
    pub fn colored_name(&self) -> ColoredString {
        match self {
            Self::Error => self.name().red().bold(),
            Self::Warning => self.name().yellow().bold(),
            Self::Info => self.name().blue().bold(),
            Self::Note => self.name().bright_black().bold(),
        }
    }

    /// 是否为错误
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_name() {
        assert_eq!(DiagnosticLevel::Error.name(), "error");
        assert_eq!(DiagnosticLevel::Warning.name(), "warning");
        assert_eq!(DiagnosticLevel::Info.name(), "info");
        assert_eq!(DiagnosticLevel::Note.name(), "note");
    }

    #[test]
    fn test_is_error() {
        assert!(DiagnosticLevel::Error.is_error());
        assert!(!DiagnosticLevel::Warning.is_error());
        assert!(!DiagnosticLevel::Info.is_error());
        assert!(!DiagnosticLevel::Note.is_error());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DiagnosticLevel::Error), "error");
        assert_eq!(format!("{}", DiagnosticLevel::Warning), "warning");
    }
}
