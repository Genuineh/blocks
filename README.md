# blocks

`blocks` 是一种面向 AI 构建的软件基础组件概念。它的目标不是推翻现有的编程语言、框架与库，而是在这些既有成果之上，补上一层更适合 AI 使用的“积木化”能力，让 AI 能更稳定地组合、验证并交付高质量结果。

在今天的软件生产中，AI 难以稳定产出高质量结果，不仅因为大语言模型本身具有概率性，还因为现有大量工具链并不是围绕 AI 的使用方式设计的。它们对人类工程师友好，但对 AI 来说，往往存在理解成本高、调用路径不清晰、行为边界模糊、验证代价大等问题。

`blocks` 的核心主张是：如果希望 AI 生产高质量产品，就必须先让 AI 使用的每一块最小组件都能够稳定工作、可被验证、可被评估。只有高质量的积木，才有可能搭建出高质量的软件系统。

## 核心定义

一个 `block` 是一个面向 AI 的最小可用生产单元。它不是宏大的框架抽象，而是围绕一项简单、明确、稳定的产出能力设计的组件。

一个合格的 `block` 应满足：

- AI 可以快速理解它做什么。
- AI 可以准确调用它，不依赖模糊推断。
- AI 可以快速验证它是否正确工作。
- 人与机器都可以评估它的产出质量。
- 它可以与其他 `block` 组合，形成更复杂的系统能力。

## 为什么需要 blocks

当前 AI 在软件生产中的不稳定，主要体现在三个层面：

- `LLM` 的输出存在概率性，同一任务可能出现不一致结果。
- 现有框架和库主要服务于人类开发者，并未为 AI 的调用与验证方式优化。
- AI 在不同语言、框架、需求约束下的使用策略存在不确定性，容易出现理解偏差和实现偏差。

这意味着，即便有核心 `tool` 能力，AI 依旧可能在“如何正确使用框架、如何选择抽象、如何验证结果”这些环节失去稳定性。

## blocks 的方法

`blocks` 提出的不是“让 AI 直接驾驭所有复杂系统”，而是先把复杂系统拆解为可控的、稳定的最小部件：

1. 将需求拆分为若干可独立承担职责的最小能力单元。
2. 为每个能力单元定义清晰的输入、输出、约束与验证方式。
3. 先确保这些单元本身稳定可靠，再让 AI 负责组装、协调与选择。
4. 当出现新的需求时，优先补足新的 `block`，而不是直接扩大模糊需求面。

换句话说，AI 应该在“稳定可工作的部件集合”中做组合决策，而不是在“边界不清、行为不明的复杂框架”中做高风险探索。

## 设计原则

- `AI-first`：组件设计首先考虑 AI 的理解、调用、验证路径。
- `Verifiable`：每个组件都必须自带明确的验证方式。
- `Evaluable`：组件结果必须可以被客观评估，而不是停留在主观“看起来可用”。
- `Composable`：组件能以低歧义方式组合，形成更高层能力。
- `Minimal`：组件抽象的是最简单、最稳定的产出，而不是过度泛化的大而全接口。
- `Replaceable`：组件可独立升级、替换，而不破坏整体系统。

## blocks 解决的核心问题

`blocks` 试图建立一种新的软件生产基础设施，使 AI 能在以下方面获得确定性：

- 更低的调用歧义
- 更高的实现稳定性
- 更快的结果验证速度
- 更清晰的质量边界
- 更可持续的能力沉淀方式

## 一个简单例子

假设完成一个工程需要 10 种部件共同协作：

- `blocks` 不要求 AI 从头重新发明这 10 种能力。
- `blocks` 要求这 10 个部件本身都稳定、可验证、可独立交付。
- AI 的职责是根据目标，决定如何组合、排序和协调这些部件。

如果出现第 11 种新需求，正确路径不是让 AI 在现有不稳定能力上硬拼，而是先制造第 11 个合格的 `block`，再把它纳入可组合系统。

## 这不是在做什么

- 不是替代现有编程语言。
- 不是否定现有框架与库的价值。
- 不是要求一切都重新发明。
- 不是试图消除 `LLM` 的概率性。

`blocks` 的目标，是在既有软件生态上增加一层更适合 AI 的工程约束与可验证抽象。

## 仓库文档

