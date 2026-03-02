# Rust Workspace 架构草图

## 目标

为第一阶段 MVP 固定最小 Rust workspace 边界，确保 `contract / registry / runtime / cli` 四层职责清晰、依赖单向、错误可追踪。

## Crate 边界

- `blocks-contract`
  - 负责 `block.yaml` 的解析、最小契约模型、输入校验、标准校验问题结构。
  - 不负责文件扫描、执行调度或 CLI。
- `blocks-registry`
  - 负责扫描本地 `blocks/*/block.yaml`、索引 block、提供 list/show/search 所需数据。
  - 依赖 `blocks-contract` 解析契约。
  - 不负责执行 block。
- `blocks-runtime`
  - 负责后续的执行胶水层、执行结果和运行日志模型。
  - 第一轮只保留最小占位，避免提前把业务逻辑堆进运行时。
- `blocks-cli`
  - 负责参数解析和调用下层 crate。
  - 不复制契约校验或目录扫描逻辑。

## 依赖方向

只允许以下方向：

```text
blocks-cli -> blocks-registry
blocks-cli -> blocks-runtime
blocks-registry -> blocks-contract
blocks-runtime -> blocks-contract
```

禁止：

- `blocks-contract` 依赖任何上层 crate
- `blocks-registry` 依赖 `blocks-runtime`
- `blocks-runtime` 依赖 `blocks-registry`（至少在第一阶段前半段不允许）
- 任意核心逻辑回流到 CLI

## 公共类型归属

- 契约模型、字段 schema、校验问题、契约加载错误：放在 `blocks-contract`
- 发现结果、注册条目、目录扫描错误：放在 `blocks-registry`
- 执行记录、运行错误、后续日志模型：放在 `blocks-runtime`
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
blocks-cli::CliError -> prints RegistryError
```

## 当前非目标

- 不在这一轮引入 `composer`
- 不在这一轮引入远程 registry
- 不在这一轮引入复杂执行模型
- 不在这一轮引入前端耦合

这份草图是 P0 的架构基线；后续新增 crate 或跨层引用前，应先修改本文件再实现。

