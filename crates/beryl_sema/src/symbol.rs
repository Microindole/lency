//! Symbol Definitions
//!
//! 符号系统定义，采用可扩展的枚举设计，符合开闭原则。
//! 新增符号类型只需添加新的变体，不影响现有代码。

use beryl_syntax::ast::{Span, Type};
use std::collections::HashMap;

/// 符号 ID，用于在符号表中唯一标识
pub type SymbolId = usize;

/// 符号 - 程序中所有命名实体的统一表示
/// 
/// 设计原则：
/// - 每个变体对应一种语义实体
/// - 可扩展：未来可添加 Module, Trait, Const 等
#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(VariableSymbol),
    Function(FunctionSymbol),
    Class(ClassSymbol),
    Parameter(ParameterSymbol),
}

impl Symbol {
    /// 获取符号名称
    pub fn name(&self) -> &str {
        match self {
            Symbol::Variable(v) => &v.name,
            Symbol::Function(f) => &f.name,
            Symbol::Class(c) => &c.name,
            Symbol::Parameter(p) => &p.name,
        }
    }

    /// 获取符号的源码位置
    pub fn span(&self) -> &Span {
        match self {
            Symbol::Variable(v) => &v.span,
            Symbol::Function(f) => &f.span,
            Symbol::Class(c) => &c.span,
            Symbol::Parameter(p) => &p.span,
        }
    }

    /// 获取符号的类型（如果有）
    pub fn ty(&self) -> Option<&Type> {
        match self {
            Symbol::Variable(v) => Some(&v.ty),
            Symbol::Function(_) => None, // 函数本身不是值类型
            Symbol::Class(_) => None,    // 类本身不是值类型
            Symbol::Parameter(p) => Some(&p.ty),
        }
    }
}

/// 变量符号
/// 
/// 对应 `var x: int = 10` 或 `const PI = 3.14`
#[derive(Debug, Clone)]
pub struct VariableSymbol {
    pub name: String,
    pub ty: Type,
    pub is_mutable: bool,  // var = true, const = false
    pub span: Span,
    /// 是否已初始化（用于检测使用未初始化变量）
    pub is_initialized: bool,
}

impl VariableSymbol {
    pub fn new(name: String, ty: Type, is_mutable: bool, span: Span) -> Self {
        Self {
            name,
            ty,
            is_mutable,
            span,
            is_initialized: true,
        }
    }
}

/// 函数符号
///
/// 对应 `int add(int a, int b) { ... }`
#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub span: Span,
    /// 是否是公开的 (pub)
    pub is_public: bool,
}

impl FunctionSymbol {
    pub fn new(
        name: String,
        params: Vec<(String, Type)>,
        return_type: Type,
        span: Span,
    ) -> Self {
        Self {
            name,
            params,
            return_type,
            span,
            is_public: false,
        }
    }

    /// 获取参数数量
    pub fn arity(&self) -> usize {
        self.params.len()
    }
}

/// 类符号
///
/// 对应 `class User { ... }` 或 `class Box<T> { ... }`
#[derive(Debug, Clone)]
pub struct ClassSymbol {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: HashMap<String, FieldInfo>,
    pub methods: HashMap<String, FunctionSymbol>,
    pub span: Span,
}

/// 字段信息
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub ty: Type,
    pub span: Span,
}

impl ClassSymbol {
    pub fn new(name: String, generics: Vec<String>, span: Span) -> Self {
        Self {
            name,
            generics,
            fields: HashMap::new(),
            methods: HashMap::new(),
            span,
        }
    }

    /// 添加字段
    pub fn add_field(&mut self, name: String, ty: Type, span: Span) {
        self.fields.insert(name, FieldInfo { ty, span });
    }

    /// 添加方法
    pub fn add_method(&mut self, func: FunctionSymbol) {
        self.methods.insert(func.name.clone(), func);
    }

    /// 查找字段
    pub fn get_field(&self, name: &str) -> Option<&FieldInfo> {
        self.fields.get(name)
    }

    /// 查找方法
    pub fn get_method(&self, name: &str) -> Option<&FunctionSymbol> {
        self.methods.get(name)
    }

    /// 是否是泛型类
    pub fn is_generic(&self) -> bool {
        !self.generics.is_empty()
    }
}

/// 参数符号
///
/// 函数参数，作用域仅限函数体内
#[derive(Debug, Clone)]
pub struct ParameterSymbol {
    pub name: String,
    pub ty: Type,
    pub span: Span,
    /// 参数在参数列表中的索引
    pub index: usize,
}

impl ParameterSymbol {
    pub fn new(name: String, ty: Type, span: Span, index: usize) -> Self {
        Self { name, ty, span, index }
    }
}
