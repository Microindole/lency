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

母体后端保留分支和基本块的终止状态。两侧分支均直接返回时，不需要额外默认返回，也不生成仍可继续执行的空合流块。

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

函数签名不增加错误声明或结果包装。检测到运行时错误时，使用 panic 报告原因并终止；没有捕获、恢复或传播语法。

```lency
int divide(int left, int right) {
    if (right == 0) {
        panic("除数不能为零")
    }
    return left / right
}

int main() {
    return divide(10, 2) - 5
}
```

主动 panic 和自动除零、越界检查使用统一诊断出口，输出到标准错误，并以非零状态退出。母体主动调用保留当前编译输入的行号，本机回溯只能提供可用的机器帧；原始多文件定位和完整 Lency 调用栈尚未完善。

正常的查询缺失仍可以通过可空返回值表达；这不等于将错误静默替换为空值。错误与正常业务结果的区别由 API 契约规定，不能使用未声明的全局错误状态。
