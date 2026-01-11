//! Symbol Definitions
//!
//! 符号系统定义，采用可扩展的枚举设计，符合开闭原则。
//! 新增符号类型只需添加新的变体，不影响现有代码。

use beryl_syntax::ast::{Span, Type};
use std::collections::HashMap;

/// 符号 ID，用于在符号表中唯一标识
pub type SymbolId = usize;

/// 泛型参数符号
///
/// 表示泛型定义中的类型参数，如 `struct Box<T>` 中的 `T`
/// 或 `T identity<T>(T x)` 中的 `T`
#[derive(Debug, Clone)]
pub struct GenericParamSymbol {
    pub name: String,
    pub bound: Option<Type>, // 约束类型，如 T: Display
    pub span: Span,
}

impl GenericParamSymbol {
    pub fn new(name: String, bound: Option<Type>, span: Span) -> Self {
        Self { name, bound, span }
    }
}

/// 符号 - 程序中所有命名实体的统一表示
///
/// 设计原则：
/// - 每个变体对应一种语义实体
/// - 可扩展：未来可添加 Module, Trait, Const 等
#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(VariableSymbol),
    Function(FunctionSymbol),
    Parameter(ParameterSymbol),
    Struct(StructSymbol),
    GenericParam(GenericParamSymbol), // 泛型参数符号
    Trait(TraitSymbol),               // Trait 符号
    Enum(EnumSymbol),                 // Enum 符号
}

impl Symbol {
    /// 获取符号名称
    pub fn name(&self) -> &str {
        match self {
            Symbol::Variable(v) => &v.name,
            Symbol::Function(f) => &f.name,
            Symbol::Parameter(p) => &p.name,
            Symbol::Struct(s) => &s.name,
            Symbol::GenericParam(g) => &g.name,
            Symbol::Trait(t) => &t.name,
            Symbol::Enum(e) => &e.name,
        }
    }

    /// 获取符号的源码位置
    pub fn span(&self) -> &Span {
        match self {
            Symbol::Variable(v) => &v.span,
            Symbol::Function(f) => &f.span,
            Symbol::Parameter(p) => &p.span,
            Symbol::Struct(s) => &s.span,
            Symbol::GenericParam(g) => &g.span,
            Symbol::Trait(t) => &t.span,
            Symbol::Enum(e) => &e.span,
        }
    }

    /// 获取符号的类型（如果有）
    pub fn ty(&self) -> Option<&Type> {
        match self {
            Symbol::Variable(v) => Some(&v.ty),
            Symbol::Function(_) => None,
            Symbol::Parameter(p) => Some(&p.ty),
            Symbol::Struct(_) => None,
            Symbol::GenericParam(_) => None, // 泛型参数本身不是值类型
            Symbol::Trait(_) => None,        // Trait 不是值类型
            Symbol::Enum(_) => None,         // Enum本身是类型
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
    pub is_mutable: bool, // var = true, const = false
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
/// 泛型函数: `T identity<T>(T x) { ... }`
#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub name: String,
    pub generic_params: Vec<GenericParamSymbol>, // 泛型参数列表
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub span: Span,
    /// 是否是公开的 (pub)
    pub is_public: bool,
}

impl FunctionSymbol {
    pub fn new(name: String, params: Vec<(String, Type)>, return_type: Type, span: Span) -> Self {
        Self {
            name,
            generic_params: Vec::new(),
            params,
            return_type,
            span,
            is_public: false,
        }
    }

    /// 创建泛型函数符号
    pub fn new_generic(
        name: String,
        generic_params: Vec<GenericParamSymbol>,
        params: Vec<(String, Type)>,
        return_type: Type,
        span: Span,
    ) -> Self {
        Self {
            name,
            generic_params,
            params,
            return_type,
            span,
            is_public: false,
        }
    }

    /// 是否是泛型函数
    pub fn is_generic(&self) -> bool {
        !self.generic_params.is_empty()
    }

    /// 获取参数数量
    pub fn arity(&self) -> usize {
        self.params.len()
    }
}

/// 字段信息
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub ty: Type,
    pub span: Span,
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
        Self {
            name,
            ty,
            span,
            index,
        }
    }
}

/// 结构体符号
///
/// 对应 `struct Point { int x int y }`
/// 泛型结构体: `struct Box<T> { T value }`
#[derive(Debug, Clone)]
pub struct StructSymbol {
    pub name: String,
    pub generic_params: Vec<GenericParamSymbol>, // 泛型参数列表
    pub fields: HashMap<String, FieldInfo>,
    pub methods: HashMap<String, FunctionSymbol>,
    pub span: Span,
}

