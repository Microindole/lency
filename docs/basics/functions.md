# 函数

## 签名与调用

函数签名依次写返回类型、函数名和参数列表。每个参数也将类型放在名称前；实参按位置传递。

```lency
int add(int left, int right) {
    return left + right
}

int main() {
    var answer = add(20, 22)
    return answer - 42
}
```

函数调用必须符合参数数量和类型约束。嵌套调用的返回值同样参与检查。

## 返回路径

有返回值的函数需要在所有可达退出路径提供值；单个分支中有返回，不代表整个函数一定返回。

```lency
int absolute(int value) {
    if value < 0 {
        return -value
    }
    return value
}

int main() {
    return absolute(-3) - 3
}
```

无值函数可以执行到函数末尾隐式返回，也可以用无值返回提前结束。无值返回后使用分号，避免与后续表达式产生解析歧义。

```lency
void show(bool enabled) {
    if !enabled {
        return;
    }
    print("ready\n")
}

int main() {
    show(true)
    return 0
}
```

自举编译器对普通函数和结构体方法共用返回规则。有值函数漏返回时产生诊断，LIR 发射失败；不会通过补默认返回值伪造成功。循环按可能不执行处理，循环体内含返回不能单独证明返回路径完整。

母体后端在两个条件分支均直接返回的部分函数中，会生成未终止的合流块并导致构建失败。这是已验证的实现缺陷，不意味着语言要求额外默认返回；本页示例使用提前返回后接普通返回的等价形式。
<!-- FIXME: 母体 codegen 的双分支直接返回路径仍生成未终止的合流块。 -->

## 泛型

泛型形参声明在函数名后，调用时使用普通尖括号传入类型。

```lency
T identity<T>(T value) {
    return value
}

int main() {
    var answer = identity<int>(42)
    return answer - 42
}
```

泛型示例属于母体能力。自举前端能够解析泛型声明和调用，但完整实例化尚不属于稳定端到端能力。不得仅凭解析成功认定泛型程序可以自举运行。

## 外部函数

外部声明给出签名但没有函数体，末尾必须有分号。检查声明不等于提供实现；实际调用时还需链接对应外部符号。

```lency
extern int native_size(int value);

int main() {
    return 0
}
```

结构体方法通过实现块定义，写法与接收者约束见[结构体](../types/structs.md)。

## 错误与普通返回值

函数签名不附加异常类型、错误效果或隐式传播标记。整数除零等程序错误按设计终止程序，不通过额外错误通道返回。可能发生且需要调用者处理的业务结果，通过普通可空值或自定义枚举返回。

```lency
enum DivideOutcome {
    Value(int),
    DivisionByZero
}

DivideOutcome try_divide(int left, int right) {
    if right == 0 {
        return DivideOutcome.DivisionByZero
    }
    return DivideOutcome.Value(left / right)
}

int main() {
    var outcome = try_divide(10, 0)
    var code = match outcome {
        case DivideOutcome.Value(value) => value,
        case DivideOutcome.DivisionByZero => 0
    }
    return code
}
```

上例的类型和枚举项都是普通用户声明，没有编译器特权。类型名不要求使用某个包装名称，也没有独立的捕获流程。需要区分更多业务结果时扩展枚举，由正常控制流处理。

不提供内建结果类型、异常捕获或后缀问号传播。两套前端目前的枚举构造和匹配形式存在差异，详见[枚举与匹配](../types/enums.md)。
