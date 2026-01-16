# Sprint 13 总结 & Sprint 14 规划

> **Sprint 13 已完成**: Import 系统 + Trait 系统  
> **Sprint 14 目标**: 架构重构 + 自举准备

---

## ✅ Sprint 13 完成内容

### Import 系统 ✅
- [x] import 语法解析
- [x] 模块加载和解析
- [x] 循环依赖检测
- [x] 测试：import_basic.lcy, import_group.lcy

### Trait 系统 ✅
- [x] trait 定义和解析
- [x] impl Trait for Type
- [x] 泛型约束（T: Trait）
- [x] 测试：trait_basic.lcy, hash_basic.lcy

### 标准库扩展 ✅
- [x] 类型转换 FFI
- [x] 字符串函数扩展
- [x] std/math 模块
- [x] lib/test 测试框架
- [x] 文件系统操作

**测试通过**: 48个集成测试 ✅

---

## 🎯 Sprint 14: 架构重构与自举第一步

### 目标 1: 模块化重构

**问题**: 
- `lency_monomorph` 只是空壳
- `lency_diagnostics` 未实现
- 违反模块化原则

**任务**:
1. **迁移单态化** (3天)
   - [ ] 移动 sema/monomorphize → lency_monomorph
   - [ ] 更新依赖关系
   - [ ] 测试验证

2. **实现统一诊断** (3天)
   - [ ] 实现 Diagnostic 类型
   - [ ] 集成到 syntax/sema/codegen
   - [ ] 改进错误信息

### 目标 2: 自举准备 - HashMap<String, V>

**当前问题**: HashMap 只支持整数键

**实现计划** (1周):

```lency
// 运行时扩展
lency_runtime/src/hashmap.rs:
  - lency_hashmap_string_new
  - lency_hashmap_string_insert
  - lency_hashmap_string_get
  // ... 其他方法

// 代码生成
lency_codegen/src/expr/hashmap.rs:
  - 识别 hashmap_string_* 调用
  - 生成对应 FFI 调用

// 标准库
lib/std/collections.lcy:
  struct HashMapStringInt {
      int handle
  }
  
  impl HashMapStringInt {
      // 包装器方法
  }
```

**测试**:
```lency
// tests/integration/collections/hashmap_string.lcy
var map = HashMapStringInt::new()
map.insert("key1", 100)
assert_eq(map.get("key1"), 100)
```

### 目标 3: 自举准备 - Result<T, E> 方法

**当前问题**: Result 只有语法，没有实用方法

**实现计划** (1周):

```lency
// lib/std/result.lcy
enum Result<T, E> {
    Ok(T),
    Err(E)
}

impl<T, E> Result<T, E> {
    bool is_ok() {
        match this {
            Ok(_) => true,
            Err(_) => false
        }
    }
    
    bool is_err() {
        return !this.is_ok()
    }
    
    T unwrap() {
        match this {
            Ok(val) => val,
            Err(_) => {
                print("unwrap on Err!\n")
                // FIXME: 需要 panic
                return val  // 编译错误，故意的
            }
        }
    }
    
    T unwrap_or(T default_val) {
        match this {
            Ok(val) => val,
            Err(_) => default_val
        }
    }
}
```

**挑战**:
- 需要泛型 enum 的方法调用
- 需要泛型匹配

---

## 📋 Sprint 14 详细任务清单

### Week 1: 重构

**Day 1-2: 迁移 monomorph**
- [ ] 复制代码到 lency_monomorph
- [ ] 更新 Cargo 依赖
- [ ] 修改 driver 调用

**Day 3-4: 实现 diagnostics**  
- [ ] 定义核心类型
- [ ] 集成到各模块
- [ ] 更新错误信息

**Day 5: 测试和文档**
- [ ] 运行完整测试
- [ ] 更新文档
- [ ] Code review

### Week 2: HashMap<String>

**Day 1-2: 运行时实现**
- [ ] 实现 string hash 函数
- [ ] 实现 hashmap_string_* FFI
- [ ] 单元测试

**Day 3-4: 代码生成**
- [ ] 扩展 hashmap.rs
- [ ] 标准库包装
- [ ] 集成测试

**Day 5: 优化和文档**
- [ ] 性能测试
- [ ] 文档和示例

---

## 🎯 成功标准

**Sprint 14 完成后**:
- ✅ 架构清晰，职责分明
- ✅ HashMap 支持 String 键
- ✅ 统一的错误诊断系统
- ✅ 所有测试通过

**自举准备度**: 45% → 60%

---

## 🚀 Sprint 15 预告

### 重点：Iterator + Result 方法

1. **Iterator Trait**
   - trait Iterator<T>
   - Vec<T> impl Iterator
   - 基础方法（map, filter）

2. **Result<T, E> 完善**
   - 方法实现
   - ? 操作符（可选）

3. **字符串格式化**
   - format! 宏（或函数）
   - 字符串插值

**预期**: 自举准备度 60% → 75%

---

## 💡 关键里程碑

```
Sprint 13 ✅ - Import + Trait (已完成)
Sprint 14 ⏳ - 架构 + HashMap<String> (进行中)
Sprint 15 📋 - Iterator + Result
Sprint 16 📋 - Regex + Format
---
Sprint 17-20 📋 - 用 Lency 编写词法分析器
Sprint 21+ 📋 - 完整的自举编译器
```

**预计自举开始**: 3个月后（Sprint 17）
