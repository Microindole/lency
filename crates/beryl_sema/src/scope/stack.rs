use super::{Scope, ScopeId, ScopeKind};
use crate::error::SemanticError;
use crate::symbol::{Symbol, SymbolId};

/// 作用域栈 - 管理嵌套作用域
#[derive(Debug)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
    current: ScopeId,
    // 存储所有的符号
    symbols: Vec<Symbol>,
}

impl ScopeStack {
    /// 创建新的作用域栈，初始化全局作用域
    pub fn new() -> Self {
        let global_scope = Scope::new(0, None, ScopeKind::Global);
        Self {
            scopes: vec![global_scope],
            current: 0,
            symbols: Vec::new(),
        }
    }

    /// 进入新作用域
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let id = self.scopes.len();
        let new_scope = Scope::new(id, Some(self.current), kind);
        self.scopes.push(new_scope);
        self.current = id;
        id
    }

    /// 退出当前作用域，返回到父作用域
    pub fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current].parent {
            self.current = parent;
        } else {
            // 已经是全局作用域，不操作或 panic
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

        // 检查当前作用域是否已经有同名符号
        if let Some(prev_id) = self.scopes[self.current].lookup_local(&name) {
            let prev_span = self.symbols[prev_id].span().clone();
            return Err(SemanticError::DuplicateDefinition {
                name,
                span: symbol.span().clone(),
                previous_span: prev_span,
            });
        }

        let id = self.symbols.len();
        self.symbols.push(symbol);
        self.scopes[self.current].define(name, id);
        Ok(id)
    }

    /// 查找符号（从当前作用域向上查找）
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.lookup_id(name).and_then(|id| self.symbols.get(id))
    }

    /// 从指定作用域开始查找符号（向上查找）
    pub fn lookup_from(&self, name: &str, start_scope: ScopeId) -> Option<&Symbol> {
        let mut current_id = start_scope;
        loop {
            if let Some(symbol_id) = self.scopes[current_id].lookup_local(name) {
                return self.symbols.get(symbol_id);
            }

            if let Some(parent) = self.scopes[current_id].parent {
                current_id = parent;
            } else {
                break;
            }
        }
        None
    }

    /// 仅在全局作用域查找符号
    pub fn lookup_global(&self, name: &str) -> Option<&Symbol> {
        self.scopes[0]
            .lookup_local(name)
            .and_then(|id| self.symbols.get(id))
    }

    /// 查找符号 ID（从当前作用域向上查找）
    pub fn lookup_id(&self, name: &str) -> Option<SymbolId> {
        let mut current_id = self.current;
        loop {
            if let Some(symbol_id) = self.scopes[current_id].lookup_local(name) {
                return Some(symbol_id);
            }

            if let Some(parent) = self.scopes[current_id].parent {
                current_id = parent;
            } else {
                break;
            }
        }
        None
    }

    /// 仅在当前作用域查找（不向上查找）
    pub fn lookup_local(&self, name: &str) -> Option<&Symbol> {
        self.scopes[self.current]
            .lookup_local(name)
            .and_then(|id| self.symbols.get(id))
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
        // 这效率不高，但对于 ScopeStack 结构来说是最简单的实现
        // 优化方案：Scope 结构存储 children 列表
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

    /// 添加类型细化
    pub fn add_refinement(&mut self, name: String, ty: beryl_syntax::ast::Type) {
        self.scopes[self.current].add_refinement(name, ty);
    }

    /// 查找类型细化（Flow Sensitive）
    pub fn lookup_refinement(&self, name: &str) -> Option<beryl_syntax::ast::Type> {
        let mut current_id = self.current;
        loop {
            if let Some(ty) = self.scopes[current_id].refinements.get(name) {
                return Some(ty.clone());
            }

            if let Some(parent) = self.scopes[current_id].parent {
                current_id = parent;
            } else {
                break;
            }
        }
        None
    }

    /// 检查是否在函数作用域内
    pub fn is_in_function(&self) -> bool {
        let mut current_id = self.current;
        loop {
            if self.scopes[current_id].kind == ScopeKind::Function {
                return true;
            }
            if let Some(parent) = self.scopes[current_id].parent {
                current_id = parent;
            } else {
                break;
            }
        }
        false
    }

    /// 获取当前所在的函数名（如果在函数内）
    pub fn current_function(&self) -> Option<&str> {
        let mut current_id = self.current;
        loop {
            if self.scopes[current_id].kind == ScopeKind::Function {
                // Find symbol that created this scope?
                // Scope doesn't link back to symbol directly.
                // But usually Function scope is created after Function declaration.
                // This might need better linkage in future.
                // For now, return None or rely on traversal context.
                return None;
            }
            if let Some(parent) = self.scopes[current_id].parent {
                current_id = parent;
            } else {
                break;
            }
        }
        None
    }

    /// 获取所有符号（用于调试）
    pub fn all_symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// 获取特定作用域的可变引用 (用于 Flow Analysis 注入 refinement)
    pub fn get_scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id)
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
