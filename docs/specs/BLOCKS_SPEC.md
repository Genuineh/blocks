# blocks 规范

## 1. 目的

本规范用于定义什么是一个合格的 `block`，以及一个 `block` 在 `blocks` 体系中必须满足的结构、契约、验证、质量和生命周期要求。

本规范的目标是确保：

- 任意 `block` 都可以被 AI 快速理解和稳定调用。
- 任意 `block` 的结果都可以被验证和评估。
- 任意 `block` 都可以被纳入统一的编排和交付体系。
- `block` 的增长不会破坏整体系统的一致性。

## 2. 术语

### 2.1 block

一个 `block` 是一个面向 AI 的最小可用能力单元，用于完成一项明确、单一、可验证的产出。

一个完整 `block` 由两部分组成：

- 描述层：`block.yaml`，只负责声明契约、实现类型、适用端和验证要求。
- 实现层：具体代码文件，当前应使用 `Rust` 或 `Tauri + TypeScript` 承载。

### 2.2 契约

契约是对一个 `block` 的输入、输出、约束、成功标准、失败边界和副作用的正式定义。

### 2.3 验证

验证是判断一个 `block` 是否按契约正确工作的过程。

### 2.4 评估

评估是对一个 `block` 结果质量进行量化或规则化判断的过程。验证关注“是否正确工作”，评估关注“工作质量是否达标”。

### 2.5 moc

`moc`（`my own creation`）是最终交付单元，`block` 对 `moc` 的角色应等同于“高约束的库能力”。

规则：

- `block` 负责稳定能力，不负责承担最终产品身份。
- `moc` 可以自由调用 `block`，也可以定义内部私有 `block`。
- 多 `moc` 系统应通过显式协议交互，而不是共享隐式内部状态。

## 3. 规范级别

本规范采用以下约束术语：

- `MUST`：必须满足，否则不能被视为合格 `block`。
- `SHOULD`：强烈建议满足，只有明确理由时才可偏离。
- `MAY`：可选项，可根据具体场景决定是否采用。

## 4. 一个合格 block 的必要条件

一个合格的 `block` 必须满足以下条件。

### 4.1 单一职责

`block` `MUST` 只承担一个明确能力目标。若一个能力无法用一句话准确描述其职责，则该 `block` 很可能过大。

### 4.2 明确边界

`block` `MUST` 明确定义：

- 输入是什么
- 输出是什么
- 什么情况下成功
- 什么情况下失败
- 是否存在副作用

### 4.3 可被 AI 理解

`block` `MUST` 提供足够清晰的说明，使 AI 无需依赖隐性知识即可理解其用途和调用方式。

### 4.4 可执行验证

`block` `MUST` 提供至少一种可执行验证方式，以证明其功能符合契约。

### 4.5 可质量评估

`block` `MUST` 提供结果质量判断标准，避免只有“能运行”而没有“是否达标”的判断。

### 4.6 可组合

`block` `MUST` 具备可被其他 `block` 或 `moc` 稳定调用的输入输出结构，不能依赖难以控制的隐性状态。

### 4.7 实现与描述分离

`block.yaml` `MUST` 只是描述文件，不能被视为功能实现本身。实际能力 `MUST` 由代码承载：

- `Rust` 实现可用于后端、前端共享逻辑或通用库能力。
- `Tauri + TypeScript` 实现仅用于前端能力。
- `block` 的最终使用位置应是 `moc` 主代码或其他稳定边界内，而不是把 `block.yaml` 当执行入口。

对 `Rust` block，推荐进一步收敛为：

- 代码 `SHOULD` 能作为普通 Rust crate 被 `moc` 直接依赖。
- `block.yaml` `SHOULD` 主要服务于 AI 理解、发现、契约校验和生成辅助。
- 人类编写的 Rust `moc` 不应被迫通过 `block.yaml` 或 registry 才能调用一个 Rust block。

## 5. block 标准契约

每个 `block` `MUST` 定义一份标准契约。标准契约至少包含以下字段。

### 5.1 基础标识

- `id`：全局唯一标识，建议使用稳定的短横线命名。
- `name`：可读名称。
- `version`：版本号。
- `status`：当前生命周期状态。
- `owner`：维护责任人或维护主体。

### 5.2 能力说明

- `purpose`：该 `block` 解决什么问题。
- `scope`：该 `block` 负责的范围。
- `non_goals`：该 `block` 明确不负责什么。

### 5.3 输入契约

- `inputs`：输入项清单。
- `input_schema`：输入结构、类型、必填项、默认值。
- `preconditions`：调用前必须满足的前置条件。

### 5.4 输出契约

- `outputs`：输出项清单。
- `output_schema`：输出结构、类型、格式要求。
- `postconditions`：成功执行后必须成立的结果条件。

### 5.5 实现声明