impl StructSymbol {
    pub fn new(name: String, span: Span) -> Self {
        Self {
            name,
            generic_params: Vec::new(),
            fields: HashMap::new(),
            methods: HashMap::new(),
            span,
        }
    }

    /// 创建泛型结构体符号
    pub fn new_generic(name: String, generic_params: Vec<GenericParamSymbol>, span: Span) -> Self {
        Self {
            name,
            generic_params,
            fields: HashMap::new(),
            methods: HashMap::new(),
            span,
        }
    }

    /// 是否是泛型结构体
    pub fn is_generic(&self) -> bool {
        !self.generic_params.is_empty()
    }

    /// 添加字段
    pub fn add_field(&mut self, name: String, ty: Type, span: Span) {
        self.fields.insert(name, FieldInfo { ty, span });
    }

    /// 查找字段
    pub fn get_field(&self, name: &str) -> Option<&FieldInfo> {
        self.fields.get(name)
    }

    /// 添加方法
    pub fn add_method(&mut self, name: String, method: FunctionSymbol) {
        self.methods.insert(name, method);
    }

    /// 查找方法
    pub fn get_method(&self, name: &str) -> Option<&FunctionSymbol> {
        self.methods.get(name)
    }
}

/// Trait 方法签名
///
/// 表示 Trait 中定义的方法签名（无函数体）
#[derive(Debug, Clone)]
pub struct TraitMethodSignature {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
}

impl TraitMethodSignature {
    pub fn new(name: String, params: Vec<(String, Type)>, return_type: Type) -> Self {
        Self {
            name,
            params,
            return_type,
        }
    }
}

/// Trait 符号
///
/// 对应 `trait Greeter { void greet(); }`
/// 泛型 Trait: `trait Comparable<T> { bool equals(T other); }`
#[derive(Debug, Clone)]
pub struct TraitSymbol {
    pub name: String,
    pub generic_params: Vec<GenericParamSymbol>,
    pub methods: Vec<TraitMethodSignature>,
    pub span: Span,
}

impl TraitSymbol {
    pub fn new(name: String, span: Span) -> Self {
        Self {
            name,
            generic_params: Vec::new(),
            methods: Vec::new(),
            span,
        }
    }

    /// 创建泛型 Trait 符号
    pub fn new_generic(name: String, generic_params: Vec<GenericParamSymbol>, span: Span) -> Self {
        Self {
            name,
            generic_params,
            methods: Vec::new(),
            span,
        }
    }

    /// 添加方法签名
    pub fn add_method(&mut self, method: TraitMethodSignature) {
        self.methods.push(method);
    }

    /// 查找方法
    pub fn get_method(&self, name: &str) -> Option<&TraitMethodSignature> {
        self.methods.iter().find(|m| m.name == name)
    }

    /// 是否是泛型 Trait
    pub fn is_generic(&self) -> bool {
        !self.generic_params.is_empty()
    }
}

/// Enum 符号
///
/// 对应 `enum Option<T> { Some(T), None }`
#[derive(Debug, Clone)]
pub struct EnumSymbol {
    pub name: String,
    pub generic_params: Vec<GenericParamSymbol>,
    pub variants: HashMap<String, Vec<Type>>, // 变体名 -> 字段类型列表
    pub methods: HashMap<String, FunctionSymbol>,
    pub span: Span,
}

impl EnumSymbol {
    pub fn new(name: String, span: Span) -> Self {
        Self {
            name,
            generic_params: Vec::new(),
            variants: HashMap::new(),
            methods: HashMap::new(),
            span,
        }
    }

    pub fn new_generic(name: String, generic_params: Vec<GenericParamSymbol>, span: Span) -> Self {
        Self {
            name,
            generic_params,
            variants: HashMap::new(),
            methods: HashMap::new(),
            span,
        }
    }

    pub fn is_generic(&self) -> bool {
        !self.generic_params.is_empty()
    }

    pub fn add_variant(&mut self, name: String, types: Vec<Type>) {
        self.variants.insert(name, types);
    }

    pub fn get_variant(&self, name: &str) -> Option<&Vec<Type>> {
        self.variants.get(name)
    }

    pub fn add_method(&mut self, name: String, method: FunctionSymbol) {
        self.methods.insert(name, method);
    }

    pub fn get_method(&self, name: &str) -> Option<&FunctionSymbol> {
        self.methods.get(name)
    }
}
