# Lency Monomorph 重构计划

## 当前状态

**问题**: 单态化功能在 `lency_sema/src/monomorphize/` 中，但应该在独立的 `lency_monomorph` crate

**影响**:
- 模块职责不清晰
- sema 过于臃肿
- 违反单一职责原则

---

## 重构步骤

### Step 1: 迁移代码

**迁移文件**:
```
lency_sema/src/monomorphize/
├── mod.rs         → lency_monomorph/src/lib.rs
├── collector.rs   → lency_monomorph/src/collector.rs
├── mangling.rs    → lency_monomorph/src/mangling.rs
├── rewriter.rs    → lency_monomorph/src/rewriter.rs
└── specializer.rs → lency_monomorph/src/specializer.rs
```

### Step 2: 更新依赖

**lency_monomorph/Cargo.toml**:
```toml
[dependencies]
lency_syntax = { workspace = true }
```

**lency_driver/Cargo.toml**:
```toml
[dependencies]
lency_monomorph = { workspace = true }
```

### Step 3: 更新 driver

**lency_driver/src/lib.rs**:
```rust
use lency_monomorph::Monomorphizer;

// 在编译流程中
let mono_program = {
    let mut monomorphizer = Monomorphizer::new();
    monomorphizer.process(sema_program)
};
```

### Step 4: 删除旧代码

```bash
rm -rf crates/lency_sema/src/monomorphize
# 从 lency_sema/src/lib.rs 移除 pub mod monomorphize
```

---

## 测试验证

```bash
cargo build
cargo test
./scripts/run_checks.sh
```

---

## 预期结果

- ✅ lency_monomorph 成为独立功能模块
- ✅ lency_sema 职责更清晰
- ✅ 所有测试通过
- ✅ 架构更模块化
