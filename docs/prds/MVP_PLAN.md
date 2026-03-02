# blocks MVP 规划（Rust + Tauri）

## 目标

基于当前白皮书，第一阶段先验证一个最小但完整的闭环：

1. AI 可以基于统一规范创建新的 `block`。
2. AI 可以使用已有 `block` 组装出一个独立可运行程序。
3. 运行时可以在执行前后做契约校验、记录结果，并提供基础失败恢复。

第二阶段再补上 `BCL`，把“AI 如何使用 blocks”也纳入编译期检查。

这个顺序的核心是：先证明 `block` 和运行时成立，再证明 `block` 组合语言成立。

## 开发与迭代原则

### 架构优先

实现前先做宏观架构分析，再进入局部细节。每一轮迭代都应先回答：

- 这次变更影响哪些模块和边界
- 公共类型应该归属在哪一层
- 依赖方向是否仍然单向、清晰
- 失败路径和恢复路径是否仍然可解释

如果这些问题没有先收敛，就不应该直接进入实现。

### TDD 优先

默认采用 `red/green TDD`：

1. 先写失败测试，证明当前行为不满足目标。
2. 只做最小实现让测试通过。
3. 在测试护栏内重构，清理命名、边界和重复逻辑。

第一阶段尤其要优先为契约解析、运行时校验和组合绑定写失败测试。

### 安全迭代

每次只推进一个清晰边界内的变化，避免同时改动契约模型、执行模型和前端 block 组装层。优先通过小步增量方式推进，始终让已有测试和示例应用保持可回归。

### 严格验收

验收标准从严，不以“功能演示成功一次”为完成。任何阶段都应同时满足：

- 架构边界没有退化
- 新行为有测试证明
- 失败路径可观测、可定位
- 文档、示例和操作入口同步更新
- 结果是可维护资产，而不是临时补丁

## 总体技术路线

### 核心判断

- 后端核心全部用 `Rust`，保证类型约束、可执行验证、交付为单二进制更容易。
- 前端端侧用 `Tauri + TypeScript`，用于组装前端能力 block，并承载前端交互、预览和调试；底层契约、发现与运行时逻辑仍由 Rust 核心负责。
- `block.yaml` 只是描述文件，不是实现文件；真正的 block 能力必须由 `Rust` 或 `Tauri + TS` 代码承载。
- `Rust` block 本质上是库能力，可应用于后端、共享逻辑，必要时也可承载前端共享库逻辑。
- `Tauri + TS` block 是前端能力，只能在前端启动器中运行。
- 第一阶段不做复杂分布式运行时，不做远程仓库，不做在线注册中心。
- 第一阶段也不做完整 `BCL` 编译器，只做一个更轻的描述与验证层。

### 最小架构

```text
blocks/
  crates/
    blocks-contract/
    blocks-registry/
    blocks-runtime/
    blocks-cli/
    blocks-composer/
  blocks/
    core.http.get/
    core.fs.read_text/
    core.fs.write_text/
    core.json.transform/
    core.llm.chat/
  skills/
    create-block.md
    compose-app.md
  apps/
    <app-name>/
      app.yaml          # optional descriptor / validation metadata
      backend/
        Cargo.toml
        src/main.rs     # Rust launcher
      frontend/         # optional
        src-tauri/
        src/            # Tauri + TS launcher
```

这个结构有一个硬约束：`block.yaml` 只描述，不承载实现；`app.yaml` 只描述 app 使用了哪些 block 和哪些约束，不能替代真正的 app 启动代码。实际对外提供能力的，是 app 内的 Rust 启动器和可选的 Tauri + TS 前端启动器。

### 运行模型

第一阶段把系统收敛成 5 个核心对象：

- `Block Contract`：`block.yaml`，定义输入、输出、前后置条件、失败策略。
- `Block Implementation`：由 `Rust` 或 `Tauri + TS` 代码实现具体能力。
- `Registry`：扫描本地 `blocks/` 目录，发现可用 block。
- `Runtime`：按契约执行 block，做验证、日志、失败恢复。
- `App Launcher`：用 Rust 和可选的 Tauri + TS 代码启动 app，并在代码中组织 block 调用。

为了控制复杂度，第一阶段还需要遵守三条架构约束：

- `Contract` 负责定义和验证，不负责执行。
- `Registry` 负责发现和索引，不负责调度。
- `Runtime` 负责执行胶水，不承载具体 block 业务。
- `App Launcher` 才是 app 的真实运行入口；app 逻辑应写在启动器代码中，而不是只写在描述文件中。

## 第一阶段 MVP

### 范围

第一阶段只做“可运行闭环”，不追求生态完整。

进入实现前，先完成一份简短架构草图，明确：

- crate 边界
- 公共模型归属
- 错误类型分层
- CLI 到 runtime 的调用链

