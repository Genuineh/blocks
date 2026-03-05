# Rust Workspace 架构草图（MOC 模型）

## 1. 目标

固定最新的 Rust workspace 边界，确保仓库从较早的顶层原型迁回“`moc` 模型”：

- `block` 是库能力
- `moc` 是最终交付单元
- `moc` 主入口是自由代码

## 2. 当前推荐边界

- `blocks-contract`
  - 负责 `block.yaml` 和未来 `moc.yaml` 的基础契约模型、字段校验、错误类型
  - 不负责文件扫描和执行
- `blocks-registry`
  - 负责发现公共 `block`
  - 不负责执行，也不负责 `moc` 编排
- `blocks-runtime`
  - 只负责单 `block` 执行胶水
  - 不负责 `moc` 级调度
- `blocks/*/rust`
  - 对可直接复用的 Rust `block`，应逐步提供独立 crate 入口
  - `block.yaml` 继续保留为 AI 和校验使用的描述层
- `blocks-moc`
  - 当前承载 `moc` 描述层
  - 负责 `moc.yaml` 解析、类型校验、依赖 `block` 校验、最小协议校验、`internal_blocks` 布局校验
  - 只做描述层和辅助层，不做最终运行入口
- `blocks-cli`
  - 负责 list/show/run/validate 等入口
  - 当前提供 `moc run` 与 `moc verify` 入口：前者只分发真实运行路径，后者显式执行 `verification` 流
  - 对 `blocks run` / `moc verify` 保留最小本地 block runner
  - 不承载 `moc` 业务逻辑
- `mocs/*`
  - 承载真实 `moc` 代码入口
  - 可以自由调用公共 `block` 和内部私有 `block`

## 3. 关键依赖方向

允许：

```text
blocks-cli -> blocks-registry
blocks-cli -> blocks-runtime
blocks-cli -> blocks-moc
blocks-cli -> blocks/*/rust (debug-run path only)
blocks-registry -> blocks-contract
blocks-runtime -> blocks-contract
blocks-moc -> blocks-contract
blocks-moc -> blocks-registry
mocs/* -> blocks/*/rust (preferred direct dependency)
mocs/* -> blocks-registry (optional, descriptor lookup only)
mocs/* -> blocks-moc (optional, descriptor validation only)
```

禁止：

- `blocks-runtime` 依赖 `blocks-registry`
- `blocks-moc` 直接承担最终运行
- `CLI` 回收 `moc` 主逻辑
- 新增 Rust `block` 默认只接入单一中央分发层

## 4. 当前实现与目标差距

当前仓库仍有几个过渡问题：

- 目录和命名已切到 `mocs/`、`moc.yaml`、`blocks-moc`
- 描述层已经支持类型、协议和可选校验流，但 `verification.flows` 仍是过渡能力
- 示例已同时覆盖 `backend_app(console)` 和 `rust_lib`
- 当前公共 Rust block 已具备独立 crate 入口，主要示例已直接依赖这些 crate
- `moc run` 已可分发到 descriptor-only Rust backend moc 的真实入口
- 当前已包含最小 `frontend_app` 结构样例，并已提供本地静态预览与真实 Tauri 宿主运行入口

因此，下一轮整改应优先做“行为模型回正”，再继续扩展功能。

## 5. MOC 层约束

- `moc.main` 是自由代码，不是步骤图解释器
- `moc.main` 可以自由调用 `block`
- `moc.main` 可以自由调用内部私有 `block`
- 一个 `moc` 只允许一种交付类型
- 多服务、多终端应拆成多个 `moc`

## 6. 非目标

当前不在这一轮引入：

- 复杂工作流引擎
- 跨进程自动编排器
- 多 moc 自动部署系统
- 完整 BCL 编译器

先把 `moc` 模型走正，再向上扩展。