- [docs/TODO.md](./docs/TODO.md)：当前代办、优先级与近期推进顺序。
- [docs/prds/MVP_PLAN.md](./docs/prds/MVP_PLAN.md)：当前最小 MVP 的 `moc` 模型、分期与实施路径。
- [docs/prds/RUST_WORKSPACE_ARCHITECTURE.md](./docs/prds/RUST_WORKSPACE_ARCHITECTURE.md)：当前 Rust workspace 的 `moc` 模型架构草图。
- [docs/prds/BLOCKS_CLI_DECOUPLING_PLAN.md](./docs/prds/BLOCKS_CLI_DECOUPLING_PLAN.md)：`blocks-cli` 运行 wiring 解耦方案，规划如何把可执行 block 注册从 CLI 命令层拆出。
- [docs/prds/GREETING_PROOF_SLICE_PLAN.md](./docs/prds/GREETING_PROOF_SLICE_PLAN.md)：最小真实前后端 proof slice 方案，定义 `greeting-api-service` 与 `greeting-panel-web` 的交付边界。
- [docs/prds/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_PLAN.md](./docs/prds/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_PLAN.md)：block 可调试与可观测基础能力规划，定义统一诊断与可观测基线。
- [docs/prds/ARCHITECTURE_DEBT_REDUCTION_PLAN_2026Q1.md](./docs/prds/ARCHITECTURE_DEBT_REDUCTION_PLAN_2026Q1.md)：当前 1-2 周架构整改计划，聚焦 contract/runtime/cli 三条主线。
- [docs/prds/R10_PHASE1_MINIMAL_RUNTIME_BOUNDARY_PLAN.md](./docs/prds/R10_PHASE1_MINIMAL_RUNTIME_BOUNDARY_PLAN.md)：R10 Phase 1 最小落地计划，聚焦 run/verify runtime 边界统一、moc 级诊断归属与 taxonomy 映射。
- [docs/guide/README.md](./docs/guide/README.md)：使用说明和后续贡献流程入口。
- [docs/decisions/README.md](./docs/decisions/README.md)：架构决策记录入口。
- [docs/decisions/001-enforce-contract-runtime-boundary.md](./docs/decisions/001-enforce-contract-runtime-boundary.md)：contract 强校验、统一运行边界与 CLI 分层的决策记录。
- [docs/decisions/002-r10-phase1-runtime-observability-boundary.md](./docs/decisions/002-r10-phase1-runtime-observability-boundary.md)：R10 Phase 1 最小决策，固定 runtime 观测边界统一与受控 fallback 方案。
- [docs/archive/README.md](./docs/archive/README.md)：历史文档与归档说明入口。
- [docs/whitepapers/WHITEPAPER.md](./docs/whitepapers/WHITEPAPER.md)：`blocks` 的理念白皮书，解释为什么需要这种面向 AI 的基础组件。
- [docs/whitepapers/DEVELOPMENT_WHITEPAPER.md](./docs/whitepapers/DEVELOPMENT_WHITEPAPER.md)：面向 `blocks` 能力进行项目产出的开发白皮书，定义交付方法与工程流程。
- [docs/specs/BLOCKS_SPEC.md](./docs/specs/BLOCKS_SPEC.md)：`block` 规范，定义公共能力单元的结构、契约、验证与质量要求。
- [docs/specs/MOC_SPEC.md](./docs/specs/MOC_SPEC.md)：`moc` 规范，定义最终交付单元的类型、结构、描述文件和协议边界。
- [docs/specs/GREETING_PROOF_SLICE_SPEC.md](./docs/specs/GREETING_PROOF_SLICE_SPEC.md)：全栈 greeting proof slice 的技术规格，明确接口与验证边界。
- [docs/specs/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_SPEC.md](./docs/specs/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_SPEC.md)：block 调试与可观测基础能力技术规格，定义诊断事件、工件与 CLI 能力边界。
- [docs/specs/ARCHITECTURE_REFACTOR_SPEC_2026Q1.md](./docs/specs/ARCHITECTURE_REFACTOR_SPEC_2026Q1.md)：contract/runtime/cli 架构整改技术规格，定义边界与实施步骤。
- [docs/specs/R10_PHASE1_RUNTIME_BOUNDARY_SPEC.md](./docs/specs/R10_PHASE1_RUNTIME_BOUNDARY_SPEC.md)：R10 Phase 1 函数级改造规格，给出 run/verify 共享执行边界、moc diagnose 修正与 error_id 映射清单。
- [docs/whitepapers/BLOCKS_LANGUAGE_WHITEPAPER.md](./docs/whitepapers/BLOCKS_LANGUAGE_WHITEPAPER.md)：面向 AI 的 `blocks` 抽象语言白皮书，定义语言模型、基础语法、编译器与产物形态。
- [mocs/echo-pipeline/README.md](./mocs/echo-pipeline/README.md)：当前最小 moc 示例，后端已直接依赖 `demo.echo` 的 Rust crate。
- [mocs/hello-pipeline/README.md](./mocs/hello-pipeline/README.md)：当前最小 moc 示例，后端已直接依赖文件类 Rust block crate。
- [mocs/hello-world-console/README.md](./mocs/hello-world-console/README.md)：自由 `moc.main` 示例，组合 `hello-message-lib` 和 `core.console.write_line`。
- [mocs/hello-message-lib/README.md](./mocs/hello-message-lib/README.md)：最小 `rust_lib` moc 示例，同时提供跨 `moc` 协议样例。
- [mocs/hello-panel-lib/README.md](./mocs/hello-panel-lib/README.md)：最小 `frontend_lib` moc 示例，提供统一 `moc dev` 预览入口。
- [mocs/counter-panel-web/README.md](./mocs/counter-panel-web/README.md)：最小交互式 `frontend_app` moc 示例，提供计数器界面、预览页面和真实 Tauri 宿主。
- [mocs/hello-panel-web/README.md](./mocs/hello-panel-web/README.md)：最小 `frontend_app` moc 示例，展示 Tauri + TypeScript 边界并提供本地预览入口。
- [mocs/greeting-api-service/README.md](./mocs/greeting-api-service/README.md)：最小 `backend_app(service)` moc 示例，提供真实 HTTP API 合同。
- [mocs/greeting-panel-web/README.md](./mocs/greeting-panel-web/README.md)：最小真实取数的 `frontend_app` moc 示例，获取后端 greeting 并渲染状态。
- [skills/create-block.md](./skills/create-block.md)：创建新 block 的标准流程。
- [skills/build-moc.md](./skills/build-moc.md)：当前 moc 构建技能文档。
