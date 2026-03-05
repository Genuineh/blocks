# TODO

## 执行原则

- 每轮迭代先做架构分析，再做实现。
- 默认采用 `red/green TDD`，先写失败测试，再做最小实现。
- 先纠正模型，再扩展功能；禁止在错误模型上继续堆能力。
- 每次完成实质推进时，同步更新本文件的进度与优先级。

## 当前进度

- `CLI Decoupling`：已按 `docs/prds/BLOCKS_CLI_DECOUPLING_PLAN.md` 完成 Phase 1、Phase 2 与 Phase 3；当前解耦计划中的三阶段目标均已落地。
- `P0`：已完成。Rust workspace、`blocks-contract`、`blocks-registry`、`blocks-runtime`、最小 CLI 已落地。
- `P1`：已完成“旧 app 模型”最小闭环，但该闭环已不再是正确目标模型。
- `P2`：已完成“旧 app 模型”下的技能和示例，但同样需要迁回 `moc` 模型。
- `R0`：已完成。解决了“描述与实现分离”的第一轮整改。
- `R1`：已完成。`moc` 命名、类型约束、协议字段、`internal_blocks` 目录样式、自由 `moc.main` 与可选 `verification.flows` 已回正。
- `R2`：已完成。`moc run`、`rust_lib` 示例、跨 `moc` 协议校验都已补齐，`hello-world-console` 也已收口为在 `main` 中直接调用 block 的零输入示例。
- `R3`：已完成。`moc run` 已可分发到真实 Rust backend 启动器，前端 `moc` 样例已补齐（包含交互式 counter 示例），跨 `moc` 运行时连接也已具备最小可执行链路。
- `R4`：已完成。前端 `moc` 已补最小本地预览运行路径，`moc run` 已同时覆盖 `backend_app` 与 `frontend_app`，`blocks-core` 也已退出默认执行路径。
- `R5`：已完成。`counter-panel-web` 已升级为真实 Tauri 宿主，`moc run` 会通过该宿主的 headless probe 路径验证前端运行入口；`moc dev` 已覆盖 `rust_lib` 与 `frontend_lib`，`verification.flows` 也已移出默认运行路径。
- `R6`：已完成。前端预览已改为复用共享前端 block 渲染模块，`moc verify` 已提供更直接的 bind/类型/引用错误提示，前端 `moc` 的 GUI 与终端说明也已分离清楚。
- `R7`：已完成。`counter-panel-web` 的 `src/preview` HTML 壳层已通过共享 `preview/shell.css` 收口，`moc verify` 的绑定错误已可定位到具体 `flow/step/bind`，前端 `moc` 的 `moc dev` 也已提供本地浏览器预览命令与 URL。
- `R8`：已完成。新增 `greeting-api-service` 与 `greeting-panel-web` 双 `moc` 全栈 proof slice，前端已通过真实 HTTP fetch 渲染后端返回的数据，且验证边界已明确区分自动化与手工环节。
- `R9`：已完成。block 可调试与可观测基础能力已收口：契约校验、runtime 诊断事件与工件、CLI diagnose、首批 active block 迁移、仓库级验证路径均已落地。
- `Block Spec Migration`：已完成当前仓库全部 `block.yaml` 向最新 `BLOCKS_SPEC` 标准契约字段集迁移，包含基础标识、能力边界、执行约束、失败契约、验证评估等补齐。

## 当前最高优先级的架构问题