- `implementation.kind`：实现类型，当前推荐 `rust` 或 `tauri_ts`。
- `implementation.entry`：实现入口，例如 Rust 模块或前端入口文件。
- `implementation.target`：适用端，推荐 `backend`、`frontend` 或 `shared`。

规则：

- 当 `implementation.kind = rust` 时，`implementation.target` 可以是 `backend`、`shared`，也可以是面向前端的共享库逻辑。
- 当 `implementation.kind = tauri_ts` 时，`implementation.target` `MUST` 为 `frontend`。

### 5.6 执行约束

- `dependencies`：运行依赖。
- `side_effects`：可能产生的副作用。
- `timeouts`：超时约束。
- `resource_limits`：资源使用限制。

### 5.7 失败契约

- `failure_modes`：可预期失败类型。
- `error_codes`：标准错误码或标准错误类别。
- `recovery_strategy`：失败后的恢复或回退方式。

### 5.8 验证与评估

- `verification`：如何验证功能正确性。
- `evaluation`：如何评估结果质量。
- `acceptance_criteria`：达到什么条件才算验收通过。

### 5.9 开发态调试与可观测（Contract v1.1）

为避免“规范真源”分裂，自 `Contract v1.1` 起，开发态 `block` 契约新增以下 `MUST` 字段：

- `debug`
- `observe`
- `errors.taxonomy`

约束：

- 上述字段是开发态可调试、可观测基线的一部分，不得只在局部计划文档中定义。
- `errors.taxonomy` 需可被机器校验并与失败事件关联。

迁移窗口：

- 在迁移窗口内（截至 `2026-04-15`），缺失字段可先按 `warn` 报告。
- 自 `2026-04-16` 起升级为 `error`，阻断“声明为 active 的 block”通过校验。

## 6. 标准目录结构

每个 `block` `SHOULD` 采用统一目录结构，便于 AI 和人类工具一致识别：

```text
blocks/<block-id>/
  block.yaml
  README.md
  rust/          # optional
    src/
  tauri_ts/      # optional
    src/
  tests/
  examples/
  evaluators/
  fixtures/
```

各目录建议含义如下：

- `block.yaml`：标准契约文件。
- `README.md`：人类与 AI 可读说明。
- `rust/`：Rust 实现代码。
- `tauri_ts/`：Tauri + TypeScript 前端实现代码。
- `tests/`：功能验证。
- `examples/`：最小可运行示例。
- `evaluators/`：质量评估逻辑。
- `fixtures/`：测试与评估样例数据。

## 7. 标准文件要求

### 7.1 block.yaml

每个 `block` `MUST` 提供一个结构化契约文件，推荐使用 `YAML` 或 `JSON`。

该文件 `MUST` 包含：

- 基础标识
- 实现声明
- 输入输出定义
- 约束定义
- 验证定义
- 评估定义
- 验收标准
- 若 `status: active`，还必须包含 `debug`、`observe`、`errors.taxonomy`

该文件 `MUST NOT` 承载具体业务实现代码；它是描述文件，不是执行文件。

### 7.2 README.md

每个 `block` `MUST` 提供简洁说明文档，至少包括：

- 作用说明
- 适用场景
- 不适用场景
- 调用示例
- 验证方式
- 常见失败场景

### 7.3 示例

每个 `block` `SHOULD` 提供至少一个最小成功示例和一个最小失败示例，以帮助 AI 快速建立正确调用模式。

## 8. 接口规范

### 8.1 低歧义输入

输入接口 `MUST` 避免模糊语义。任何依赖主观解释的输入字段，都应被进一步结构化或加上枚举约束。

### 8.2 稳定输出

输出 `MUST` 尽量结构化，避免只有自然语言描述。若必须输出文本，也应定义格式边界和关键字段。

### 8.3 显式副作用

凡是会产生文件、网络请求、数据库写入、状态修改等副作用的 `block`，`MUST` 在契约中显式声明。

### 8.4 幂等性说明

若一个 `block` 多次执行可能产生不同结果，`MUST` 说明其幂等性条件和重复执行风险。

### 8.5 实现类型约束

- `Rust` block 应被视为库能力，可被后端、共享逻辑或启动器代码调用。
- `Rust` block 在条件允许时应优先提供可直接依赖的 crate 入口。
- `Tauri + TypeScript` block 应被视为前端能力，只能在前端启动器或前端运行上下文中调用。
- 任何跨端调用关系都应通过 `moc` 主入口或明确桥接层完成，而不是由 `block.yaml` 隐式表达。

### 8.6 MOC 内私有 block

- `moc` 可以包含内部私有 `block`。
- 内部私有 `block` 不要求进入公共 registry，但仍应有清晰契约。
- 当内部私有 `block` 产生跨 `moc` 复用价值时，应提升为公共 `block`。

## 9. 验证规范

### 9.1 基础验证

每个 `block` `MUST` 至少提供以下一种或多种验证方式：

