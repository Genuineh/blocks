---
status: draft
owner: Developer
created: 2026-03-05
updated: 2026-03-05
version: 1.0
supersedes: N/A
related_prds:
  - docs/prds/ARCHITECTURE_DEBT_REDUCTION_PLAN_2026Q1.md
---

# Architecture Refactor Specification 2026Q1

## Overview
该规格定义三项架构整改的技术实施边界：契约强校验、统一观测执行边界、CLI 分层解耦。

## Goals
- 建立 `BLOCKS_SPEC` 到 `blocks-contract` 的可执行一致性。
- 统一 `moc run` / `moc verify` 的 runtime 诊断边界与 taxonomy 约束。
- 降低 CLI 复杂度，明确命令解析与业务执行边界。

## Non-Goals
- 不在本轮引入动态插件系统。
- 不在本轮重写现有 block 业务逻辑。

## Architecture

### Components
- `blocks-contract`: 扩展 `BlockContract` 字段模型并补齐 `MUST` 字段校验规则。
- `blocks-runtime`: 根据 contract `errors.taxonomy` 与 `observe.artifact_policy` 驱动失败事件与工件行为。
- `blocks-cli`: 拆分为 `commands/*`、`app/*`、`render/*` 模块。
- `blocks-moc`: 保持验证层职责，新增与 `moc run` 的统一 runtime boundary 适配点。

### Data Flow
1. CLI 接收命令后只做参数解析并调用应用层。
2. 应用层为 `moc run`/`verify` 统一构造 runtime context（含 `trace_id`）。
3. runtime 按 contract 进行 input/output 校验、taxonomy 映射和 artifact policy 执行。
4. 渲染层将执行结果与诊断结果输出为 human/json 两种格式。

## API Specification

### Contract Validation
- **Input**: `block.yaml` 原文
- **Output**: `ContractValidationReport`（warnings/errors）
- **Errors**: 关键 `MUST` 字段缺失时返回 `InvalidDefinition`

### Runtime Failure Mapping
- **Input**: `BlockContract` + 执行错误类型
- **Output**: `error_id`（必须命中 taxonomy 或保留系统分类）
- **Errors**: taxonomy 不匹配时返回结构化 runtime 错误

### CLI Application Boundary
- **Input**: 解析后的 command DTO
- **Output**: domain result DTO
- **Errors**: 参数错误/执行错误均保持稳定错误枚举

## Data Models

### Contract Must Fields (minimum)
| Field | Type | Requirement |
|-------|------|-------------|
| owner | string | required |
| scope | array[string] | required |
| non_goals | array[string] | required |
| inputs | array[object] | required |
| outputs | array[object] | required |

### Runtime Observability Policy
| Field | Type | Requirement |
|-------|------|-------------|
| observe.artifact_policy.mode | enum | required for active |
| errors.taxonomy | array[id] | required for active |
| trace_id | string | required in multi-block moc run |

## Technical Decisions
| Decision | Choice | Rationale |
|---------|--------|-----------|
| MUST 字段执行策略 | 迁移窗后直接 error | 防止“文档合规、运行不合规”漂移 |
| runtime 失败映射 | taxonomy-first，保留系统错误兜底 | 兼顾严格性与可迁移性 |
| CLI 结构 | command/app/render 三层 | 降低耦合并提升可测性 |

## Security Considerations
- runtime 工件必须继续执行基础脱敏策略。
- taxonomy/diagnostics 输出不得泄露敏感原文。

## Performance Requirements
- 契约校验新增字段后，registry 扫描耗时增长应控制在 10% 以内。
- CLI 分层不引入额外进程边界。

## Testing Strategy
- `blocks-contract`: 新增 MUST 字段缺失与格式错误测试。
- `blocks-runtime`: 新增 taxonomy 映射、artifact policy 分支测试。
- `blocks-cli`: 新增命令分层单测与 `moc run` 诊断一致性集成测试。