- `Blocks CLI` 解耦已完成 Phase 3：catalog manifest 已成为唯一手工注册面，构建期生成胶水也已替代手写分发表；下一轮若继续推进，应聚焦更高层的工具链体验而不是再扩 CLI wiring。
- 当前 `R9` 已收口；下一轮可考虑把 diagnose 导出/聚合能力进一步产品化（例如更稳定的汇总报告和筛选维度）。
- `BLOCKS_SPEC` 标准契约字段虽已完成存量 block 迁移补齐，但 `blocks-contract` 对 `owner/scope/non_goals/...` 等 MUST 字段仍未全面机器强校验；建议补一轮 `warn -> error` 门禁实现，避免后续回归。
- 仓库级检查已补充 `./scripts/repo_check.sh`，会同时覆盖 root workspace tests 与 `counter-panel-web` 的独立 Tauri headless probe；后续应将这条路径固化为更稳定的 CI 基线。
- 当前前端 `moc` 的浏览器预览已可通过本地静态服务器命令直接访问，但 CLI 仍只是输出辅助命令，尚未内置长驻预览子命令。
- 当前 `moc verify` 已可定位到具体 `flow/step/bind`，但运行期 block 执行失败仍主要停留在 step 级别。
- 当前 `counter-panel-web` 已收口共享 HTML 壳层，下一轮可以把同类壳层模式推广到更多前端 `moc`。
- 新增架构整改计划：[`prds/ARCHITECTURE_DEBT_REDUCTION_PLAN_2026Q1.md`](prds/ARCHITECTURE_DEBT_REDUCTION_PLAN_2026Q1.md) / [`specs/ARCHITECTURE_REFACTOR_SPEC_2026Q1.md`](specs/ARCHITECTURE_REFACTOR_SPEC_2026Q1.md) / [`decisions/001-enforce-contract-runtime-boundary.md`](decisions/001-enforce-contract-runtime-boundary.md)；优先完成 contract 强校验、runtime 边界统一、CLI 分层。

## Blocks CLI 解耦计划（按 Phase 跟踪）

关联文档：

- 计划：[`prds/BLOCKS_CLI_DECOUPLING_PLAN.md`](prds/BLOCKS_CLI_DECOUPLING_PLAN.md)
- Phase 1 记录：[`logs/2026-03-04-blocks-cli-decoupling-phase-1.md`](logs/2026-03-04-blocks-cli-decoupling-phase-1.md)
- Phase 2 记录：[`logs/2026-03-04-blocks-cli-decoupling-phase-2.md`](logs/2026-03-04-blocks-cli-decoupling-phase-2.md)
- Phase 3 记录：[`logs/2026-03-04-blocks-cli-decoupling-phase-3.md`](logs/2026-03-04-blocks-cli-decoupling-phase-3.md)

### Phase 1（提取 wiring 边界，已完成）

- [x] 新增 `blocks-runner-catalog`，承接可执行 Rust block 的注册与分发。
- [x] 将 `CliBlockRunner` 与 `match block_id` 分发表从 `blocks-cli` 迁出。
- [x] 保持命令行为不变，仅完成所有权边界抽离。

Phase 1 验收：

- [x] `blocks-cli` 不再直接导入具体 block crate。
- [x] 现有命令行为与抽离前保持一致。

### Phase 2（稳定 runner 契约，已完成）

- [x] 在 catalog 层暴露稳定的公共构造入口（例如 `default_block_runner()`）。
- [x] 让 `blocks-cli` 只通过 `blocks-runtime::BlockRunner` trait 边界与 catalog 交互。
- [x] 将 unknown-block 与已注册 block 的重点回归测试收口到 catalog 层。

Phase 2 验收：

- [x] 运行期注册测试归属 `blocks-runner-catalog`。
- [x] CLI 测试不再覆盖 catalog 自身拥有的分发表细节。

### Phase 3（减少手工注册，已完成）

- [x] 保持 `crates/blocks-runner-catalog/Cargo.toml` 作为唯一手工注册面。
- [x] 在 catalog crate 本地引入构建期生成胶水，基于已存在的 `block.yaml` 元数据生成可审查、可预测的分发代码。
- [x] 保持运行时简单，仍不引入动态加载。

Phase 3 验收：

- [x] 新增一个可执行 Rust block 时，只需要改 `crates/blocks-runner-catalog/Cargo.toml`，不再修改 CLI 命令代码。
- [x] catalog 生成顺序按 block id 稳定排序，输出保持确定性。
- [x] 无效 `block.yaml` 元数据会直接导致构建失败，而不是被静默跳过。

## R1（moc 模型整改，已完成）

