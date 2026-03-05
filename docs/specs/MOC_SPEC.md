# blocks MOC 规范

## 1. 目的

本规范定义 `moc` 在 `blocks` 体系中的角色、类型、目录结构、描述文件和交互边界。

`moc` 的含义是 `my own creation`。

核心原则：

- `block` 是库能力，不是最终交付单元。
- `moc` 才是最终交付单元。
- `moc` 的主入口是自由代码，不是声明式流程图。

## 2. 核心定义

### 2.1 moc

一个 `moc` 是一个单一类型的可交付单元。它可以是库，也可以是应用，但一次只能属于一种类型。

### 2.2 moc 与 block 的关系

- `block` 为 `moc` 提供稳定能力，角色等同于“高约束的库”。
- `moc` 可以在主入口代码中自由调用多个 `block`。
- `moc` 也可以定义仅供自身使用的内部私有 `block`。
- `moc` 不应被限制为只能执行一份线性声明式步骤。

### 2.3 moc main

`moc` 必须有真实代码入口，例如：

- Rust: `src/lib.rs` 或 `src/main.rs`
- Tauri + TypeScript: `src-tauri/` + `src/`

该入口是自由实现区域，可以：

- 按任意合理顺序调用 `block`
- 组织控制流、错误处理和协议交互
- 调用当前 `moc` 的内部私有 `block`

但它仍然必须遵守已声明的契约和类型边界。

## 3. MOC 类型

每个 `moc` 必须声明且只能声明一种类型：

- `rust_lib`
- `frontend_lib`
- `frontend_app`
- `backend_app`

当 `type = backend_app` 时，还应声明：

- `backend_mode: console | service`

规则：

- 一个 `moc` 不得同时是库和应用。
- 一个 `moc` 不得同时承载前端和后端两种最终交付形态。
- 多服务、多终端系统必须拆成多个 `moc`。

## 4. moc.yaml

每个 `moc` 应提供一份 `moc.yaml` 作为描述文件。它用于声明类型、入口、依赖的 block、协议和验证要求。

`moc.yaml` 是描述文件，不是运行入口。

建议最小字段：

- `id`
- `name`
- `type`
- `backend_mode`（仅 `backend_app` 需要）
- `entry`
- `language`
- `public_contract`
- `uses.blocks`
- `uses.internal_blocks`
- `depends_on_mocs`
- `protocols`
- `verification`
- `acceptance_criteria`

`verification` 是可选的验证层，不是运行层。建议最小结构：

- `verification.commands`
- `verification.entry_flow`（可选，仅在需要串行校验时提供）
- `verification.flows`（可选，仅在需要串行校验时提供）

当存在 `verification.flows` 时：

- `verification.entry_flow` 必须存在
- 当前 MVP 仅支持单入口、串行 `steps` 和显式 `binds`

示例：

```yaml
id: hello-world
name: Hello World Console
type: backend_app
backend_mode: console
language: rust
entry: src/main.rs
public_contract:
  input_schema: {}
  output_schema: {}
uses:
  blocks:
    - core.console.write_line
  internal_blocks:
    - hello_world.message
depends_on_mocs: []
protocols: []
verification:
  commands:
    - cargo test
acceptance_criteria:
  - prints hello world exactly once
```

如果需要额外的串行校验，可再增加：

- `verification.entry_flow`
- `verification.flows`

## 5. 标准目录结构

建议目录：

```text
mocs/<moc-id>/
  moc.yaml
  README.md
  src/
  src-tauri/         # optional, frontend_app only
  internal_blocks/   # optional
  tests/
  examples/
```

说明：

- `moc.yaml`：描述与验证边界。
- `src/`：Rust 或前端主代码入口。
- `src-tauri/`：Tauri 宿主代码。
- `internal_blocks/`：当前 `moc` 私有 block。

实际仓库中，`backend_app` 当前使用 `backend/src/main.rs` 作为主入口；未来可继续收敛目录标准，但 `moc.yaml` 中的 `entry` 必须指向真实代码入口。

## 6. internal_blocks

`moc` 可以拥有内部私有 `block`，用于封装只在当前 `moc` 中复用的稳定能力。

规则：

- 内部私有 `block` 仍应有清晰契约。
- 内部私有 `block` 不默认进入全局 `blocks/` registry。
- 当内部能力被多个 `moc` 复用时，应提升为公共 `block`。
- `moc validate` 应至少能够检查 `internal_blocks/<block-id>/block.yaml` 和实现入口是否存在。
- `moc validate` 应至少能够检查 `depends_on_mocs` 指向的 `moc.yaml` 是否存在，以及对应 `protocols` 是否兼容。

## 7. 多 MOC 协作

复杂系统不应通过一个巨型 `moc` 同时承载多个终端或多个服务。

应拆分为多个 `moc`，并通过明确协议协作。

协议要求：

- 输入输出契约显式定义
- 版本兼容策略明确
- 调用边界可验证
- 失败行为可观测

`moc` 与 `moc` 的交互应被视为“协议连接”，而不是隐式共享内部实现。

最小校验要求：

- 依赖目标 `moc` 必须存在
- `depends_on_mocs[].protocol` 必须在本地和目标 `moc` 中同时声明
- `channel`、`input_schema`、`output_schema` 必须一致

建议在 `moc.yaml` 中至少声明：

- `depends_on_mocs[].moc`
- `depends_on_mocs[].protocol`
- `protocols[].name`
- `protocols[].channel`
- `protocols[].input_schema`
- `protocols[].output_schema`

## 8. 设计约束

- `moc.yaml` 不得替代真实代码入口。
- `moc` 主入口可以自由实现，但不应绕过 `block` 契约。
- 若 `moc` 声明遵循 block 可观测基线，则 block 执行不得绕过统一观测边界（统一 runtime wrapper 或等价诊断 trait）。
- `moc` 应优先复用已有公共 `block`，而不是重复造轮子。
- 一个 `moc` 只承载一种交付类型。
- 多端、多服务场景应拆解为多个 `moc`，再通过协议连接。
- 在多 block 的 `moc run` 场景，诊断关联键 `trace_id` 应作为必填链路字段。

## 9. 验收基线

一个合格的 `moc` 至少应满足：

- 类型声明正确且唯一
- 主入口真实存在
- 所有外部依赖 `block` 可被发现并可验证
- 内部私有 `block` 的边界清晰
- 对外协议可说明、可测试、可追溯
- 文档、测试、运行方式齐全