- 单元测试
- 契约测试
- 样例断言
- 静态检查
- 结果检查器

### 9.2 验证范围

验证 `SHOULD` 覆盖：

- 正常路径
- 关键边界条件
- 可预期失败路径
- 主要副作用检查

### 9.3 验证自动化

验证 `SHOULD` 可自动运行，避免完全依赖人工肉眼判断。若无法自动化，必须说明原因与人工校验步骤。

## 10. 评估规范

### 10.1 评估目标

每个 `block` `MUST` 定义至少一个质量维度，例如：

- 准确性
- 完整性
- 一致性
- 性能
- 可恢复性

### 10.2 评估方式

评估 `MUST` 尽量转化为规则、分数、阈值或明确检查项，避免仅用“看起来不错”作为质量标准。

### 10.3 验收阈值

若存在量化指标，`block` `MUST` 给出最低可接受阈值。

## 11. 生命周期规范

每个 `block` `MUST` 具有明确生命周期状态。推荐使用以下状态：

- `draft`：草案阶段，契约可能变化，不可用于正式交付。
- `candidate`：候选阶段，已具备实现和验证，但尚未稳定。
- `stable`：稳定阶段，可进入正式项目编排。
- `deprecated`：弃用阶段，不建议新增使用。
- `retired`：退役阶段，不再维护。

只有 `stable` 状态的 `block` 才应进入关键项目路径。

## 12. 版本与兼容性规范

### 12.1 版本要求

每个 `block` `MUST` 标记版本号。版本变更 `SHOULD` 反映契约变化，而不仅仅是实现变化。

### 12.2 兼容性说明

当输入输出、错误行为、依赖或副作用发生变化时，`MUST` 在版本说明中明确兼容性影响。

### 12.3 破坏性变更

发生破坏性变更时，`SHOULD` 提供：

- 迁移说明
- 替代方案
- 兼容窗口

## 13. 依赖规范

### 13.1 最小依赖

`block` `SHOULD` 尽量减少非必要依赖，防止能力边界被外部复杂性污染。

### 13.2 显式依赖

所有运行依赖、环境依赖、权限依赖 `MUST` 被声明。

### 13.3 依赖风险

对于不稳定、昂贵或高风险依赖，`SHOULD` 标记风险等级，并提供替代或降级策略。

## 14. 可观测性规范

### 14.1 可观察执行结果

`block` `SHOULD` 提供可检查的运行结果、日志、状态输出或执行摘要，便于 AI 和人类判断发生了什么。

### 14.2 失败可解释

失败信息 `MUST` 可解释，不能只返回模糊错误。至少应说明：

- 失败发生在什么阶段
- 是输入问题、依赖问题还是内部实现问题
- 是否可以重试

## 15. block 验收清单

一个 `block` 只有在以下问题均能回答“是”时，才应被视为合格：

- 是否有单一且清晰的职责定义？
- 是否有结构化的输入输出契约？
- 是否有明确前置条件和后置条件？
- 是否有可执行验证方式？
- 是否有结果质量评估方式？
- 是否有失败模式和恢复策略？
- 是否有最小成功示例？
- 是否有版本和状态标记？
- 是否可以被其他 `block` 稳定组合？

## 16. 标准示例

以下是一个最小 `block.yaml` 示例：

```yaml
id: text-normalizer
name: Text Normalizer
version: 1.0.0
status: stable
owner: platform-team
purpose: Normalize raw text into a predictable structured form
scope:
  - trim whitespace
  - normalize line breaks
  - strip unsupported control characters
non_goals:
  - semantic rewriting
  - language translation
inputs:
  - name: raw_text
    type: string
    required: true
input_schema:
  type: object
  required:
    - raw_text
outputs:
  - name: normalized_text
    type: string
output_schema:
  type: object
  required:
    - normalized_text
preconditions:
  - input must be valid utf-8 text
postconditions:
  - output must not contain unsupported control characters
dependencies: []
side_effects: []
timeouts:
  default_ms: 1000
resource_limits:
  memory_mb: 64
failure_modes:
  - invalid_encoding
  - input_too_large
error_codes:
  - INVALID_ENCODING
  - INPUT_TOO_LARGE
recovery_strategy:
  - reject invalid input
verification:
  - run unit tests
  - check output against fixtures
evaluation:
  - output_format_stability
acceptance_criteria:
  - all tests pass
  - output matches fixture expectations
```

## 17. 结论

`blocks` 规范的核心，不是把组件写成统一格式，而是把组件变成可理解、可调用、可验证、可评估、可组合的稳定能力单元。只有当一个 `block` 具备这些性质时，它才适合作为 AI 软件生产中的基础积木。

如果没有这套规范，`blocks` 很容易退化为对“模块”或“脚手架”的另一种命名；而一旦遵循这套规范，`blocks` 才能真正成为面向 AI 时代的软件生产基础设施。