- [x] 新增并固化 `moc.yaml`，替代当前 `app.yaml` 作为上层描述文件。
- [x] 将仓库中的 `apps/` 迁移为 `mocs/`。
- [x] 将 `AppManifest` 相关命名统一迁移为 `MocManifest`。
- [x] 将 `blocks-composer` 的职责和命名迁移为 `blocks-moc` 或等价的 `moc` 描述层。
- [x] 在规范层固定 `moc` 类型：`rust_lib`、`frontend_lib`、`frontend_app`、`backend_app`。
- [x] 为 `backend_app` 增加 `console | service` 模式声明。
- [x] 固定“一个 moc 只允许一种类型”的约束，并在描述层做校验。
- [x] 为 `moc` 增加内部私有 `block` 的正式结构（如 `internal_blocks/`）。
- [x] 调整运行模型：`moc.main` 必须是自由代码入口，不能再被描述文件限制成固定步骤图。
- [x] 将当前示例从 `hello-pipeline` / `echo-pipeline` 重构为更符合模型的 `hello-world-console` 等 `moc` 示例。
- [x] 新增一个最小控制台输出能力，例如 `core.console.write_line`，用于证明 `moc.main` 直接调用 block 的真实路径。
- [x] 为多 moc 场景补最小协议模型：至少定义 `moc -> moc` 的输入输出契约边界。
- [x] 将技能文档从“compose app”迁移到“build moc”。
- [x] 同步修正 README、guide、whitepapers、specs、PRD 文档中的剩余 `app` 旧表述。

R1 验收：

- [x] 仓库上层交付单元统一使用 `moc` 命名，而不是 `app` 或 `project`。
- [x] 至少一个 `moc` 使用 `moc.yaml + 真实代码入口` 结构。
- [x] 至少一个 `backend_app(console)` 示例可以通过真实入口运行。
- [x] 至少一个示例明确使用公共 `block`，并可选使用内部私有 `block`。
- [x] 多 moc 协议边界已有最小规范，不再依赖隐式共享。

## R2（moc 体验收口，已完成）

- [x] 为 `mocs/<moc>/internal_blocks/` 增加本地发现或校验入口，避免它只停留在目录约定。
- [x] 为 `blocks-cli` 增加更统一的 `moc run` 入口，降低示例启动成本。
- [x] 增加一个非 `backend_app(console)` 的最小 `moc` 示例（优先 `rust_lib` 或 `frontend_lib`）。
- [x] 增加一个最小多 `moc` 协议验证示例，证明 `depends_on_mocs` 和 `protocols` 的实际用途。

R2 验收：

- [x] `internal_blocks` 不只是目录约定，至少可被本地检查。
- [x] 至少一个 `moc` 可通过统一 CLI 入口启动。
- [x] 至少覆盖两种不同 `moc` 类型。
- [x] 至少一个跨 `moc` 协议声明可被验证。

## R3（真实运行与前端样例，已完成）

- [x] 让 `blocks moc run` 能分发到更多真实 `moc` 入口，而不只执行 `verification` 过渡流。
- [x] 将当前公共 Rust `block` 提升为独立 crate，并让现有主要 Rust backend 示例直接依赖它们。
- [x] 增加一个最小 `frontend_lib` 或 `frontend_app` 示例，落地 Tauri + TS 边界。
- [x] 增加一个跨 `moc` 的真实运行时连接样例，而不只停留在协议校验。

R3 验收：

- [x] 至少一个 descriptor-only moc 可通过统一 CLI 入口触发真实启动。
- [x] 至少覆盖一个前端 `moc` 类型。
- [x] 至少一个跨 `moc` 连接不只是静态声明，而是可执行链路。

## R4（统一运行与前端落地，已完成）

- [x] 为前端 `moc` 增加最小可执行运行路径，而不只停留在源文件结构。
- [x] 继续让更多 `moc` 默认通过统一分发入口启动，而不是只保留文档级命令。
- [x] 收缩 `blocks-core` 的过渡职责，逐步让更多调用直接走 block crate。

R4 验收：

- [x] 至少一个前端 `moc` 具备最小本地运行路径。
- [x] 至少一个以上的 `moc` 类型可通过统一分发入口启动。
- [x] `blocks-core` 不再是新增 Rust block 的默认接入方式。

## R5（真实前端宿主与统一入口深化，已完成）

- [x] 为至少一个 `frontend_app` 增加可执行宿主启动器骨架，让统一入口不只返回静态预览路径。
- [x] 将前端宿主骨架升级为真实 Tauri 宿主运行入口。
- [x] 为 `rust_lib` / `frontend_lib` 明确更清晰的本地开发命令（至少是统一检查或预览入口）。
- [x] 继续收缩 `verification.flows` 在实际运行中的角色，避免它重新膨胀为主运行模型。

R5 验收：

- [x] 至少一个前端 `moc` 不仅可预览，还可通过真实 Tauri 宿主入口启动。
- [x] 至少一种非 `*_app` 类型 `moc` 有明确的本地开发入口。
- [x] `verification.flows` 继续保持辅助层，而不是默认运行主路径。

