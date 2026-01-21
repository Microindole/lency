# 函数

## 函数定义

```lency
返回类型 函数名(参数类型 参数名, ...) {
    // 函数体
}
```

### 示例

```lency
int add(int a, int b) {
    return a + b
}

void greet(string name) {
    print("Hello, " + name)
}

int main() {
    var result = add(1, 2)
    greet("World")
    return 0
}
```

## 泛型函数

```lency
T identity<T>(T value) {
    return value
}

int main() {
    var x = identity::<int>(42)
    var s = identity::<string>("hello")
    return 0
}
```

## 方法（结构体上的函数）

```lency
struct Point {
    int x
    int y
}

impl Point {
    int distance_from_origin() {
        return this.x * this.x + this.y * this.y
    }
    
    void move_by(int dx, int dy) {
        this.x = this.x + dx
        this.y = this.y + dy
    }
}
```

## 外部函数

与 C 代码链接：

```lency
extern int strlen(string s)
extern void exit(int code)
```
