---
adr_number: 002
date: 2026-03-05
status: proposed
author: Developer
reviewed_by: []
---

# 002: R10 Phase 1 Minimal Runtime Observability Boundary

## Status
Proposed

## Context
`R10 Phase 1` 需要在不引入大规模重构的前提下解决三个直接影响诊断可信度的问题：
- `moc run` 与 `moc verify` 执行边界分裂。
- 诊断事件缺少 `moc_id`，导致默认 latest trace 可能串链。
- runtime 失败 `error_id` 未与 block taxonomy 对齐。

## Decision
采用以下最小策略：
1. 对声明 `verification.entry_flow` 的 moc，`run/verify` 统一复用 runtime wrapper 执行路径，并显式传递 `trace_id`。
2. 诊断事件与失败工件新增 `moc_id` 字段；`moc diagnose` 默认 latest trace 选择改为 `moc_id` 优先。
3. runtime failure `error_id` 采用 taxonomy-first 映射；未命中时只允许命中保留 fallback 码集合。

## Consequences
### Positive
- run/verify 在 flow-based moc 场景具备可比较诊断输出。
- `moc diagnose` 默认行为不再依赖“共享 block 的弱推断”。
- `error_id` 分类规则可预测，可用于自动化分析。

### Negative
- 对有 verification flow 的 `moc run` 语义会更接近 verify，可能与“始终启动真实 launcher”的历史预期不同。
- 需要为历史无 `moc_id` 的事件保留兼容逻辑，短期增加判断分支。

## Alternatives Considered

### Alternative 1: 只修 `moc diagnose`，不改 run/verify 边界
**Pros**: 改动最小。  
**Cons**: 根因未解，run 仍可绕过统一 runtime 观测边界。  
**Why Rejected**: 不满足 R10 Phase 1 的边界统一目标。

### Alternative 2: 要求所有 moc launcher 立即接入 runtime SDK
**Pros**: 观测一致性最强。  
**Cons**: 迁移成本高，超出 Phase 1 最小落地范围。  
**Why Rejected**: 交付风险过大，不符合“最小可落地”约束。

## Notes
- Related PRD: `docs/prds/R10_PHASE1_MINIMAL_RUNTIME_BOUNDARY_PLAN.md`
- Related Spec: `docs/specs/R10_PHASE1_RUNTIME_BOUNDARY_SPEC.md`
- Parent ADR: `docs/decisions/001-enforce-contract-runtime-boundary.md`