## R6（前端资源收口与校验反馈增强，已完成）

- [x] 减少前端 `src/` 与 `preview/` 的重复维护，避免同一界面有两套手写实现。
- [x] 强化 `moc verify` 的错误输出，让 bind/类型/缺失字段问题更易直接定位。
- [x] 为前端 `moc` 增加更清晰的开发/验证说明，区分“真实宿主启动”和“无界面 probe”。

R6 验收：

- [x] 至少一个前端 `moc` 的预览资源不再与源码形成明显重复实现。
- [x] `moc verify` 的常见失败路径有更直接的定位信息。
- [x] 前端 `moc` 的运行说明对 GUI 与终端场景边界清晰。

## R7（前端壳层收口与验证定位深化，已完成）

- [x] 继续减少前端 `src/*.html` 与 `preview/*.html` 的重复壳层代码。
- [x] 将 `moc verify` 的错误提示进一步定位到具体 `flow`、`step`、`bind` 条目。
- [x] 为前端 `moc` 增加更自动化的本地浏览器预览方式（不强制依赖 GUI 打开工具）。

R7 验收：

- [x] 至少一个前端 `moc` 的 HTML 壳层重复明显减少。
- [x] `moc verify` 的错误提示可直接指向更具体的 flow/step/bind 位置。
- [x] 前端 `moc` 的浏览器预览路径比“手工打开文件”更顺手。

## R9（block 可调试与可观测基础能力，已完成）

关联文档：

- 计划：[`prds/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_PLAN.md`](prds/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_PLAN.md)
- 规格：[`specs/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_SPEC.md`](specs/BLOCK_DEBUG_OBSERVABILITY_FOUNDATION_SPEC.md)

R9 范围（规范口径）：

- `in scope active blocks`：由 `blocks-registry` 发现且 `block.yaml` 声明 `status: active` 的 block。
- 首批 active block（已迁移）：`core.console.write_line`、`core.fs.read_text`、`core.fs.write_text`、`core.http.get`、`core.json.transform`、`core.llm.chat`、`demo.echo`。

### R9 Phase 1（规范真源对齐）

- [x] 在 `BLOCKS_SPEC` 正式纳入 `debug`、`observe`、`errors.taxonomy`（Contract v1.1），并声明迁移窗口（warn -> error）。
- [x] 在 `MOC_SPEC` 明确观测边界不可绕过规则，补充 `moc run` 场景 `trace_id` 必填要求。
- [x] 在 R9 PRD/Spec/TODO 对齐 scope 定义、术语与验收口径。

Phase 1 验收：

- [x] `BLOCKS_SPEC`、`MOC_SPEC`、R9 PRD/Spec/TODO 不再出现规范冲突。
- [x] R9 scope 在文档中可被唯一判定（不再混用“首批核心/任一/活跃”分母）。

### R9 Phase 2（CLI 与运行时能力）

- [x] 在运行时补齐统一诊断事件 envelope（含 `execution_id`，并在 `moc verify`/诊断链路提供 `trace_id`）。
- [x] 为失败路径补齐诊断工件（最小包含输入快照、错误报告、条件输出快照）与基础脱敏策略。
- [x] 在 CLI 增加 block/moc 级诊断查看与导出入口，并保持现有命令形状兼容。

Phase 2 验收：

- [x] `cargo run -p blocks-cli -- --help` 可见 diagnose 能力。
- [x] `cargo run -p blocks-cli -- block diagnose <blocks-root> <block-id> --json` 输出可机器解析诊断数据。
- [x] `cargo run -p blocks-cli -- moc diagnose <blocks-root> <moc.yaml> --json` 可输出同一 `trace_id` 下的多 block 关联结果。

### R9 Phase 3（active block 迁移）

- [x] 明确首批 active block 列表并固定快照（commit/date）。
- [x] A：在 `blocks-contract` / `blocks-registry` 落地 R9 合同约束解析与校验（`debug`、`observe`、`errors.taxonomy`、taxonomy id/唯一性、`status: active` 必填、`warn -> error` 可配置常量）。
- [x] 迁移首批 active block 到 R9 契约基线（字段、事件、工件、脱敏）。
- [x] 将对应校验纳入自动化检查路径。

Phase 3 验收：

