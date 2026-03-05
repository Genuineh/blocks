---
status: draft
owner: Developer
created: 2026-03-05
updated: 2026-03-05
version: 1.0
supersedes: N/A
related_prds:
  - docs/prds/R10_PHASE1_MINIMAL_RUNTIME_BOUNDARY_PLAN.md
  - docs/prds/ARCHITECTURE_DEBT_REDUCTION_PLAN_2026Q1.md
---

# R10 Phase 1 Runtime Boundary Specification

## Overview
本规格定义 `R10 Phase 1` 的最小改造面：统一 `moc run/verify` 的 runtime 执行边界、为诊断补齐 `moc_id` 并修正默认 latest trace 选取、实现 runtime `error_id` 的 taxonomy 映射与受控 fallback。

## Goals
- 让 flow-based moc 的 `run/verify` 在同一 runtime wrapper 中执行 block。
- 为诊断链路增加稳定 `moc_id` 归属键。
- 让失败 `error_id` 具备可预测映射规则。

## Non-Goals
- 不重写真实 launcher（`cargo run` / frontend host）能力。
- 不在 Phase 1 引入新的命令形状。
- 不在 Phase 1 扩展 `artifact_policy` 的全量执行矩阵。

## Architecture

### Components
- `blocks-runtime`: 诊断 envelope 增补 `moc_id`；失败 `error_id` 映射逻辑收口。
- `blocks-cli`: 提取 run/verify 共享执行函数；`moc diagnose` 选择逻辑修正。
- `blocks-moc`: 不新增能力，仅复用 `ExecutionPlan`。

### Data Flow
1. `moc run` / `moc verify`（仅 flow-based moc）进入统一执行函数。
2. CLI 构造 `ExecutionContext { trace_id, moc_id }`，逐 step 调用 runtime。
3. runtime 产出带 `trace_id + execution_id + moc_id` 的事件/工件。
4. `moc diagnose` 默认 latest trace 以 `moc_id` 过滤后选取；旧数据走兼容兜底。

## API Specification

### Runtime Context Extension
- **Input**: `ExecutionContext`
- **Output**: 事件/工件带 `moc_id`
- **Errors**: 无新增对外错误类型

### Diagnose Trace Selection
- **Input**: `events[] + moc_id`
- **Output**: latest trace_id（同 moc）
- **Errors**: 无匹配事件时，保持现有提示

### Runtime Error Mapping
- **Input**: `BlockContract + FailureKind`
- **Output**: `error_id`
- **Errors**: 无 taxonomy 命中时返回保留 fallback 码，不抛新错误

## Function-Level Change List

| File | Function | Change | Type |
|------|----------|--------|------|
| `crates/blocks-runtime/src/lib.rs` | `ExecutionContext` | 新增 `moc_id: Option<String>` | modify |
| `crates/blocks-runtime/src/lib.rs` | `DiagnosticEvent` | 新增 `moc_id: Option<String>`（`skip_serializing_if`） | modify |
| `crates/blocks-runtime/src/lib.rs` | `DiagnosticArtifact` | 新增 `moc_id: Option<String>`（与事件对齐） | modify |
| `crates/blocks-runtime/src/lib.rs` | `Runtime::execute_with_context` | start/success/failure 事件写入 `moc_id`；失败路径改为先做 error_id 映射 | modify |
| `crates/blocks-runtime/src/lib.rs` | `Runtime::handle_failure` | 入参新增 `contract` 或 `taxonomy` 视图；写入映射后的 `error_id` | modify |
| `crates/blocks-runtime/src/lib.rs` | `resolve_error_id_for_failure`（新增） | taxonomy-first + fallback 映射主函数 | add |
| `crates/blocks-runtime/src/lib.rs` | `fallback_error_id`（新增） | 返回稳定保留码：`runtime_fallback_invalid_input` / `runtime_fallback_internal_error` / `runtime_fallback_invalid_output` | add |
| `crates/blocks-cli/src/main.rs` | `run_moc_command` | 对 `manifest.has_validation_flow()` 分支改走共享 runtime 执行函数；其余路径保持原行为 | modify |
| `crates/blocks-cli/src/main.rs` | `verify_moc_command` | 改为调用共享 runtime 执行函数（避免逻辑分叉） | modify |
| `crates/blocks-cli/src/main.rs` | `execute_validation_plan` | 入参从 `trace_id: &str` 升级为 `ExecutionContext`（含 `moc_id`） | modify |
| `crates/blocks-cli/src/main.rs` | `moc_diagnose_command` | trace 过滤条件加入 `event.moc_id == manifest.id` | modify |
| `crates/blocks-cli/src/main.rs` | `select_latest_trace_id_for_moc` | 选择策略改为 `moc_id` 优先；历史事件（无 `moc_id`）回退到 `uses.blocks` 推断 | modify |
| `crates/blocks-cli/src/main.rs` | `event_belongs_to_moc`（新增） | 统一封装“当前事件是否属于 moc”判断 | add |

