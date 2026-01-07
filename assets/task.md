# 当前任务清单: Sprint 8 特性 (Traits)
> 目标: 引入接口机制，实现泛型约束，为标准库和错误处理打下基础。遵循 "Safety by Default" 哲学。

### [ ] 1. 语法 & AST
- [ ] **Parser**: 支持 `trait Name { ... }` 接口定义。
- [ ] **Parser**: 支持 `impl Name for Type { ... }` 实现定义。
- [ ] **Generic Constraints**: 支持 `struct<T: Trait>` 和 `fn<T: Trait>` 语法。

### [ ] 2. 语义分析
- [ ] **Symbol Table**: 注册 Trait 符号及其方法签名。
- [ ] **Resolution**: 解析 `impl` 块，验证是否实现了 Trait 所有方法。
- [ ] **Constraint Check**: 在泛型实例化时，检查类型实参是否满足 Trait 约束。

### [ ] 3. 虚表与分发 (VTable)
- [ ] **VTable Layout**: 为实现了 Trait 的类型生成虚表 (可选，或仅做静态单态化约束)。
- [ ] **Static Dispatch**: 确保单态化时能正确查找到 Trait 方法的实现。

---

## 已完成任务 (History)

### Sprint 7: 泛型 (Generics)
- [x] Parser support (`<T>`)
- [x] Semantic Instantiation
- [x] Monomorphization & Codegen
- [x] Integration Tests (`struct_basic`, `fn_basic`, `complex_generics`)

## 未来规划

### Sprint 9: 枚举与错误处理 (Enums & Result)
- 实现 Sum Types (`enum`)
- 实现 `Result<T, E>` 模式
- 模式匹配 (`match`)

### Sprint 10: 标准库 (Std Lib)
- `List<T>`, `Map<K,V>` (基于 Traits)
- String 增强
- IO 与文件系统