没有这份草图，不进入编码阶段。

必须有的交付物：

1. 5 个左右核心 `block`
2. 统一 Contract SDK
3. 本地 Registry
4. 可执行 Runtime
5. 一个可用的 `blocks` CLI
6. 两份给 AI 使用的 `skills`
7. 一个由 Rust 启动器承载的独立示例程序
8. 一个最小 Tauri + TS 前端启动器示例（用于前端能力 block）

### 第一阶段必须实现的能力

#### 1. Contract SDK（最优先）

用一个轻量 Rust crate 统一处理所有契约逻辑，避免每个 block 自己手写校验。

建议职责：

- 解析 `block.yaml`
- 输入输出 schema 校验
- 前置条件检查
- 后置条件检查
- 标准错误结构
- 标准执行结果结构

建议 crate：`crates/blocks-contract`

建议核心类型：

```rust
pub struct BlockContract { ... }
pub struct ExecutionInput { ... }
pub struct ExecutionOutput { ... }
pub struct ValidationIssue { ... }

pub trait ContractValidator {
    fn validate_input(&self, input: &Value) -> Result<(), Vec<ValidationIssue>>;
    fn validate_output(&self, output: &Value) -> Result<(), Vec<ValidationIssue>>;
}
```

第一阶段不需要做完整 DSL 级条件表达式，可先支持：

- 必填字段
- 基础类型
- 字符串长度 / 枚举
- 数值范围
- 简单存在性断言

测试要求：

- 先写契约加载失败测试
- 先写输入缺失 / 类型错误测试
- 再实现最小通过路径

#### 2. Registry（本地发现）

第一阶段只做本地文件夹扫描，不做远程源。

建议职责：

- 扫描 `blocks/*/block.yaml`
- 读取 block 元数据
- 按 `id` 建索引
- 提供 `list` / `show` / `search`

建议 crate：`crates/blocks-registry`

建议 CLI：

- `blocks list`
- `blocks show <block-id>`
- `blocks search <keyword>`

这一步解决“AI 不知道当前项目有哪些 block 可用”的问题。

测试要求：

- 目录不存在
- `block.yaml` 缺失或损坏
- 重复 `id`
- 正常发现路径

#### 3. Runtime（薄执行层）

运行时只做胶水，不做复杂调度器。

建议职责：

- 根据 `block_id` 加载 contract + implementation
- 执行前做输入校验与前置条件检查
- 调用 block
- 执行后做输出校验与后置条件检查
- 记录执行日志
- 按 `failure_modes` 做最小恢复动作

建议 crate：`crates/blocks-runtime`

建议统一入口：

```rust
runtime.execute("core.http.get", input)
```

建议先支持 3 种失败恢复策略：

- `fail_fast`
- `retry_once`
- `fallback_to_default`

这已经足够支撑 MVP 验证，不要在第一阶段引入复杂规则引擎。

测试要求：

- 输入校验失败时不执行 block
- block 执行失败时返回结构化错误
- 输出校验失败时能阻断成功结果
- `retry_once` 不会无限重试

#### 4. Composer（轻量描述与验证层，不是最终 app 运行时）

为了让 AI 能先描述 block 之间的关系，第一阶段可以保留一个极简组合格式；但它不应被视为最终 app 运行时。

建议使用 `app.yaml`：

```yaml
name: hello-pipeline
entry: main
flows:
  - id: main
    steps:
      - id: fetch
        block: core.http.get
      - id: parse
        block: core.json.transform
      - id: save
        block: core.fs.write_text
    binds:
      - from: input.url
        to: fetch.url
      - from: fetch.body
        to: parse.source
      - from: parse.result
        to: save.content
```

建议 crate：`crates/blocks-composer`

第一阶段它只做：

- 解析组合清单
- 校验步骤引用的 block 是否存在
- 校验基础绑定是否存在、类型是否兼容
- 作为生成或校验 app 启动器代码的辅助输入

这一步本质上是“BCL 前的过渡层”，也是“代码启动器前的验证层”，不是 app 本身。

架构约束：

- `composer` 只产出描述、校验结果或辅助执行计划，不直接承载 block 实现
- `composer` 不能替代 app 的 Rust / Tauri 启动器
- `composer` 不引入复杂控制流，保持为 BCL 的过渡层

测试要求：

- 未知 block
- 缺失 bind
- 类型不兼容
- 串行成功路径

#### 5. CLI（AI 与人类共同入口）

CLI 是第一阶段真正的产品入口，比桌面 UI 更重要。

建议 crate：`crates/blocks-cli`

第一阶段建议命令：

- `blocks list`
- `blocks show <block-id>`
- `blocks run <block-id> --input <file>`
- `blocks compose run <app.yaml> --input <file>`
- `blocks init block <block-id>`
- `blocks init app <app-name>`
- `blocks validate <path>`

