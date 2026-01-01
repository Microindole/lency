//! 类型注册表
//!
//! 当前版本为简化实现，为未来的自定义类型和类型别名做准备

use beryl_syntax::ast::Type;
use std::collections::HashMap;

/// 类型注册表
///
/// 用途：
/// 1. 为未来的自定义类型做准备
/// 2. 集中管理类型别名（如 i64 -> int）
///
/// 当前版本为简化实现，主要作为扩展预留
pub struct TypeRegistry {
    // 类型别名: "i64" -> Type::Int
    aliases: HashMap<String, Type>,
}

impl TypeRegistry {
    /// 创建新的类型注册表
    pub fn new() -> Self {
        let mut registry = Self {
            aliases: HashMap::new(),
        };
        registry.register_builtins();
        registry
    }

    /// 注册内置类型别名
    fn register_builtins(&mut self) {
        // 为未来的类型别名系统预留
        // 例如:
        // self.aliases.insert("i64".to_string(), Type::Int);
        // self.aliases.insert("f64".to_string(), Type::Float);
    }

    /// 根据名称查找类型
    ///
    /// # Examples
    ///
    /// ```
    /// use beryl_sema::types::TypeRegistry;
    ///
    /// let registry = TypeRegistry::new();
    /// // 将来可以: let ty = registry.lookup("i64");
    /// ```
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.aliases.get(name)
    }

    /// 注册自定义类型别名（为未来扩展预留）
    #[allow(dead_code)]
    pub fn register_alias(&mut self, alias: String, ty: Type) {
        self.aliases.insert(alias, ty);
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_registry() {
        let registry = TypeRegistry::new();
        // 当前版本只确保能创建
        assert!(registry.aliases.is_empty());
    }

    #[test]
    fn test_register_alias() {
        let mut registry = TypeRegistry::new();
        registry.register_alias("integer".to_string(), Type::Int);

        assert_eq!(registry.lookup("integer"), Some(&Type::Int));
        assert_eq!(registry.lookup("unknown"), None);
    }
}
