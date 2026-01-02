/// 结构体符号
///
/// 对应 `struct Point { int x int y }`
#[derive(Debug, Clone)]
pub struct StructSymbol {
    pub name: String,
    pub fields: HashMap<String, FieldInfo>,
    pub span: Span,
}

impl StructSymbol {
    pub fn new(name: String, span: Span) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            span,
        }
    }

    /// 添加字段
    pub fn add_field(&mut self, name: String, ty: Type, span: Span) {
        self.fields.insert(name, FieldInfo { ty, span });
    }

    /// 查找字段
    pub fn get_field(&self, name: &str) -> Option<&FieldInfo> {
        self.fields.get(name)
    }
}
