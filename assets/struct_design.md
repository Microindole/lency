# Beryl 结构体系统设计规范

## 设计哲学

**核心原则**: 组合优于继承 (Composition over Inheritance)

Beryl 选择**组合模式**而非传统类继承，采用 **impl 块** + **隐式 this** 的设计。

## 语法设计

### 1. 结构体定义

```beryl
struct Point {
    int x
    int y
}

struct Circle {
    Point center    // 组合
    int radius
}
```

### 2. 方法定义（impl 块 + 隐式 this）

```beryl
impl Point {
    // 方法：无需 self 参数
    // 直接访问字段 x, y（编译器自动理解为 this.x, this.y）
    int getX() {
        return x
    }
    
    float distance(Point other) {
        var dx = x - other.x
        var dy = y - other.y
        return sqrt(dx*dx + dy*dy)
    }
    
    // 如需明确引用整个对象，可用保留字 this（可选）
    Point copy() {
        return this  // 返回自身副本
    }
}
```

**关键特性**：
- ✅ 使用 impl 块组织方法
- ✅ **无需显式 self 参数**
- ✅ 裸标识符 `x` 自动解析为 `this.x`
- ✅ `this` 作为保留字可选使用

### 3. 结构体实例化

```beryl
// 字面量语法
var p = Point { x: 10, y: 20 }

// 嵌套初始化
var c = Circle {
    center: Point { x: 0, y: 0 },
    radius: 5
}
```

### 4. 字段访问

```beryl
print(p.x)           // 读取字段
p.x = 15             // 修改字段
print(p.getX())      // 调用方法
print(c.center.x)    // 嵌套访问
```

## 完整示例

```beryl
// 定义结构体
struct Vec2 {
    float x
    float y
}

// 实现方法
impl Vec2 {
    float length() {
        return sqrt(x*x + y*y)  // 直接访问 x, y
    }
    
    Vec2 add(Vec2 other) {
        return Vec2 {
            x: x + other.x,
            y: y + other.y
        }
    }
    
    Vec2 scale(float factor) {
        return Vec2 {
            x: x * factor,
            y: y * factor
        }
    }
}

int main() {
    var v1 = Vec2 { x: 3.0, y: 4.0 }
    var v2 = Vec2 { x: 1.0, y: 0.0 }
    
    print(v1.length())      // 5.0
    
    var sum = v1.add(v2)
    print(sum.x)            // 4.0
    
    var scaled = v1.scale(2.0)
    print(scaled.x)         // 6.0
    
    return 0
}
```

## 组合模式示例

```beryl
struct Transform {
    Vec2 position
    Vec2 scale
    float rotation
}

impl Transform {
    void translate(Vec2 offset) {
        position = position.add(offset)  // position 是 this.position
    }
    
    Vec2 getPosition() {
        return position
    }
}

int main() {
    var t = Transform {
        position: Vec2 { x: 0.0, y: 0.0 },
        scale: Vec2 { x: 1.0, y: 1.0 },
        rotation: 0.0
    }
    
    var offset = Vec2 { x: 10.0, y: 5.0 }
    t.translate(offset)
    
    print(t.position.x)  // 10.0
    
    return 0
}
```

## 设计对比

| 特性 | Beryl | Rust | Go | C++ |
|------|-------|------|-----|-----|
| 继承 | ❌ | ❌ | ❌ | ✅ |
| 组合 | ✅ | ✅ | ✅ | ✅ |
| 方法定义 | impl 块 | impl 块 | 接收者 | 成员函数 |
| self 参数 | ❌ 隐式 | ✅ 显式 | ✅ 显式 | ❌ 隐式 |
| 字段访问 | `x` | `self.x` | `r.x` | `x` 或 `this->x` |

## 与现有特性集成

### 结构体与空安全

```beryl
struct User {
    string name
    int age
}

int main() {
    User u = User { name: "Alice", age: 30 }
    User? maybe = null
    
    if maybe != null {
        print(maybe.name)  // 智能转换后安全
    }
    
    return 0
}
```

### 结构体与数组

```beryl
struct Point {
    int x
    int y
}

int main() {
    var points: [3]Point = [
        Point { x: 0, y: 0 },
        Point { x: 1, y: 1 },
        Point { x: 2, y: 2 }
    ]
    
    for p in points {
        print(p.x + p.y)
    }
    
    return 0
}
```

## 实现计划

### Phase 1: 基础结构体（当前）
1. AST 扩展：`Decl::Struct`
2. Parser：解析 struct 定义和字面量
3. Sema：结构体类型注册和字段检查
4. Codegen：LLVM struct type 生成

### Phase 2: impl 块与方法
1. AST 扩展：`Decl::Impl`
2. Parser：解析 impl 块
3. Sema：方法查找、隐式 this 处理
4. Codegen：方法调用展开

### Phase 3: 高级特性
1. 嵌套结构体
2. 结构体与空安全集成
3. 性能优化

## 内存与性能

- **内存布局**: 兼容 C ABI
- **传递语义**: 默认值传递
- **方法调用**: 零开销抽象（静态分发）
- **编译展开**: `p.getX()` → `Point_getX(&p)`

## 关键设计决策

1. ✅ **impl 块**：清晰的命名空间分离
2. ✅ **隐式 this**：去除冗余的 self 参数
3. ✅ **组合优于继承**：避免继承复杂度
4. ✅ **简洁语法**：符合 Beryl 哲学

---

**状态**: 设计已定稿，开始实现
**版本**: v1.0
**日期**: 2026-01-02
