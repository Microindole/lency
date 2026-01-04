use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // --- 基础类型 ---
    Int,    // i64
    Float,  // f64
    Bool,   // bool
    String, // string
    Void,   // void

    // --- 复杂类型 ---
    // 类引用: User, MyClass

    // 泛型实例化: List<int>, Map<string, int>
    Generic(String, Vec<Type>),

    // 可空类型: string? (Beryl 的核心安全特性)
    // 只有包了一层 Nullable 的才能是 null，其他默认非空
    Nullable(Box<Type>),

    // 数组类型: [int; 5]
    // 固定大小数组，长度是类型的一部分
    Array {
        element_type: Box<Type>,
        size: usize,
    },

    // 结构体类型: Point
    Struct(String),

    // 动态数组类型: Vec<T>
    Vec(Box<Type>),

    // 错误占位符 (当用户写错类型时，编译器用这个占位，防止崩溃)
    Error,
}

// 让类型能打印成好看的字符串: "List<int>", "string?"
impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::String => write!(f, "string"),
            Type::Void => write!(f, "void"),

            Type::Generic(name, args) => {
                write!(f, "{}<", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ">")
            }
            Type::Nullable(inner) => write!(f, "{}?", inner),
            Type::Array { element_type, size } => write!(f, "[{}]{}", size, element_type),
            Type::Struct(name) => write!(f, "{}", name),
            Type::Vec(inner) => write!(f, "Vec<{}>", inner),
            Type::Error => write!(f, "<?>"),
        }
    }
}
