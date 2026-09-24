# 自举状态

更新：2026-09-23

## 当前结论

- Rust 母体编译器已经形成端到端的语法、语义、单态化、LLVM codegen、CLI 和 runtime 链路，并已按冻结语法固化为 stage0。后续只修复参考行为缺陷或自举阻塞，不再并行扩展语言表面。
- `lencyc/` 已打通 `Read -> Lex -> Parse -> Resolve -> Emit(AST/LIR)`，并具备依赖 Rust LIR backend 的 stage1/stage2/stage3 收敛检查。
- `bootstrap-check` 已能由 stage1 生成 stage2、由 stage2 生成 stage3，并通过编译器 LIR、样例 LIR 与 smoke pipeline 的收敛比较。
- 当前主线是让 Lency 自举编译器完成并稳定最基础的自举子集，而不是继续扩展 Rust 母体。
- selfhost 前端覆盖面已明显大于后端可运行子集；它目前不是普通程序的默认编译器。下一步应由真实自举或 runtime 用例驱动缺口，不再以 parser 外形为进度指标。

## 已具备的 selfhost 基线

### 前端与语义

- 基础字面量、表达式、调用、成员访问和常用控制流。
- 函数、`struct`、`impl`、`const`、`import`、`extern`、`enum`、`match` 的解析入口。
- 名称解析、作用域、基础类型一致性、函数签名与 return 约束。
- enum 构造、payload、嵌套模式、guard、重复与穷尽性检查。
- 非 `std.*` 模块加载，以及 `std.*` 源码签名导入。
- 基础 nullable 签名和普通 enum 构造/匹配语义。

### LIR 与运行

- 基础表达式、控制流和用户函数 LIR。
- non-generic struct 字面量、字段读写和跨函数传递的最小链路。
- literal、wildcard、guard 和递归 enum payload 的 match lowering。
- enum runtime ABI，以及若干字符串、文件系统和转换 builtin。
- LIR emitter 使用结构化结果报告 lowering 错误；未知表达式、语句和运算符不再生成 `expr_unknown` 等伪成功占位，过渡 Rust LIR backend 同样拒绝该占位。
- Rust LIR backend 负责当前 `.lir -> LLVM -> executable` 过渡链路。

当前的 stage2/stage3 是混合自举：Lency 编译器负责处理自身并发射 LIR，Rust 母体仍负责把 LIR 构建和链接为下一阶段可执行文件。这证明了 LIR 层的自处理与收敛，但不等于已经完成独立自举。

以上能力以 `tests/example/` 中已有回归为准。前端能够接受某种语法，不代表 selfhost 已能把它生成并运行。

## 当前缺口

- selfhost codegen/runtime 仍不足以承载完整标准库和真实程序。
- selfhost emitter 的未知节点占位和未知函数 ABI 类型均已改为显式失败；下一步应继续收紧兼容性类型推断等宽松回退。
- 泛型签名目前只保留供类型传播使用的名称信息；generic struct 实例化、完整 impl method、trait 和更完整 member lowering 尚未形成稳定端到端子集。
- resolver 中仍存在兼容性的 `TYPE_UNKNOWN` 路径，可能弱化部分诊断。
- Rust 母体与 selfhost 的顶层语法接受范围仍不完全一致。
- Rust 母体和 selfhost 均不再预置 `Result`、`Ok`、`Err`。它们只能来自普通用户类型或 enum 项；导入 `std.core` 不会隐式注入这些名称。
- Rust 母体的冻结语法已改为类型在前的显式局部变量、C 风格泛型调用、`vec[...]` 字面量和 `int[3]` 固定数组；`=>` 仅保留给 match。旧的 `var x: T`、`::<T>`、`vec![]` 与 Rust 风格闭包已退出公开语法。
- Rust 母体的 runtime intrinsic 已统一解析为普通 `Call` AST，并通过全局函数符号完成名称与类型检查；专用处理只保留在集中 lowering 边界。字符串动态索引也已补齐越界 panic，不再静默读取越界内存。
- 普通调用现在统一执行参数数量和参数类型检查，嵌套调用也不再绕过强静态约束。由此暴露出的 selfhost 无类型空 Vec 已全部改为 `Vec<T> name = vec[]`，selfhost parser/AST/resolver/LIR 同步保留显式局部变量类型，stage2/stage3 继续收敛。
- selfhost resolver 的括号表达式会继续传播内部类型；未知表达式 kind 会显式报错，不再统一降级为 `TYPE_UNKNOWN`。
- selfhost resolver 会先登记导入模块中的类型名，再严格解析字段、枚举载荷和函数签名；同模块前向类型引用仍然有效，未知类型不再静默降级。
- selfhost 签名校验会递归检查泛型参数中的类型名，并在当前模块与导入模块间共用同一规则；显式声明的 `T` 等泛型参数仍按声明作用域放行。
- selfhost resolver 已为缺失子节点、未知 literal token 和不支持运算符补齐显式诊断，畸形表达式 AST 不再静默降级为 `TYPE_UNKNOWN`。
- 检查输出以 `[xtask]`、`[rust/cargo]`、`[lency/rust-host]`、`[lency/selfhost]` 标记执行来源；交互终端中使用颜色区分，`NO_COLOR` 或非终端输出保持纯文本。
- 块内声明和顶层声明仍存在中间表示上的双轨边界。
- Lency 尚不支持 `/* ... */` 块注释。

## 近期验收目标

1. `cargo run -p xtask -- check-lency` 稳定通过。
2. `cargo run -p xtask -- bootstrap-check` 的 stage2/stage3 LIR 保持收敛。
3. 每个新增 selfhost 能力都有最小正/负例或 runtime 端到端用例。
4. 只补阻塞编译器自举的语言与后端能力，暂不追求与 Rust 母体全面对齐。
5. 普通 Lency 程序继续以 Rust 母体为默认实现，直到 selfhost 达到独立、可诊断且有足够端到端覆盖的可用门槛。

## 事实来源

- 当前实现：`lencyc/`、`lib/`
- 当前回归：`tests/example/`
- 检查编排：`xtask/src/checks/`
- 历史变更：Git 提交记录

本文只记录仍影响下一步决策的状态；已完成事项不在这里累计为长期流水账。
