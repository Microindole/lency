//! Emitter - 诊断输出器
//!
//! 负责将诊断信息格式化输出

use crate::diagnostic::Diagnostic;
use colored::*;

/// 诊断输出器
pub struct Emitter {
    /// 是否使用颜色
    use_colors: bool,
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new()
    }
}

impl Emitter {
    /// 创建新的输出器
    pub fn new() -> Self {
        Self { use_colors: true }
    }

    /// 创建无颜色的输出器
    pub fn without_colors() -> Self {
        Self { use_colors: false }
    }

    /// 输出单个诊断
    pub fn emit(&self, diagnostic: &Diagnostic) {
        if self.use_colors {
            self.emit_colored(diagnostic);
        } else {
            self.emit_plain(diagnostic);
        }
    }

    /// 输出所有诊断
    pub fn emit_all(&self, diagnostics: &[Diagnostic]) {
        for diagnostic in diagnostics {
            self.emit(diagnostic);
            println!(); // 诊断之间空行
        }
    }

    /// 输出带颜色的诊断
    fn emit_colored(&self, diagnostic: &Diagnostic) {
        // 级别和消息
        println!(
            "{}: {}",
            diagnostic.level.colored_name(),
            diagnostic.message.bold()
        );

        // 位置信息（如果有）
        if let Some(span) = &diagnostic.span {
            println!("  {} {:?}", "-->".blue().bold(), span);
        }

        // 注释
        for note in &diagnostic.notes {
            println!(
                "  {} {}",
                "=".blue().bold(),
                format!("note: {}", note).bright_black()
            );
        }

        // 建议
        for suggestion in &diagnostic.suggestions {
            println!(
                "  {} {}",
                "=".green().bold(),
                format!("help: {}", suggestion.message).green()
            );
            if let Some(replacement) = &suggestion.replacement {
                println!("        try: {}", replacement.green().italic());
            }
        }
    }

    /// 输出纯文本诊断
    fn emit_plain(&self, diagnostic: &Diagnostic) {
        // 级别和消息
        println!("{}: {}", diagnostic.level, diagnostic.message);

        // 位置信息（如果有）
        if let Some(span) = &diagnostic.span {
            println!("  --> {:?}", span);
        }

        // 注释
        for note in &diagnostic.notes {
            println!("  = note: {}", note);
        }

        // 建议
        for suggestion in &diagnostic.suggestions {
            println!("  = help: {}", suggestion.message);
            if let Some(replacement) = &suggestion.replacement {
                println!("        try: {}", replacement);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitter_creation() {
        let emitter = Emitter::new();
        assert!(emitter.use_colors);

        let emitter_no_color = Emitter::without_colors();
        assert!(!emitter_no_color.use_colors);
    }

    #[test]
    fn test_emit_basic() {
        let emitter = Emitter::without_colors();
        let diag = Diagnostic::error("test error");

        // 这个测试只是确保不会panic
        emitter.emit(&diag);
    }

    #[test]
    fn test_emit_with_details() {
        let emitter = Emitter::without_colors();
        let diag = Diagnostic::error("test error")
            .span(10..20)
            .with_note("this is a note")
            .suggest("try this instead");

        emitter.emit(&diag);
    }
}
