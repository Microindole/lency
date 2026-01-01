//! Scope Management
//!
//! 作用域管理，处理嵌套作用域和符号查找。
//! 采用栈式作用域设计，支持块级作用域。

use crate::error::SemanticError;
use crate::symbol::{Symbol, SymbolId};
use std::collections::HashMap;

/// 作用域 ID
pub type ScopeId = usize;

/// 单个作用域
#[derive(Debug, Clone)]
pub struct Scope {
    /// 作用域 ID
    pub id: ScopeId,
    /// 父作用域 ID（全局作用域为 None）
    pub parent: Option<ScopeId>,
    /// 作用域内的符号（名称 -> 符号 ID）
    symbols: HashMap<String, SymbolId>,
    /// 作用域类型
    pub kind: ScopeKind,
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
            symbols: HashMap::new(),
            kind,
        }
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

/// 作用域栈 - 管理嵌套作用域
#[derive(Debug)]
pub struct ScopeStack {
    /// 所有作用域
    scopes: Vec<Scope>,
    /// 所有符号（由 SymbolId 索引）
    symbols: Vec<Symbol>,
    /// 当前作用域 ID
    current: ScopeId,
}

impl ScopeStack {
    /// 创建新的作用域栈，初始化全局作用域
    pub fn new() -> Self {
        let global = Scope::new(0, None, ScopeKind::Global);
        Self {
            scopes: vec![global],
            symbols: Vec::new(),
            current: 0,
        }
    }