- [x] 首批 active block 的 `block.yaml` 全部通过 R9 新字段校验。
- [x] 首批 active block 失败执行全部可产出包含 `execution_id` 的结构化诊断记录。
- [x] 首批 active block 的失败工件默认落在 `.blocks/diagnostics`，并满足基础脱敏规则。

### R9 Phase 4（跨 block/跨 moc 关联与验证收口）

- [x] 增加跨 block 关联验证用例，证明单个 `moc` 中失败链路可定位。
- [x] 增加 protocol-edge 诊断用例，覆盖至少一条 moc-to-moc 协议失败路径。
- [x] 在仓库级检查中增加 R9 关键验收项。

Phase 4 验收：

- [x] 至少一条多 block `moc` 执行链路可通过 `trace_id + execution_id` 自动定位失败点。
- [x] 至少一条跨 moc 协议失败链路可通过 protocol-edge 诊断字段定位调用边界与失败类别。
- [x] 自动化覆盖与手工验证边界已绑定到明确命令、产物路径与通过条件。

R9 验收：

- [x] Scope 内 active block 的 `block.yaml` 均声明并通过调试/观测字段校验。
- [x] Scope 内 active block 的失败执行均产出可机器解析诊断记录，且包含稳定 `execution_id`。
- [x] 多 block `moc` 链路可通过 `trace_id + execution_id` 关联诊断定位失败点。
- [x] 诊断工件具备基础脱敏且可复现至少一条 benchmark 失败样例。
- [x] 文档明确自动化覆盖范围与手工诊断边界，且对应命令在仓库可执行。

## R10（架构回正，进行中）

- [ ] 为 `blocks-contract` 补齐 `BLOCKS_SPEC` 关键 `MUST` 字段机器强校验（`owner/scope/non_goals/inputs/outputs` 等），并收口迁移窗后的 error 门禁。
- [ ] 为 runtime 补齐 taxonomy 对齐与 `artifact_policy` 策略执行，避免 `moc run`/`moc verify` 观测边界分裂。
- [ ] 将 `blocks-cli` 从单文件入口拆分为命令解析层、应用编排层、输出渲染层，保持现有命令行为兼容。

### R10 Phase 1（统一执行与观测边界，最高优先）

- [x] 产出 R10 Phase 1 最小可落地函数级改造清单并落文档：
  [`prds/R10_PHASE1_MINIMAL_RUNTIME_BOUNDARY_PLAN.md`](prds/R10_PHASE1_MINIMAL_RUNTIME_BOUNDARY_PLAN.md) /
  [`specs/R10_PHASE1_RUNTIME_BOUNDARY_SPEC.md`](specs/R10_PHASE1_RUNTIME_BOUNDARY_SPEC.md) /
  [`decisions/002-r10-phase1-runtime-observability-boundary.md`](decisions/002-r10-phase1-runtime-observability-boundary.md)。
- [x] 已补齐 R10 Phase 1 关键回归测试草案（runtime taxonomy 命中/fallback、moc diagnose 串链隔离、flow-based run/verify 诊断关联字段一致性），当前进入实现对齐阶段。
- [x] 让 `moc run` 与 `moc verify` 共享统一 runtime wrapper 执行边界，并在多 block 链路强制 `trace_id`。
- [x] 在诊断事件中加入 `moc_id`（或等价字段），修正 `moc diagnose` 默认 trace 选择串 moc 的问题。
- [x] 将 runtime 失败 `error_id` 与 `errors.taxonomy` 做映射校验，提供受控兜底分类策略。

Phase 1 验收：

- [x] 至少一条 `moc run` 多 block 链路可稳定产出统一 `trace_id + execution_id + moc_id` 关联诊断。
- [x] `moc diagnose` 默认最新链路不会误选其他 `moc` 的 trace。
- [x] runtime 失败事件 `error_id` 与 block taxonomy 一致或命中显式兜底码。

### R10 Phase 2（契约门禁与描述一致性）

- [x] 为 `BlockContract` 增加 `owner/scope/non_goals/inputs/outputs/preconditions/postconditions/dependencies/side_effects/timeouts/resource_limits/failure_modes/error_codes/recovery_strategy/verification/evaluation/acceptance_criteria` 的结构化解析与校验。
- [x] 将 `status: active` 迁移窗后门禁从 warn 收口到 error，并保留可配置迁移开关。
- [x] 在 `blocks-moc` 中补齐 `uses.blocks` 声明与 flow step 实际依赖一致性校验。

