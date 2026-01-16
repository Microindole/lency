//! Span - 源码位置信息
//!
//! 表示源代码中的位置范围

/// 源码位置范围 (字节偏移)
pub type Span = std::ops::Range<usize>;

/// Span 辅助函数
pub trait SpanExt {
    /// 创建一个新的 Span
    fn new(start: usize, end: usize) -> Self;

    /// 获取起始位置
    fn start(&self) -> usize;

    /// 获取结束位置
    fn end(&self) -> usize;

    /// 获取长度
    fn len(&self) -> usize;

    /// 是否为空
    fn is_empty(&self) -> bool;
}

impl SpanExt for Span {
    fn new(start: usize, end: usize) -> Self {
        start..end
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }

    fn len(&self) -> usize {
        self.end - self.start
    }

    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(10, 20);
        assert_eq!(span.start(), 10);
        assert_eq!(span.end(), 20);
        assert_eq!(SpanExt::len(&span), 10);
        assert!(!span.is_empty());
    }

    #[test]
    fn test_empty_span() {
        let span = Span::new(5, 5);
        assert!(span.is_empty());
        assert_eq!(SpanExt::len(&span), 0);
    }
}