    /// 进入新作用域
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let new_id = self.scopes.len();
        let new_scope = Scope::new(new_id, Some(self.current), kind);
        self.scopes.push(new_scope);
        self.current = new_id;
        new_id
    }

    /// 退出当前作用域，返回到父作用域
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current].parent {
            self.current = parent;
        }
    }

    /// 获取当前作用域 ID
    pub fn current_scope(&self) -> ScopeId {
        self.current
    }

    /// 获取当前作用域类型
    pub fn current_scope_kind(&self) -> ScopeKind {
        self.scopes[self.current].kind
    }

    /// 在当前作用域定义符号
    ///
    /// 如果已存在同名符号，返回 DuplicateDefinition 错误
    pub fn define(&mut self, symbol: Symbol) -> Result<SymbolId, SemanticError> {
        let name = symbol.name().to_string();
        let span = symbol.span().clone();

        // 检查当前作用域是否已有同名定义
        if let Some(existing_id) = self.scopes[self.current].lookup_local(&name) {
            let existing = &self.symbols[existing_id];
            return Err(SemanticError::DuplicateDefinition {
                name,
                span,
                previous_span: existing.span().clone(),
            });
        }

        // 分配新的符号 ID
        let symbol_id = self.symbols.len();
        self.symbols.push(symbol);

        // 在当前作用域注册
        self.scopes[self.current].define(name, symbol_id);

        Ok(symbol_id)
    }

    /// 查找符号（从当前作用域向上查找）
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.lookup_from(name, self.current)
    }

    /// 从指定作用域开始查找符号（向上查找）
    pub fn lookup_from(&self, name: &str, start_scope: ScopeId) -> Option<&Symbol> {
        let mut scope_id = Some(start_scope);

        while let Some(id) = scope_id {
            let scope = &self.scopes[id];
            if let Some(symbol_id) = scope.lookup_local(name) {
                return Some(&self.symbols[symbol_id]);
            }
            scope_id = scope.parent;
        }

        None
    }

    /// 仅在全局作用域查找符号
    pub fn lookup_global(&self, name: &str) -> Option<&Symbol> {
        self.lookup_from(name, 0)
    }

    /// 查找符号 ID（从当前作用域向上查找）
    pub fn lookup_id(&self, name: &str) -> Option<SymbolId> {
        let mut scope_id = Some(self.current);

        while let Some(id) = scope_id {
            let scope = &self.scopes[id];
            if let Some(symbol_id) = scope.lookup_local(name) {
                return Some(symbol_id);
            }
            scope_id = scope.parent;
        }

        None
    }

    /// 仅在当前作用域查找（不向上查找）
    pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
        self.scopes[self.current]
            .lookup_local(name)
            .map(|id| &self.symbols[id])
    }

    /// 设置当前作用域（用于在分析 Pass 中跳转）
    pub fn set_current(&mut self, scope_id: ScopeId) {
        if scope_id < self.scopes.len() {
            self.current = scope_id;
        }
    }

    /// 获取作用域数量
    pub fn scope_count(&self) -> usize {
        self.scopes.len()
    }

    /// 获取子作用域列表（用于遍历）
    pub fn get_child_scopes(&self, parent_id: ScopeId) -> Vec<ScopeId> {
        self.scopes
            .iter()
            .filter(|s| s.parent == Some(parent_id))
            .map(|s| s.id)
            .collect()
    }

    /// 根据 ID 获取符号
    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id)
    }

    /// 根据 ID 获取可变符号引用
    pub fn get_symbol_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        self.symbols.get_mut(id)
    }

    /// 检查是否在函数作用域内
    pub fn is_in_function(&self) -> bool {
        let mut scope_id = Some(self.current);
        while let Some(id) = scope_id {
            if self.scopes[id].kind == ScopeKind::Function {
                return true;
            }
            scope_id = self.scopes[id].parent;
        }
        false
    }

    /// 获取当前所在的函数名（如果在函数内）
    pub fn current_function(&self) -> Option<&str> {
        let mut scope_id = Some(self.current);
        while let Some(id) = scope_id {
            let scope = &self.scopes[id];
            if scope.kind == ScopeKind::Function {
                // 函数作用域的第一个符号通常是函数本身
                // 但更好的方式是在进入函数时记录函数名
                // 这里简化处理，查找父作用域的函数定义
                if let Some(parent_id) = scope.parent {
                    for name in self.scopes[parent_id].symbols.keys() {
                        if let Some(Symbol::Function(_)) = self.lookup(name) {
                            return Some(name);
                        }
                    }
                }
            }
            scope_id = scope.parent;
        }
        None
    }

    /// 获取所有符号（用于调试）
    pub fn all_symbols(&self) -> &[Symbol] {
        &self.symbols
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::VariableSymbol;
    use beryl_syntax::ast::Type;

    #[test]
    fn test_scope_nesting() {
        let mut scopes = ScopeStack::new();

        // 全局作用域
        let var_x = Symbol::Variable(VariableSymbol::new("x".to_string(), Type::Int, true, 0..1));
        scopes.define(var_x).unwrap();

        // 进入函数作用域
        scopes.enter_scope(ScopeKind::Function);
        let var_y = Symbol::Variable(VariableSymbol::new("y".to_string(), Type::Int, true, 2..3));
        scopes.define(var_y).unwrap();

        // 可以查到外层的 x
        assert!(scopes.lookup("x").is_some());
        // 可以查到当前的 y
        assert!(scopes.lookup("y").is_some());

        // 退出函数作用域
        scopes.exit_scope();

        // 回到全局，y 不可见
        assert!(scopes.lookup("y").is_none());
        // x 仍可见
        assert!(scopes.lookup("x").is_some());
    }

    #[test]
    fn test_duplicate_definition() {
        let mut scopes = ScopeStack::new();

        let var1 = Symbol::Variable(VariableSymbol::new("x".to_string(), Type::Int, true, 0..1));
        scopes.define(var1).unwrap();

        let var2 = Symbol::Variable(VariableSymbol::new(
            "x".to_string(),
            Type::String,
            true,
            2..3,
        ));
        let result = scopes.define(var2);

        assert!(result.is_err());
        if let Err(SemanticError::DuplicateDefinition { name, .. }) = result {
            assert_eq!(name, "x");
        }
    }
}