其中：

- `init block` 负责生成标准目录模板
- `init app` 负责生成最小组合应用模板
- `validate` 负责检查 block 或 app 清单是否合法

这会直接成为 AI 使用本项目的标准操作面。

架构约束：

- CLI 只编排调用，不复制契约验证或执行逻辑
- 任何核心校验都必须落在底层 crate，可被测试复用

#### 6. 核心 Blocks（先做 5 个）

第一阶段只做通用、高复用、低歧义能力。

建议最小集合：

1. `core.fs.read_text`
2. `core.fs.write_text`
3. `core.http.get`
4. `core.json.transform`
5. `core.llm.chat`

可选第 6 个：

6. `core.template.render`

选择原则：

- 足够通用
- 输入输出容易结构化
- 容易验证成功与失败
- 能组合出真实小程序

暂时不要做数据库、鉴权、异步任务、长流程工作流。

#### 7. Skills（第一阶段关键交付）

第一阶段不是先做“大量 block”，而是给 AI 一套能持续生产 block 的稳定操作指南。

建议提供两份技能文档：

- `skills/create-block.md`
- `skills/compose-app.md`

`create-block.md` 负责约束 AI：

- 如何判断某个需求是否应该沉淀为 block
- 如何使用 `blocks init block`
- 如何填写 `block.yaml`
- 如何编写最小实现
- 如何补齐测试与示例
- 如何用 `blocks validate` 自检

`compose-app.md` 负责约束 AI：

- 如何先枚举当前可用 blocks
- 如何识别能力缺口
- 如何生成 `app.yaml`
- 如何处理输入输出绑定
- 如何运行 `blocks compose run`
- 如何在失败时回退到补充 block

这两份文档就是第一阶段“AI 可稳定使用”的真正杠杆。

#### 8. 最小 Tauri 前端 Block 启动器

第一阶段中，`Tauri + TS` 的职责不是单纯做桌面壳，而是作为前端技术栈去启动前端能力 block；前端功能编排也应写在前端启动器代码中。

建议页面只包含：

- 当前 block 列表
- block contract 查看器
- 前端 block 组装面板
- 预览 / 调试面板
- 最近执行日志

建议 Tauri 复用 Rust 后端能力完成契约读取、发现和执行，不在前端复制核心业务逻辑。

前端范围要严格控制，否则会稀释 MVP。

架构约束：

- Tauri 必须复用核心契约和可共享能力定义
- 前端负责前端能力 block 的启动与交互，不是新的底层业务层

### 第一阶段推荐目录结构

```text
blocks/
  block.yaml
  README.md
  src/
  tests/
  examples/

apps/
  <app-name>/
    app.yaml
    input.example.json
    README.md
```

### 第一阶段验收标准

做到以下几点，就算第一阶段 MVP 成立：

1. 可以执行 `blocks init block demo.echo` 生成标准模板。
2. AI 可以按 `skills/create-block.md` 新增一个 block，并通过 `blocks validate`。
3. 可以执行 `blocks list` 和 `blocks show` 发现本地 blocks。
4. 可以执行 `blocks run <block-id>`，且运行前后有契约校验。
5. 可以执行 `blocks compose run apps/hello-pipeline/app.yaml` 顺序跑通一个小程序。
6. 失败时至少能输出结构化错误，并触发最小恢复策略。
7. 至少一个 app 能通过 Rust 启动器对外提供功能；若包含前端，则由 Tauri + TS 启动器承载前端能力 block。

补充严格验收门槛：

- 上述每项能力都必须有自动化测试或可重复的示例验证，不接受只靠人工演示。
- 所有核心路径都要覆盖至少一个失败测试和一个成功测试。
- `runtime`、`composer`、`cli` 的职责没有交叉污染。
- 文档、技能文件、示例程序与 CLI 行为一致，不存在隐性操作前提。
- 若为实现某项能力引入了明显更高的复杂度，则该方案应被回退重审。

### 第一阶段建议示例程序

建议做一个“网页摘要器”作为展示样例：

1. `core.http.get` 拉取页面内容
2. `core.llm.chat` 生成摘要
3. `core.fs.write_text` 写入本地文件

这个示例足够小，但能证明：

- block 可独立运行
- block 可组合
- AI 可以按技能文件生成 block 和程序
- 运行时能做校验与日志记录

## 第二阶段 MVP（加入 BCL）

### 第二阶段目标

在第一阶段基础上，把“组合清单”升级为“可编译的组合语言”，重点不是追求语言复杂度，而是提升 AI 组合时的可检查性。

第二阶段只做一个极简 `BCL` 子集：

- `product`
- `use`
- `input`
- `output`
- `flow`
- `bind`
- `verify`
- `recover`

不要在第二阶段引入完整宏系统、泛型、复杂控制流。

