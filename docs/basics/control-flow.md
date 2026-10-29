# 控制流

## 条件分支

条件表达式后紧接花括号块；条件括号可以用于明确分组。分支体始终使用块，不依赖 C 风格的无花括号单条语句。

```lency
int main() {
    var value = 3
    if value > 0 {
        print("positive\n")
    } else {
        print("not positive\n")
    }
    return 0
}
```

自举语义检查要求条件为非可空布尔值，不把数字、字符串、结构体或空值隐式解释为真或假。母体语义也要求布尔条件；实现偏差应视为缺陷，不形成另一套设计规则。

需要多个条件时，可以在否则分支的块内继续写条件分支。自举 parser 还接受连续的否则条件形式；母体当前要求否则分支后直接跟块，两者不能混用。

## 条件循环

```lency
int main() {
    var index = 0
    while index < 3 {
        index = index + 1
    }
    return index - 3
}
```

条件在每次进入循环体前计算，条件为假时离开循环。

## 计数循环

循环头依次包含初始化、条件和更新，循环体使用花括号。母体和自举前端均允许独立选择有无外层括号，以及完整三段之间有无分号。

```lency
int main() {
    var total = 0
    for var index = 0; index < 4; index = index + 1 {
        total = total + index
    }
    return total - 6
}
```

自举 parser 将该形式降为块和条件循环。循环内继续下一次迭代时，仍执行更新表达式。母体还有遍历循环的解析入口；当前自举教程不将遍历形式列入通用可运行子集。

两套前端均接受以下等价循环头，循环体仍使用花括号：

```text
for (int index = 0 index < 4 index = index + 1)
for int index = 0; index < 4; index = index + 1
for int index = 0 index < 4 index = index + 1
```

省略分号时仍必须能确定三个表达式的边界；有歧义的表达式使用括号分组或显式分隔。自举运行子集要求三段完整，初始化可使用显式类型声明、推导声明或赋值；循环初始化中声明的变量不在循环外可见。母体已有空段形式不属于当前自举运行子集。

## 跳出、继续与返回

跳出只退出当前循环；继续跳到当前循环的下一次迭代；函数返回会退出整个函数。循环跳转不能放在循环外。

```lency
int main() {
    var total = 0
    for var index = 0; index < 5; index = index + 1 {
        if index == 1 {
            continue
        }
        if index == 4 {
            break
        }
        total = total + index
    }
    return total - 5
}
```

返回后的语句不会执行。函数返回值和路径要求见[函数](./functions.md)。

## 匹配表达式

匹配根据模式选择分支，并产生结果。母体的分支使用分支关键字；下划线匹配其余情况。

母体示例：

```lency
int main() {
    var input = 1
    var value = match input {
        case 0 => 10,
        case 1 => 20,
        case _ => 30
    }
    return value - 20
}
```

自举前端当前不写母体的分支关键字，支持带布尔条件的守卫。下面是独立的自举前端示例：

```lency-selfhost
int main() {
    var input = 1
    var value = match (input) {
        0 => 10,
        1 if (input > 0) => 20,
        _ => 30
    }
    return value - 20
}
```

守卫的条件括号不可省略。带守卫的分支不保证覆盖所有值，仍需要后续分支兜底。两种写法的差异属于当前实现边界，不代表存在两个已确认的语言规范。

枚举载荷、嵌套模式、重复与穷尽性检查见[枚举与匹配](../types/enums.md)。分支箭头不用于普通函数返回、闭包或错误处理。
