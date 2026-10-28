# 表达式 Codegen

本目录把经过语义检查的表达式降低为 LLVM 值和控制流。

| 路径 | 职责 |
|---|---|
| `mod.rs` | 表达式分派入口 |
| `literal.rs`、`variable.rs` | 基础值 |
| `binary/`、`unary.rs` | 运算符 |
| `call.rs`、`method_call.rs` | 普通函数和方法调用 |
| `array.rs`、`vec.rs` | 数组与 Vec |
| `struct_init.rs`、`struct_access/` | struct 构造和访问 |
| `match_expr/` | 模式匹配控制流 |
| `string_ops.rs` | 字符串操作和边界检查 |
| `file_io/`、`hashmap/` | runtime 支持的操作 |
| `intrinsic.rs`、`conversion.rs` | 集中的原语与转换 lowering |
| memory.rs | 浅层手动释放及可空引用的原始存储表示 |
| place.rs | 单次求值的赋值位置、固定数组原址与集合槽写入 |

所有源码调用在 AST 中都是普通调用。需要 runtime 特殊处理的名称只允许在集中 lowering 边界识别，并必须经过 resolver 与 type checker。

逻辑与、逻辑或通过条件分支与合流值实现从左到右的短路求值，不提前执行右侧。嵌套表达式和运行时检查会产生新基本块，合流必须使用操作数实际结束块作为前驱。

成员赋值的接收者只求值一次，成员地址与字段类型来自同一个接收者值。经过语义检查允许的可空结构体成员访问仍保留运行时空值检查。
