# Sprint 17 Walkthrough (2026-03-02)

## 当前进度
1. P0 完成：Parser 新增 `return` 语句解析路径。
2. AST 已新增 `STMT_RETURN` 节点与工厂函数。
3. AST Printer 已实现：`expr_to_string`/`stmt_to_string`。
4. 自举驱动样例已覆盖 `return` 解析路径（`simple_source` 新增 `return i`），并打印 AST 文本。
5. 验证结果：`./scripts/run_lency_checks.sh` 通过，运行时输出由 `2 statements` 变为 `3 statements`，并显示 AST[0..2]。
6. 工程产物管理优化：`lencyc compile/build` 新增 `--out-dir`，`run_lency_checks.sh` 的自举产物改写入 `target/lencyc_selfhost/`，避免污染仓库根目录。

## 未尽事宜
1. 扩展声明解析（`func/struct/impl` 的最小子集）。
2. 补齐字符串和浮点字面量词法支持。
3. 为 Parser 增加错误恢复同步点（避免单错中断）。

## 阻塞项
1. `return` 无值场景暂无专用节点，当前采用哑元表达式占位。
2. Parser 诊断尚未统一到 Reporter，错误恢复能力有限。

## 增量记录 (2026-03-04)
1. 新增字符串字面量 token：`T_STRING_LITERAL`。
2. Lexer 已支持双引号字符串扫描，并在未闭合场景返回错误 token。
3. Parser `primary` 已接入字符串字面量，统一映射到 `EXPR_LITERAL`。
4. 自举测试新增字符串正/负例，覆盖 `var msg = "hello"`、`print("done")` 与未闭合字符串拒绝路径。
5. Lexer `number()` 已支持浮点字面量扫描（`digits '.' digits`，例如 `3.14`）。
6. 自举测试新增浮点正/负例，覆盖 `3.14`/`0.5` 与 `12.` 拒绝路径。
