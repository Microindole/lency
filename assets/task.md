# 当前任务清单: Sprint 7 泛型

### [ ] 1. 语法 & AST
- [ ] **Parser**: 支持 `struct Name<T>` 和 `fn name<T>`。
- [ ] **Type Parsing**: 支持 `Type<Param>` 语法。

### [ ] 2. 语义分析
- [ ] **Symbol Table**: 注册并查找泛型参数 `T`。
- [ ] **Instantiation**: 在使用处将 `T` 替换为具体类型。

### [ ] 3. 代码生成 (单态化)
- [ ] **Struct**: 为每个实例 (`Box<int>`) 生成唯一的 LLVM 类型。
- [ ] **Function**: 为每个实例生成唯一的函数实现。

### [ ] 4. 验证
- [ ] **Run Tests**: 编写并运行泛型相关的集成测试。
