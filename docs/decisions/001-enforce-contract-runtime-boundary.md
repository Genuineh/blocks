---
adr_number: 001
date: 2026-03-05
status: proposed
author: Developer
reviewed_by: []
---

# 001: Enforce Contract And Runtime Boundary Consistency

## Status
Proposed

## Context
当前仓库在三处出现同类问题：
- 规范字段与 `blocks-contract` 解析模型不一致。
- `moc run` 真实运行链路可绕过统一 runtime wrapper。
- CLI 在单文件中承载过多领域职责，导致变更风险扩大。

这些问题会使“规范声明”和“运行事实”持续分裂。

## Decision
执行三项同步治理：
1. `blocks-contract` 对 `BLOCKS_SPEC` 关键 `MUST` 字段进行机器强校验，并在迁移窗结束后统一升级为 `error`。
2. `moc run` 与 `moc verify` 统一接入 runtime 可观测边界，保证 `trace_id/error_id` 与 taxonomy 一致。
3. `blocks-cli` 拆分为命令解析、应用编排、渲染输出三层模块。

## Consequences
### Positive
- 规范与实现具备一致的强约束，不再依赖人工记忆。
- 诊断链路在 run/verify 两条路径可比较、可追溯。
- CLI 可维护性和可测试性显著提升。

### Negative
- 短期会增加迁移工作量和回归测试成本。
- 部分历史示例可能在严格门禁下需要补字段或调整运行入口。

## Alternatives Considered

### Alternative 1: 继续维持 warn 策略
**Pros**: 迁移成本低。  
**Cons**: 规范漂移持续累积。  
**Why Rejected**: 无法在仓库级建立可靠质量门禁。

### Alternative 2: 只拆 CLI，不改运行边界
**Pros**: 实施快。  
**Cons**: 观测边界分裂仍在，核心风险未解。  
**Why Rejected**: 无法满足 R9 和 MOC 规范一致性目标。

## Notes
- Related PRD: `docs/prds/ARCHITECTURE_DEBT_REDUCTION_PLAN_2026Q1.md`
- Related Spec: `docs/specs/ARCHITECTURE_REFACTOR_SPEC_2026Q1.md`
