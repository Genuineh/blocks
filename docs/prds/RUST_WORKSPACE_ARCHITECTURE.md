# Rust Workspace 架构草图

## 目标

为第一阶段 MVP 固定最小 Rust workspace 边界，确保 `contract / registry / runtime / composer / core / cli / app-launcher` 职责清晰、依赖单向、错误可追踪，并为后续前端启动器代码留出清晰边界。

## Crate 边界

- `blocks-contract`
  - 负责 `block.yaml` 的解析、最小契约模型、实现类型声明、输入校验、标准校验问题结构。
  - 不负责文件扫描、执行调度或 CLI。
- `blocks-registry`
  - 负责扫描本地 `blocks/*/block.yaml`、索引 block、提供 list/show/search 所需数据。
  - 依赖 `blocks-contract` 解析契约。
  - 不负责执行 block。
- `blocks-runtime`
  - 负责后续的执行胶水层、执行结果和运行日志模型。
  - 当前负责单 block 的执行胶水：输入校验、执行调用、输出校验。
  - 不负责目录扫描和组合编排。
- `blocks-composer`
  - 负责 `app.yaml` 的解析、bind 校验、类型匹配和执行计划生成。
  - 依赖 `blocks-registry` 做 block 发现。
  - 保持轻量描述层，不演化为复杂工作流引擎。
  - 在目标架构里，它是过渡验证层，不是最终 app 运行入口。
- `blocks-core`
  - 负责静态链接当前内置 block 的 Rust 实现，并向 runtime 暴露统一 `BlockRunner`。
  - 只承载 block 具体能力，不承载目录扫描、契约校验或 app 级编排。
- `blocks-cli`
  - 负责参数解析和调用下层 crate。
  - 不复制契约校验或目录扫描逻辑。
- `apps/*/backend`
  - 负责真实 app 启动、加载执行计划、串行调用 runtime。
  - app 的对外行为应在这里落地，而不是写回 `app.yaml` 或 CLI。

App 层约束：

- app 最终应由 Rust 启动器代码承载。
- 若存在前端，则前端入口由 Tauri + TS 启动器承载。
- `blocks-composer` 可以辅助校验或生成，但不能替代这些启动器。

## 依赖方向

只允许以下方向：

```text
blocks-cli -> blocks-registry
blocks-cli -> blocks-runtime
blocks-cli -> blocks-composer
blocks-cli -> blocks-core
blocks-composer -> blocks-registry
blocks-composer -> blocks-contract
blocks-registry -> blocks-contract
blocks-runtime -> blocks-contract
blocks-core -> blocks-runtime
apps/*/backend -> blocks-composer
apps/*/backend -> blocks-core
apps/*/backend -> blocks-registry
apps/*/backend -> blocks-runtime
```

禁止：

- `blocks-contract` 依赖任何上层 crate
- `blocks-registry` 依赖 `blocks-runtime`
- `blocks-runtime` 依赖 `blocks-registry`（至少在第一阶段前半段不允许）
- `blocks-composer` 反向承载 block 实现代码
- 任意核心逻辑回流到 CLI
- app 启动逻辑回流到 `blocks-composer`

## 公共类型归属

- 契约模型、字段 schema、校验问题、契约加载错误：放在 `blocks-contract`
- 发现结果、注册条目、目录扫描错误：放在 `blocks-registry`
- 执行记录、运行错误、后续日志模型：放在 `blocks-runtime`
- 组合清单、bind 校验、执行计划和组合错误：放在 `blocks-composer`
- 静态 block 执行映射：放在 `blocks-core`
- 纯展示和退出码：放在 `blocks-cli`

原则：公共类型只放在最低合理层，避免同一概念在多个 crate 重复定义。

## 错误流转

- 每个 crate 定义自己的错误类型。
- 下层错误向上层传播时保留原始 source，不转换为临时字符串。
- CLI 只负责最终格式化输出，不吞掉结构化错误来源。

当前错误链：

```text
blocks-contract::ContractLoadError
blocks-registry::RegistryError -> wraps ContractLoadError / io::Error
blocks-runtime::RuntimeError -> wraps validation / execution errors
blocks-composer::ComposeError -> wraps manifest / bind / plan failures
app launcher -> orchestrates registry / composer / runtime failures
CLI surface -> prints lower-level errors without swallowing the source
```

## 当前非目标

- 不在这一轮引入远程 registry
- 不在这一轮引入复杂控制流（并行、分支、恢复图）
- 不在这一轮引入前端耦合

这份草图是 P0 的架构基线；后续新增 crate 或跨层引用前，应先修改本文件再实现。
