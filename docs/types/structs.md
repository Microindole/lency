# 结构体

```lency
struct Pair {
    int left
    int right
}

var pair = Pair { left: 1, right: 2 }
pair.right = 3
var total = pair.left + pair.right
```

方法通过 `impl` 声明：

```lency
impl Pair {
    int sum() {
        return this.left + this.right
    }
}
```

## selfhost 状态

已形成端到端回归的子集：

- non-generic struct 声明和字面量；
- 字段读取与赋值；
- 结构体作为普通函数参数和返回值。

尚未形成稳定端到端子集：

- generic struct；
- 完整 impl method codegen；
- trait 实现。

对应运行回归位于 `tests/example/runtime/lencyc_run_struct_*.lcy`。