Phase 2 验收：

- [x] 缺失 `BLOCKS_SPEC` 关键 MUST 字段的 block 在校验链路中可被稳定拦截。
- [x] `moc validate/verify` 可阻断“使用未声明 block 依赖”的描述漂移。
- [x] active block 契约门禁策略与规范日期窗口保持一致。

### R10 Phase 3（CLI 分层与可维护性收口）

- [ ] 将 `blocks-cli/src/main.rs` 按 `commands/*`、`app/*`、`render/*` 分层拆分，并保留现有 CLI 命令形状。
- [ ] 将命令行为测试从单文件内联测试迁移到更清晰的模块/集成测试边界。
- [ ] 为诊断输出补充稳定 JSON 契约回归测试，降低后续演进回归风险。

Phase 3 验收：

- [ ] `main.rs` 仅保留轻量入口与路由，不再承载核心业务编排逻辑。
- [ ] 现有 `blocks/moc` 命令兼容性回归通过。
- [ ] 关键诊断命令具备稳定 JSON 输出契约测试。

## P0

- [x] 建立 Rust workspace 与最小基础 crate。
- [x] 建立 `block` 契约解析与本地发现。
- [x] 跑通本地 `blocks list/show/search`。

## P1（旧模型闭环，已完成但需迁移）

- [x] 完成单 `block` 执行闭环。
- [x] 完成旧 `app.yaml` 串行绑定原型（现已迁为 `moc.yaml` 命名）。
- [x] 完成最小核心 block 集合（当前已达到 6 个）。

说明：

- 这些能力仍有价值，但上层模型需要迁移到 `moc`。

## P2（旧模型示例，已完成但需迁移）

- [x] 完成创建 `block` 的技能文档。
- [x] 完成旧“compose app”技能文档（现已迁为 `build-moc.md`）。
- [x] 完成最小示例程序。

说明：

- 下一轮应改为“create block + build moc”。

## P3（延后）

- [ ] 在 `R1` 完成后，再推进最小 Tauri + TS 前端 `moc` 示例。
- [ ] 在 `R1` 完成后，再推进最小 `BCL`，并让它面向 `moc` 结构而不是旧 `app` 流程。

P3 验收：

- [ ] 前端能力以 `frontend_lib` 或 `frontend_app` 的 `moc` 形式落地。
- [ ] `BCL` 以辅助 `moc` 描述和校验为目标，而不是重新引入错误的上层模型。

---

### Change Log
- 2026-03-04: Marked Blocks CLI Decoupling Phase 3 complete after switching the catalog to build-time generated dispatch glue and updating block registration guidance.
- 2026-03-04: Marked the greeting proof slice complete after adding paired backend/frontend mocs, bounded checks, and explicit manual render guidance.
- 2026-03-05: Added R9 planning for mandatory block debuggability/observability foundations and linked new PRD/spec documents.
- 2026-03-05: Revised R9 into phase-gated acceptance with normative scope, CLI compatibility checks, and executable verification criteria.
- 2026-03-05: Completed R9-A contract/registry constraints implementation for debug/observe/errors.taxonomy parsing + validation, taxonomy ID rules/uniqueness, active-status required fields, and configurable warn-to-error migration gating.
- 2026-03-05: Completed R9 end-to-end implementation including runtime diagnostics, CLI diagnose coverage, active block migration, and repository-level verification.
- 2026-03-05: Added architecture review follow-up (R10) and linked PRD/Spec/ADR for contract/runtime/cli debt reduction.
- 2026-03-05: Expanded R10 into phase-based execution plan with explicit acceptance criteria for runtime boundary unification, contract gate hardening, and CLI layering.
- 2026-03-05: Added R10 Phase 1 minimal landing design pack with function-level checklist, risk register, and dedicated ADR for runtime boundary + moc diagnose + taxonomy mapping.
- 2026-03-05: Completed R10 Phase 1 implementation: flow-based moc run/verify boundary unification, moc-scoped diagnostics (`moc_id`), and taxonomy-aware runtime error mapping with controlled fallback.
- 2026-03-05: Completed R10 Phase 2 implementation: BLOCKS_SPEC MUST-field contract enforcement, active-gate warn/error strategy with date+env override, and moc uses.blocks vs flow-step dependency consistency checks.
