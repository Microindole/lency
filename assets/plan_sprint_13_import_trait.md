# Sprint 13 详细实施计划：标准库完善与 HashMap

**状态更新**: 经核查，模块导入 (`Import`) 和基础 Trait (`Hash`, `Eq`) 及其标准库实现 (`std.core`) **已经存在**。
**当前目标**: 验证现有实现的正确性，修复潜在 Bug，并着手实现 `HashMap`。

## 1. 验证阶段 (Verification Phase)

### 1.1 模块导入与基础 Traits
- [ ] **执行测试**: 运行 `tests/integration/traits/hash_basic.lcy`。
- [ ] **预期结果**: 如果测试通过，说明 Import 系统和 Trait 实现均正常。
- [ ] **潜在修复**:
  - 如果 Import 失败，检查 `Resolver::resolve_import` 路径逻辑。
  - 如果 Trait 调用失败，检查 `std.core` 实现或方法查找逻辑。

### 1.2 字符串索引 (String Indexing)
- [ ] `hash_basic.lcy` 包含 `s[0]` 操作。
- [ ] 检查 Sema 是否支持 String Indexing (作为 Int 数组处理或特殊处理)。

## 2. 核心功能开发：HashMap (Phase 2)

一旦 `basic_hash.lcy` 通过，即可基于 `Hash` + `Eq` 实现 `HashMap`。

- [ ] **Target File**: `lib/std/collections.lcy` (检查是否存在，若存在则完善)。
- [ ] **Struct Definition**:
  ```rust
  struct HashMapNode<K, V> {
      K key
      V value
      HashMapNode<K, V>? next
  }

  struct HashMap<K, V> {
      Vec<HashMapNode<K, V>?> buckets // 需要 Vec<T> 支持
      int size
  }
  ```
- [ ] **Methods**:
  - `insert(K key, V value)`
  - `get(K key) -> V?` or `Result<V, Error>`
  - `contains_key(K key) -> bool`

## 3. 执行顺序

1. **Verify**: 运行 `lencyc run tests/integration/traits/hash_basic.lcy`。
2. **Fix**: 修复发现的任何集成问题 (Resolver/Codegen)。
3. **Clean up**: 移除 `hash_basic.lcy` 中的 `@expect-error` 和 TODO 标记。
4. **Implement**: 推进到 HashMap 实现 (Sprint 13 Phase 2)。