## Data Models

### ExecutionContext (Phase 1)
| Field | Type | Required |
|-------|------|----------|
| trace_id | Option<string> | yes（flow-based multi-step 时必须有值） |
| moc_id | Option<string> | yes（moc run/verify 场景必须有值） |

### Controlled Fallback Error IDs
| Failure Kind | Preferred Taxonomy ID | Controlled Fallback ID |
|--------------|------------------------|------------------------|
| input validation | `invalid_input` | `runtime_fallback_invalid_input` |
| block execution | `internal_error` | `runtime_fallback_internal_error` |
| output validation | `invalid_output` | `runtime_fallback_invalid_output` |

## Technical Decisions
| Decision | Choice | Rationale |
|---------|--------|-----------|
| run/verify 边界统一范围 | 仅覆盖 flow-based moc | 复用已有 `ExecutionPlan`，最小改动可落地 |
| latest trace 归属键 | `moc_id` 优先，旧数据兼容回退 | 解决串 moc 问题且不破坏历史诊断可读性 |
| taxonomy 映射策略 | 先命中 taxonomy，再命中保留 fallback | 保证严格性与迁移可用性并存 |

## Security Considerations
- 新增字段不改变现有脱敏策略。
- fallback `error_id` 不包含敏感业务上下文。

## Performance Requirements
- `moc diagnose` 额外 `moc_id` 过滤应保持 O(n) 单次扫描，不引入二次 IO。
- run/verify 共用逻辑不新增额外进程边界。

## Testing Strategy
- `crates/blocks-runtime/tests/runtime_execute.rs`
  - 新增：事件与工件都写入 `moc_id`。
  - 新增：taxonomy 命中时使用 taxonomy `error_id`。
  - 新增：未命中 taxonomy 时使用受控 fallback `error_id`。
- `crates/blocks-cli/src/main.rs`（现有内联 tests）
  - 新增：两个 moc 共享同一 block 时，`moc diagnose` 默认 latest trace 不串链。
  - 新增：`moc run`（flow-based）输出包含 trace_id，且诊断链路含 `moc_id`。
  - 回归：现有 `moc diagnose --trace-id --json` 结构保持兼容。

## Risks & Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| `moc run` 对 flow-based moc 行为改变 | 中 | 仅改 `has_validation_flow` 分支；保留无 flow 的 launcher 行为 |
| 历史无 `moc_id` 事件导致 diagnose 结果变化 | 中 | `moc_id` 过滤失败时回退旧的 block-based 策略 |
| fallback 码被误解为 taxonomy 声明值 | 中 | 文档明确“保留系统码”；JSON 输出增加 `moc_id` + `error_id` 解释注释（后续可加） |
| 单文件 `main.rs` 继续膨胀 | 低 | Phase 1 仅做最小改造，Phase 3 再做 CLI 分层 |

## Implementation Steps
1. 修改 runtime 结构体与失败映射函数，先补 unit tests。
2. 抽取 CLI run/verify 共享执行函数，接入 `ExecutionContext { trace_id, moc_id }`。
3. 调整 `moc diagnose` 默认 trace 选择逻辑并补串 moc 回归用例。
4. 回归 `block diagnose`/`moc diagnose --json` 输出契约。
