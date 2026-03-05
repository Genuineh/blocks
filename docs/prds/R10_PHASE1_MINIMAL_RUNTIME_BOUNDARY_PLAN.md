---
status: draft
owner: Developer
created: 2026-03-05
updated: 2026-03-05
related_issue: N/A
version: 1.0
---

# R10 Phase 1 Minimal Runtime Boundary Plan

## Summary
为 `R10 Phase 1` 提供最小可落地改造方案，目标是先收口 `moc run/verify` 的运行与观测边界，再补齐 `moc diagnose` 链路归属准确性，以及 runtime 失败 `error_id` 的 taxonomy 对齐与受控兜底。

## Problem
- `moc verify` 使用 runtime wrapper，但 `moc run` 主要走真实 launcher/preview，导致诊断边界割裂。
- 诊断事件当前无 `moc_id`，`moc diagnose` 默认 latest trace 依赖 `uses.blocks` 推断，多个 moc 复用同一 block 时可能串链。
- runtime 失败 `error_id` 使用硬编码（如 `invalid_output`），未与 block 声明的 `errors.taxonomy` 对齐。

## Users
- 维护 `blocks-cli` 运行链路与诊断命令的开发者。
- 维护 `blocks-runtime` 可观测契约的开发者。
- 依赖 `moc diagnose --json` 做自动分析的 AI/脚本调用方。

## Requirements
### Must Have
- 对“声明了 `verification.entry_flow` 的 moc”，`moc run` 与 `moc verify` 走同一 runtime wrapper 执行路径，并产出统一 `trace_id`。
- 诊断事件（及失败工件）带 `moc_id`，`moc diagnose` 默认 latest trace 以 `moc_id` 为主过滤条件。
- runtime 失败 `error_id` 实现 taxonomy-first 映射，并在未命中时使用显式受控 fallback 码。

### Should Have
- `moc diagnose` 对历史无 `moc_id` 事件保持兼容兜底（避免一次性破坏旧诊断数据）。
- 对 run/verify 共用执行路径增加最小回归测试，防止行为漂移。

### Nice to Have
- 在 diagnose 输出中增加 `mapping_source` 或等价字段，便于区分 taxonomy 命中与 fallback 命中。

## User Stories
- As a moc maintainer, I want `moc run` and `moc verify` to produce comparable diagnostics so that I can trace failures consistently.
- As a diagnostician, I want default `moc diagnose` to select the latest trace for the current moc only so that cross-moc noise is eliminated.
- As a block author, I want runtime error ids to align with declared taxonomy or explicit fallback codes so that automation stays deterministic.

## Success Metrics
- 至少 1 条 `moc run` 多 block 链路产出 `trace_id + execution_id + moc_id` 关联诊断。
- `moc diagnose` 在“两个 moc 复用同一 block”的测试夹具下，默认 latest trace 选取准确率为 100%。
- runtime 失败事件 `error_id` 全部满足“命中 taxonomy 或命中保留 fallback 码”。

## Timeline
- Step 1: runtime 结构扩展（`ExecutionContext/DiagnosticEvent/DiagnosticArtifact` 补 `moc_id`）。
- Step 2: CLI run/verify 共用执行边界抽取与接线。
- Step 3: diagnose 默认选择逻辑切换到 `moc_id` 优先并保留历史兼容。
- Step 4: taxonomy 映射与 fallback 规则落地、补齐回归测试。

## Open Questions
- fallback 码采用单一通用值，还是按失败类型分 3 个稳定保留码。
- 对已存在的“有 verification flow 且有真实 launcher”的 moc，`moc run` 是否立即切换为 runtime-wrapper 优先。

## Acceptance Criteria
- [ ] `moc run`/`moc verify` 对 flow-based moc 使用统一 runtime wrapper，且多 block 链路总是有 `trace_id`。
- [ ] 事件与工件包含 `moc_id`，`moc diagnose` 默认 latest trace 不再跨 moc 串链。
- [ ] runtime `error_id` 映射规则可重复、可测试，输出只会是 taxonomy 命中值或显式 fallback 码。
