# TODO

## 执行原则

- 每一轮迭代开始前，先做宏观架构分析：确认受影响模块、契约边界、依赖关系和失败路径，再进入实现。
- 默认采用 `red/green TDD`：先补失败测试，再做最小实现通过测试，最后在不改变行为的前提下重构。
- 变更必须以安全迭代方式推进：优先小步提交，复用现有测试做护栏，避免跨层级的大范围无验证修改。
- 验收从严：不以“能跑一次”为完成标准，必须满足结构合理、测试覆盖、文档同步和可维护性要求。

## 当前进度

- `P0`：已完成。Rust workspace、架构草图、`blocks-contract`、`blocks-registry`、最小 CLI 和本地扫描已落地并通过测试。
- `P1`：已完成。已完成 `blocks-runtime` 最小执行闭环、`blocks run`、`app.yaml` 串行组合执行、`blocks compose run`，并落地 5 个最小核心 block。
- `P2`：进行中。已完成技能文档和 `hello-pipeline` 示例，待在后续真实新增 block 中持续验证复用性。
- `P3`：未开始。

## 当前最高优先级的架构问题

- 固化第一阶段最小系统边界：`contract / registry / runtime / cli` 四层职责不能混叠。
- 确认组合执行仍保持“薄运行时”原则，避免在第一阶段把 `composer` 做成隐性工作流引擎。
- 明确 block 契约、执行计划、运行日志三种核心模型的边界，避免后续重复抽象。

## P0

- [x] 输出 Rust workspace 的宏观架构草图：crate 边界、依赖方向、公共类型归属、错误流转路径。
- [x] 为 `blocks-contract` 先写失败测试，覆盖最小契约解析和输入校验，再实现基础模型。
- [x] 建立 Rust workspace，先落 `blocks-contract`、`blocks-registry`、`blocks-runtime`、`blocks-cli` 四个基础 crate。
- [x] 定义 `block.yaml` 的最小可执行契约模型，明确输入输出校验、前后置条件和标准错误结构。
- [x] 为 `blocks-registry` 先写目录扫描与索引失败测试，再实现本地发现。
- [x] 跑通本地 `blocks/` 目录扫描与 `blocks list/show/search`，先解决可发现性问题。

P0 验收：

- 架构草图能说明每个 crate 的单一职责、依赖方向和不能跨越的边界。
- `blocks-contract` 和 `blocks-registry` 的关键路径先有失败测试，再有通过实现。
- 当前设计没有引入可避免的重复模型或跨层耦合。

## P1

- [x] 为 `blocks-runtime` 先写执行前校验失败、执行成功、执行后校验失败三类测试，再实现运行闭环。
- [x] 实现 `blocks run <block-id>` 的最小执行闭环，包含输入校验、执行、输出校验和结构化日志。
- [x] 先为 `app.yaml` 组合执行写绑定缺失、类型不兼容、串行成功的测试，再实现轻量编排。
- [x] 实现轻量 `app.yaml` 组合执行能力，先支持串行 `steps + binds`。
- [x] 补齐最小核心 block 集合（当前已完成 5 个）：`core.fs.read_text`、`core.fs.write_text`、`core.json.transform`、`core.http.get`、`core.llm.chat`。

P1 验收：

- 运行时仍是薄胶水层，没有把 block 业务逻辑反向堆进 runtime。
- 所有新增能力都有对应失败测试和成功路径测试。
- 错误输出是结构化、可定位、可复用的，而不是临时字符串。

## P2

- [x] 先审视 AI 使用路径，确保 `skills` 描述的是稳定流程，而不是一次性操作说明。
- [x] 编写 `skills/create-block.md`，约束 AI 如何创建和验证新 block。
- [x] 编写 `skills/compose-app.md`，约束 AI 如何发现 block 并组装独立程序。
- [x] 提供 `hello-pipeline` 示例应用，验证 blocks 能组装出一个最小独立程序。

P2 验收：

- AI 能按技能文件稳定重复同一流程，关键步骤无隐性前提。
- 示例应用除了可运行，还能证明契约校验、组合执行和失败反馈都真实可用。

## P3

- [ ] 增加最小 Tauri + TS 前端组装器，用于组合前端能力 block，并提供基础预览、调试和运行入口。
- [ ] 在第一阶段闭环稳定后，再推进最小 `BCL` parser、语义检查和编译到执行计划。

P3 验收：

- Tauri + TS 明确用于组装前端能力 block，但不复制底层契约、发现与运行时逻辑，不破坏现有 CLI 入口。
- `BCL` 进入实现前，先完成最小语法与语义边界的架构复审，确认不引入超出 MVP 的复杂度。
