# 结构体

## 定义

```lency
struct Point {
    int x
    int y
}
```

## 创建实例

```lency
var p = Point { x: 10, y: 20 }
print(p.x)  // 10
```

## 方法

使用 `impl` 块为结构体添加方法：

```lency
impl Point {
    int distance_squared() {
        return this.x * this.x + this.y * this.y
    }
    
    void translate(int dx, int dy) {
        this.x = this.x + dx
        this.y = this.y + dy
    }
}

var p = Point { x: 3, y: 4 }
print(p.distance_squared())  // 25
```

## 泛型结构体

```lency
struct Box<T> {
    T value
}

impl<T> Box<T> {
    T get() {
        return this.value
    }
}

var box = Box::<int> { value: 42 }
print(box.get())  // 42
```

## Trait 实现

```lency
trait Printable {
    void print_self()
}

impl Printable for Point {
    void print_self() {
        print(this.x)
        print(this.y)
    }
}
```
