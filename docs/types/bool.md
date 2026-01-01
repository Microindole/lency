# Bool 类型

**Beryl 布尔类型文档**

---

## 概述

`bool` 是 Beryl 的布尔类型，只有两个值：`true` 和 `false`。

---

## 字面量

```beryl
var flag = true;
var check = false;
```

---

## 运算符

### 逻辑运算

| 运算符 | 名称 | 描述 | 示例 |
|--------|------|------|------|
| `&&` | 逻辑与 | 两个都为true时返回true | `true && false` → `false` |
| `\|\|` | 逻辑或 | 任一为true时返回true | `true \|\| false` → `true` |
| `!` | 逻辑非 | 取反 | `!true` → `false` |

**示例**:
```beryl
var a = true && false;  // false
var b = true || false;  // true
var c = !true;          // false
```

### 比较运算

所有比较运算返回 `bool` 类型：

```beryl
var x = 5 > 3;      // true
var y = 10 == 10;   // true
var z = 7 != 8;     // true
var w = 4 >= 4;     // true
```

---

## 运算符优先级

从高到低：
1. `!` (一元逻辑非)
2. `>`, `<`, `>=`, `<=` (比较)
3. `==`, `!=` (相等性)
4. `&&` (逻辑与)
5. `||` (逻辑或)

**示例**:
```beryl
var result = !true && false || true;
// 等价于: ((!true) && false) || true
// 结果: true
```

---

## 控制流

布尔值主要用于控制流语句：

### if 语句
```beryl
if condition {
    // 当 condition 为 true 时执行
}
```

### while 循环
```beryl
while flag {
    // 当 flag 为 true 时循环
}
```

---

## 类型系统

### 类型注解
```beryl
bool is_valid(int age) {
    return age >= 0 && age <= 120;
}
```

### 类型安全
Beryl 是静态类型语言，bool 类型不会隐式转换：

```beryl
// ❌ 错误示例
int x = true;           // 类型错误
bool y = 5;             // 类型错误
var z = true + false;   // bool 不支持算术运算
```

---

## LLVM 映射

内部实现细节（供编译器开发者参考）：

- Beryl `bool` → LLVM `i1`
- `true` → `i1 1`
- `false` → `i1 0`

**LLVM IR 示例**:
```llvm
%flag = alloca i1, align 1
store i1 true, ptr %flag, align 1
```

---

## 示例程序

### 基础示例
```beryl
int main() {
    var flag = true;
    if flag {
        return 1;
    }
    return 0;
}
```

### 逻辑运算示例
```beryl
bool check_range(int x, int min, int max) {
    return x >= min && x <= max;
}

int main() {
    var in_range = check_range(5, 1, 10);  // true
    if in_range {
        return 0;
    }
    return 1;
}
```

### 复杂条件示例
```beryl
bool is_valid_user(int age, bool verified) {
    return age >= 18 && verified;
}

int main() {
    if is_valid_user(25, true) {
        return 0;  // 成功
    }
    return 1;  // 失败
}
```

---

## 最佳实践

1. **使用描述性变量名**
   ```beryl
   var is_valid = check(x);  // ✅ 好
   var v = check(x);          // ❌ 不好
   ```

2. **避免冗余比较**
   ```beryl
   if flag == true { ... }    // ❌ 冗余
   if flag { ... }            // ✅ 简洁
   ```

3. **利用短路求值**
   ```beryl
   if expensive_check() && quick_check() {
       // expensive_check 为 false 时，quick_check 不会执行
   }
   ```

---

**参考**: [Beryl Language Specification](../../assets/design_spec.md)
