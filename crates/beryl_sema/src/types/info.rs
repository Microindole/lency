//! 类型信息查询接口
//!
//! 提供统一的类型属性查询，避免散落在各处的 match 表达式

use beryl_syntax::ast::Type;

/// 类型信息查询 Trait
///
/// 为 Type 枚举提供统一的查询接口，遵循开闭原则：
/// - 添加新类型时，只需在此处更新实现
/// - 避免在多处散落 match 语句
pub trait TypeInfo {
    /// 是否是数值类型（int, float）
    ///
    /// # Examples
    ///
    /// ```
    /// use beryl_sema::types::TypeInfo;
    /// use beryl_syntax::ast::Type;
    ///
    /// assert!(Type::Int.is_numeric());
    /// assert!(Type::Float.is_numeric());
    /// assert!(!Type::Bool.is_numeric());
    /// ```
    fn is_numeric(&self) -> bool;

    /// 是否可空
    ///
    /// # Examples
    ///
    /// ```
    /// use beryl_sema::types::TypeInfo;
    /// use beryl_syntax::ast::Type;
    ///
    /// let nullable_int = Type::Nullable(Box::new(Type::Int));
    /// assert!(nullable_int.is_nullable());
    /// assert!(!Type::Int.is_nullable());
    /// ```
    fn is_nullable(&self) -> bool;

    /// 是否是基础类型
    ///
    /// 基础类型包括：int, float, bool, string, void
    fn is_primitive(&self) -> bool;

    /// 是否是数组类型
    ///
    /// # Examples
    ///
    /// ```
    /// use beryl_sema::types::TypeInfo;
    /// use beryl_syntax::ast::Type;
    ///
    /// let arr = Type::Array {
    ///     element_type: Box::new(Type::Int),
    ///     size: 5,
    /// };
    /// assert!(arr.is_array());
    /// assert!(!Type::Int.is_array());
    /// ```
    fn is_array(&self) -> bool;

    /// 获取内层类型（用于 Nullable<T> 返回 T）
    ///
    /// # Returns
    ///
    /// - `Some(&Type)` - 如果是 Nullable 类型，返回内层类型
    /// - `None` - 如果不是 Nullable 类型
    fn inner_type(&self) -> Option<&Type>;

    /// 类型的显示名称
    fn display_name(&self) -> String;
}

/// 为 Type 实现 TypeInfo
impl TypeInfo for Type {
    fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    fn is_nullable(&self) -> bool {
        matches!(self, Type::Nullable(_))
    }

    fn is_primitive(&self) -> bool {
        matches!(
            self,
            Type::Int | Type::Float | Type::Bool | Type::String | Type::Void
        )
    }

    fn is_array(&self) -> bool {
        matches!(self, Type::Array { .. })
    }

    fn inner_type(&self) -> Option<&Type> {
        match self {
            Type::Nullable(inner) => Some(inner),
            _ => None,
        }
    }

    fn display_name(&self) -> String {
        self.to_string()
    }
}
