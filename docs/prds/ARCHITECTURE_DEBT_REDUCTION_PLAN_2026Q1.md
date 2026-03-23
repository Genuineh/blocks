---
status: active
owner: Developer
created: 2026-03-05
updated: 2026-03-13
related_issue: N/A
version: 1.0
---

# Architecture Debt Reduction Plan 2026Q1

## Summary
在 `blocks-contract/registry/runtime/moc/cli` 的当前实现中，存在三类会持续放大维护成本的架构问题：规范与模型漂移、观测边界绕过、CLI 单体化。该计划用于在 1-2 周内完成第一轮结构回正。

当前状态：首轮整改已完成，后续仅保留为历史与验收参考。

## Problem
- `BLOCKS_SPEC` 的多个 `MUST` 字段尚未由 `blocks-contract` 机器强校验，导致规范合规状态不可被稳定判定。
- `moc run` 的真实执行路径可绕过统一 runtime wrapper，使 `trace_id/error_id/taxonomy/artifact_policy` 边界在运行态出现分裂。
- `blocks-cli` 入口文件承载过多职责，变更成本和回归风险持续上升。

## Users
- 维护 `block` 契约与运行边界的开发者
- 维护 `moc` 编排与交付链路的开发者
- 依赖 CLI 自动化执行的 AI agent

## Requirements
### Must Have
- 补齐 `blocks-contract` 对 `BLOCKS_SPEC` 关键 `MUST` 字段的结构化解析与校验，并落地 `warn -> error` 门禁。
- 为 `moc run` 建立统一执行边界：不允许无 `trace_id` 和无 taxonomy 对齐的链路被视作合规执行。
- 将 `blocks-cli` 拆为命令层、应用层、渲染层，去除单文件超级入口。

### Should Have
- 为 runtime 增加基于契约的 `artifact_policy` 执行策略。
- 新增针对 `moc run` 真实链路的诊断一致性集成测试。

### Nice to Have
- 在 CLI 增加诊断导出聚合能力（按 block/trace 聚合）。

## User Stories
- As a block author, I want schema-level MUST fields to fail fast so that spec drift cannot silently ship.
- As a moc author, I want `moc run` and `moc verify` to share one observability boundary so that diagnostics are comparable.
- As a CLI maintainer, I want command code split by responsibility so that feature changes do not cause broad regressions.

## Success Metrics
- active block 的规范关键字段校验覆盖率达到 100%，并可通过自动化测试验证。
- `moc run` 多 block 链路诊断中 `trace_id` 覆盖率达到 100%。
- `blocks-cli` 主入口文件体积降到 < 400 行，命令逻辑迁移到子模块。

## Timeline
- Week 1: 契约校验补齐 + runtime taxonomy/policy 对齐。
- Week 2: CLI 分层拆分 + 端到端回归与文档收口。

## Open Questions
- taxonomy 缺省策略是强制 block 显式声明，还是引入保留系统错误码集合？
- `moc run` 对历史示例是否提供兼容开关（软迁移）？

## Acceptance Criteria
- [x] `blocks-contract` 对关键 `MUST` 字段具备机器强校验与失败门禁。
- [x] `moc run` 与 `moc verify` 均通过统一 runtime wrapper 产出一致诊断包络。
- [x] `blocks-cli` 完成模块化拆分并保持现有命令行为兼容。
