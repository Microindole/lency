# lency_runtime

提供生成程序调用的 Rust 运行时和稳定 C ABI。它同时构建为 `rlib` 与动态库。

## 结构

| 路径 | 职责 |
|---|---|
| `src/lib.rs` | ABI 导出、基础内存和容器入口 |
| `src/string.rs` | 字符串分配、查询与转换 |
| `src/file.rs` | 文件系统运行时操作 |
| `src/enum_value.rs` | enum payload 运行时表示 |
| `src/hashmap.rs` | 整数键 HashMap |
| `src/hashmap_string.rs` | 字符串键 HashMap |
| src/memory.rs 与 src/memory/tests.rs | 手动分配登记、匹配分配器的浅层释放及回归 |
| src/trace.rs | Lency 源码调用帧登记与输出 |

## 边界

codegen 中的声明必须与这里的符号名、参数、返回类型和所有权约定完全一致。修改 ABI 时必须同时检查 codegen、LIR backend 和端到端运行测试。

panic 入口将消息、可用源行号和已登记的 Lency 源码调用栈写到标准错误，刷新输出并以非零状态结束进程，不提供捕获或恢复接口。旧的单参数 ABI 委托带行号入口，以维持过渡 LIR 后端兼容。母体 AST 路径提供函数、原始文件和行列映射，LIR 路径尚未提供同等信息。设置 LENCY_NATIVE_BACKTRACE 环境变量可额外输出本机回溯。

内存由程序手动管理。分配登记保存每个存活对象对应的释放函数，使 C 分配、Rust Box 和 CString 使用匹配的释放方式；登记不是 GC，不扫描引用也不自动销毁对象。统一释放及原有专用释放入口均撤销登记，未登记地址触发 panic。释放容器会销毁内部缓冲区，但不会递归释放元素引用的独立对象。未复用地址上的重复释放可被发现，裸地址复用后的旧别名及普通读取时的释放后使用不受此机制保护。

分配数量诊断入口只统计登记表中的存活对象，不等同于进程内存用量。stage0 验收用它检查标准库重复调用前后的余额。动态集合越界和空集合弹出使用统一 Lency panic，不通过宿主 Rust unwind 穿越 C ABI，也不返回伪造的零值。
