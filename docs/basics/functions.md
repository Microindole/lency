# 函数

函数签名采用“返回类型在前”的形式：

```lency
int add(int left, int right) {
    return left + right
}

void print_twice(string value) {
    print(value)
    print(value)
}
```

泛型参数使用 `<...>`：

```lency
T identity<T>(T value) {
    return value
}

var answer = identity<int>(42)
```

外部函数使用 `extern` 声明：

```lency
extern void exit(int code)
```

## 实现说明

- selfhost 已支持普通函数的解析、签名检查、调用、return 约束和最小 LIR 发射。
- 泛型声明和调用已有解析入口，但泛型实例化尚不是稳定的 selfhost 端到端能力。
- `impl` 方法见[结构体](../types/structs.md)。
