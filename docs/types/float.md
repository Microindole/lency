# Float 类型

**Beryl 浮点数类型文档**

---

## 概述

`float` 是 Beryl 的浮点数类型，用于表示小数和实数运算。

---

## 字面量

```beryl
var pi = 3.14159;
var e = 2.71828;
var zero = 0.0;
var negative = -5.5;
```

**格式**: 必须包含小数点

```beryl
var x = 3.0;    // ✅ float
var y = 3;      // ✅ int (不是 float)
```

---

## 运算符

### 算术运算

| 运算符 | 描述 | 示例 |
|--------|------|------|
| `+` | 加法 | `3.5 + 2.5` → `6.0` |
| `-` | 减法 | `10.0 - 3.5` → `6.5` |
| `*` | 乘法 | `4.0 * 2.5` → `10.0` |
| `/` | 除法 | `10.0 / 4.0` → `2.5` |
| `-x` | 一元负号 | `-3.14` → `-3.14` |

**示例**:
```beryl
var a = 10.5 + 2.5;      // 13.0
var b = 10.0 / 3.0;      // 3.333...
var c = -5.5;            // -5.5
```

### 比较运算

所有比较运算返回 `bool` 类型：

```beryl
var x = 5.5 > 3.0;       // true
var y = 10.0 == 10.0;    // true
var z = 7.2 != 8.1;      // true
var w = 4.5 >= 4.5;      // true
```

---

## 类型提升

Beryl **不允许隐式类型转换**，但支持**自动类型提升**：

### 规则

当 `int` 和 `float` 混合运算时，结果自动提升为 `float`：

```beryl
var result = 5 + 2.5;        // int + float → float (7.5)
var product = 4 * 2.5;       // int * float → float (10.0)
var division = 10 / 2.0;     // int / float → float (5.0)
```

### 类型提升表

| 左操作数 | 右操作数 | 结果类型 | 示例 |
|----------|----------|----------|------|
| `int` | `float` | `float` | `5 + 2.5` → `7.5` |
| `float` | `int` | `float` | `2.5 + 5` → `7.5` |
| `float` | `float` | `float` | `2.5 + 3.5` → `6.0` |
| `int` | `int` | `int` | `5 + 2` → `7` |

---

## 类型安全

### 不允许的操作

```beryl
// ❌ 错误：不允许隐式转换
float x = 5;        // int 不能直接赋值给 float
int y = 3.14;       // float 不能直接赋值给 int
```

### 显式类型声明

```beryl
// ✅ 正确：明确类型
float x = 5.0;      // 使用浮点字面量
int y = 3;          // 使用整数字面量
```

---

## LLVM 映射

内部实现细节（供编译器开发者参考）：

- Beryl `float` → LLVM `double` (f64)
- 64 位 IEEE 754 浮点数
- 精度: 约 15-17 位有效数字

**LLVM IR 示例**:
```llvm
%x = alloca double, align 8
store double 3.140000e+00, ptr %x, align 8
```

---

## 浮点数注意事项

### 1. 精度限制

浮点数是近似表示，可能存在精度误差：

```beryl
var x = 0.1 + 0.2;  // 可能不完全等于 0.3
```

**建议**: 避免直接比较浮点数相等性，使用范围比较。

### 2. 特殊值

```beryl
var zero = 0.0;
var negative_zero = -0.0;  // -0 和 +0 在大多数运算中相同
```

---

## 示例程序

### 基础示例
```beryl
int main() {
    var pi = 3.14159;
    var radius = 5.0;
    var area = pi * radius * radius;
    return 0;
}
```

### 类型提升示例
```beryl
int main() {
    var i = 10;      // int
    var f = 3.0;     // float
    var result = i / f;  // int / float → float (3.333...)
    return 0;
}
```

### 比较运算示例
```beryl
int check_range(float x, float min, float max) {
    if x >= min && x <= max {
        return 1;
    }
    return 0;
}

int main() {
    var in_range = check_range(5.5, 0.0, 10.0);
    return in_range;
}
```

---

## 最佳实践

1. **明确小数点**
   ```beryl
   var x = 5.0;    // ✅ 清晰
   var y = 5;      // ✅ 但是 int
   ```

2. **避免精度敏感的相等比较**
   ```beryl
   // ❌ 不推荐
   if x == 0.3 { ... }
   
   // ✅ 推荐
   if x > 0.29 && x < 0.31 { ... }
   ```

3. **利用类型提升**
   ```beryl
   var result = 5 / 2.0;  // 2.5 (float)
   var result2 = 5 / 2;   // 2 (int，整数除法)
   ```

---

## 与其他类型的关系

| 从类型 | 到类型 | 是否允许 | 说明 |
|--------|--------|----------|------|
| `float` | `int` | ❌ | 需要显式转换 |
| `int` | `float` | ❌ | 赋值时不允许，运算时自动提升 |
| `float` | `bool` | ❌ | 不允许 |
| `float` | `string` | ❌ | 未来可能支持 |

---

**参考**: [Beryl Language Specification](../design_spec.md)
