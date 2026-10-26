# 表达式

表达式产生值，语句组织声明、赋值和执行顺序。调用无值函数用于执行操作，不能据此产生集合元素或条件值。

## 字面量

基础示例包含整数、带小数点的浮点数、布尔值和双引号字符串。空值需进入可空类型。

```lency
int main() {
    var count = 12
    var ratio = 1.5
    var enabled = true
    var label = "Lency\n"
    int? missing = null
    print(label)
    return count - 12
}
```

字符串常用转义包括换行、制表符、双引号和反斜线。字符字面量与科学计数法在自举词法回归中有覆盖，Rust 母体的词法接受范围并不相同；它们不作为本页两端通用示例。

## 运算与优先级

以下运算按从高到低的层级组织；括号可以明确改变分组。

```text
调用、成员访问、索引
一元取负、逻辑取反：- !
乘除与取余：* / %
加减：+ -
比较与相等：< <= > >= == !=
逻辑与：&&
逻辑或：||
```

比较与相等在当前母体解析器中属于同一层。避免连续比较；需要范围判断时分别比较后再进行逻辑运算。

```lency
int main() {
    var total = 2 + 3 * 4
    var inside = total >= 10 && total < 20
    var outside = !inside
    if outside {
        return 1
    }
    return (total - 14)
}
```

优先级只决定解析分组，不表示自举后端已经实现列表中的所有运算。例如取余具有母体语法入口，但不属于当前自举运算符的完整运行子集。

自举中的算术操作会检查操作数类型；不要依赖隐式的整数与布尔转换。条件必须产生非可空布尔值，见[控制流](./control-flow.md)。

## 调用、成员和索引

普通调用按位置传入参数。点号访问结构体字段或调用接收者的方法，方括号读取集合元素。

```lency
int add(int left, int right) {
    return left + right
}

int main() {
    var values = vec[add(1, 2), 4]
    values.push(5)
    var first = values[0]
    return first - 3
}
```

调用检查参数数量和类型。字段或方法必须属于对应接收者；不同类型的同名方法不是同一个签名。索引的具体边界行为见[Vec](../stdlib/vec.md)和[字符串](../stdlib/string.md)。

泛型实参使用紧跟函数名的尖括号，见[函数](./functions.md)。可空类型后的问号不可以放到任意调用结果后充当隐式错误传播。

## 赋值与匹配结果

普通赋值使用等号；相等比较使用双等号。基础示例使用展开的赋值，不依赖复合赋值或自增语法。

```lency
int main() {
    var count = 0
    count = count + 1
    if count == 1 {
        return 0
    }
    return 1
}
```

匹配是产生结果的表达式，分支结果需要满足类型约束。分支箭头仅用于匹配，不用于声明匿名函数；具体形式见[控制流](./control-flow.md)。