进入第二阶段前提：

- 第一阶段测试基线稳定
- 第一阶段架构边界未被前端或示例侵蚀
- `app.yaml` 过渡层已经验证了最小组合模型成立

### 第二阶段新增能力

#### 1. BCL Parser

用 Rust 实现一个最小语法解析器。

建议技术：

- `winnow` 或 `nom` 做 parser
- 或者直接用 `pest`，优先选择开发速度

输出：

- 抽象语法树（AST）
- 语法错误诊断

#### 2. BCL Semantic Checker

这是第二阶段的核心价值，不是 parser 本身。

最小语义检查应包含：

- 引用的 block 是否存在
- block 状态是否允许使用
- `bind` 的源/目标是否存在
- 类型是否兼容
- 关键输出是否可达
- 是否存在未处理失败分支

这一步直接把“AI 用错 block”从运行期前移到编译期。

测试要求：

- 未知 block 诊断
- 缺失 bind 诊断
- 类型不匹配诊断
- 输出不可达诊断

#### 3. BCL Compiler（先编译到 app manifest）

第二阶段不必直接编译成复杂运行产物，先编译到第一阶段的 `app.yaml` 或内部执行计划即可。

即：

`BCL source -> AST -> semantic checks -> execution plan`

这样可以复用第一阶段的 runtime，不要重复造轮子。

#### 4. 更好的诊断反馈

第二阶段需要把错误反馈显式化，让 AI 更容易修正：

- 未知 block
- 版本不兼容
- 缺失 bind
- 类型不匹配
- 输出不可达
- 恢复策略缺失

这是第二阶段最重要的“AI 友好性”能力。

### 第二阶段验收标准

做到以下几点，就算第二阶段成立：

1. AI 可以生成一段合法的最小 BCL。
2. 编译器能解析并生成 AST。
3. 编译器能在运行前发现常见 block 使用错误。
4. BCL 可以编译到第一阶段的执行计划并实际运行。
5. 错误信息足够结构化，AI 能据此自动修正一次以上。

补充严格验收门槛：

- BCL 新增的复杂度必须被限制在“编译前检查”范围内，不能破坏第一阶段薄运行时原则。
- 语义错误必须先通过自动化测试固定，再实现修复。
- 编译到执行计划的结果必须与第一阶段运行模型兼容，不能平行造第二套执行系统。

## 实施顺序建议

### 里程碑一：先把后端核心跑通

顺序：

1. 先做架构草图和 crate 依赖审查
2. 建 Rust workspace
3. 先写 `blocks-contract` 失败测试，再实现最小契约能力
4. 先写 `blocks-registry` 失败测试，再实现本地发现
5. 先写 `blocks-runtime` 关键失败测试，再实现执行闭环
6. 实现 `blocks-cli`

这个阶段先不接 Tauri。

### 里程碑二：补齐最小 block 与示例

顺序：

1. 先写 `app.yaml` 绑定与校验失败测试
2. 做 `app.yaml` 描述与校验
3. 做 5 个核心 blocks
4. 用 Rust 启动器做 `hello-pipeline` 示例
5. 写两份 skills

这个阶段结束时，项目已经具备对外演示价值。

### 里程碑三：接入 Tauri 前端 Block 启动器

用 `Tauri + TS` 启动最小前端能力 block，并复用现有 Rust 核心能力：

1. 浏览 blocks
2. 查看 contract
3. 启动前端 block
4. 运行或预览前端结果
5. 查看日志

不要先做复杂前端工作流编辑器。

进入本里程碑前，先确认前端只负责前端启动与交互，避免前端反向定义核心契约和执行行为。

### 里程碑四：推进 BCL

顺序：

1. 先做最小语法
2. 再做语义检查
3. 最后复用 runtime 执行

不要反过来先做“复杂语言设计”。

每一步都必须先用测试固定预期错误，再补实现。

## 明确不做的事

为了保证 MVP 成立，以下内容应明确延后：

- 远程 block 仓库
- 多语言 block 实现（第一阶段只支持 Rust）
- 沙箱执行系统
- 分布式调度
- GUI 流程编排器
- 复杂权限系统
- 完整版本解析与依赖求解器
- 完整 BCL 生态和标准库

这些都不是验证核心假设所必需的。

## 结论

最小可行路线非常明确：

- 第一阶段验证“稳定 block + 统一契约 + 本地发现 + 薄运行时 + AI skills”这条链路。
- 第二阶段验证“BCL 能让 AI 对 block 的使用在编译前被检查”。

如果第一阶段做成，`blocks` 的基础假设就已经成立：

- AI 可以稳定生产 block
- AI 可以稳定使用 block
- 项目可以由 block 组装出来

如果第二阶段再做成，`blocks` 才从“可运行框架”升级为“可编译的软件生产模型”。
