//! Scope Management
//!
//! 作用域管理，处理嵌套作用域和符号查找。
//! 采用栈式作用域设计，支持块级作用域。

use crate::symbol::SymbolId;
use std::collections::HashMap;

pub mod stack;
pub use stack::ScopeStack;

/// 作用域 ID
pub type ScopeId = usize;

/// 单个作用域
#[derive(Debug, Clone)]
pub struct Scope {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub kind: ScopeKind,
    symbols: HashMap<String, SymbolId>,
    pub refinements: HashMap<String, beryl_syntax::ast::Type>,
}

/// 作用域类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// 全局作用域
    Global,
    /// 函数作用域
    Function,
    /// 类作用域
    Class,
    /// 块作用域 (if, while, {} 等)
    Block,
}

impl Scope {
    pub fn new(id: ScopeId, parent: Option<ScopeId>, kind: ScopeKind) -> Self {
        Self {
            id,
            parent,
            kind,
            symbols: HashMap::new(),
            refinements: HashMap::new(),
        }
    }

    /// 添加类型细化 (Flow Analysis)
    pub fn add_refinement(&mut self, name: String, ty: beryl_syntax::ast::Type) {
        self.refinements.insert(name, ty);
    }

    /// 在当前作用域定义符号
    pub fn define(&mut self, name: String, symbol_id: SymbolId) {
        self.symbols.insert(name, symbol_id);
    }

    /// 在当前作用域查找符号（不向上查找）
    pub fn lookup_local(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }
}
