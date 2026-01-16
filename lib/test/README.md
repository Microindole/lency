# lib/test - Lency 测试框架

测试框架提供断言工具和测试辅助函数。

## 模块

### assert.lcy
提供各种断言函数用于测试：

**基础断言**
- `assert(condition, message)` - 断言条件为真
- `assert_true(condition, message)` - 明确断言为真
- `assert_false(condition, message)` - 明确断言为假

**相等性断言**
- `assert_eq_int(actual, expected, message)` - 整数相等
- `assert_eq_string(actual, expected, message)` - 字符串相等
- `assert_eq_float(actual, expected, epsilon, message)` - 浮点数近似相等

**集合断言**
- `assert_vec_len(vec, expected_len, message)` - 向量长度

**测试辅助**
- `test_passed(test_name)` - 打印通过消息
- `test_failed(test_name, reason)` - 打印失败消息

## 使用示例

```lency
// 注意：由于 import 系统限制，暂时无法导入 test.assert
// 推荐使用 std.core 中已有的 assert_* 函数

import std.core

int main() {
    // 使用 core 中的断言
    assert_eq_int(1 + 1, 2, "basic math")
    assert_true(true, "should be true")
    
    return 0
}
```

## 注意事项

- 断言失败会打印错误信息但不会终止程序（需要 panic 支持）
- 建议在测试中手动检查断言后返回非零退出码
