# blocks MVP 规划（MOC 模型）

## 1. 当前结论

最新模型应明确收敛为两层：

- `block`：稳定、可验证、可复用的库能力
- `moc`：`my own creation`，最终交付单元，拥有真实代码入口，负责对外提供产品能力

这意味着：

- `block.yaml` 只描述 `block`
- `moc.yaml` 只描述 `moc`
- 真正运行的是 `moc` 的代码入口，不是描述文件

## 2. moc 的核心定义

### 2.1 moc 才是交付单元

`moc` 不是流程图，也不是一份声明式绑定文件。`moc` 的本质是一个单一类型的可交付单元。

`moc` 的主入口必须是自由代码，例如：

- `src/main.rs`
- `src/lib.rs`
- `src/` + `src-tauri/`

主入口代码可以自由：

- 调用公共 `block`
- 调用当前 `moc` 内部私有 `block`
- 组织控制流
- 做错误处理
- 与其他 `moc` 按协议交互

### 2.2 moc 类型

每个 `moc` 只能属于一种类型：

- `rust_lib`
- `frontend_lib`
- `frontend_app`
- `backend_app`

当 `type = backend_app` 时，还必须声明：

- `backend_mode: console | service`

约束：

- 一个 `moc` 不能同时是前端和后端应用
- 一个 `moc` 不能同时是库和应用
- 多服务、多终端系统必须拆为多个 `moc`

### 2.3 moc 与 block 的关系

- `block` 对 `moc` 来说是库能力，不是应用
- `moc` 可以有内部私有 `block`
- 内部私有 `block` 只服务于当前 `moc`
- 当内部私有 `block` 跨单元复用时，应升级为公共 `block`

## 3. moc.yaml

建议第一阶段收敛到 `moc.yaml`，并让它作为唯一的上层描述文件。

建议最小字段：

- `id`
- `name`
- `type`
- `backend_mode`（仅 `backend_app`）
- `language`
- `entry`
- `uses.blocks`
- `uses.internal_blocks`
- `depends_on_mocs`
- `protocols`
- `verification`
- `acceptance_criteria`

关键边界：

- `moc.yaml` 只做描述、校验、发现和生成辅助
- `moc.yaml` 不能替代真实入口代码
- `moc.yaml` 不应把 `moc.main` 限制成固定步骤图

## 4. 多 MOC 系统

复杂系统不应由单个 `moc` 同时承载多个服务或多个终端。

正确方式：

1. 按最终交付形态拆分多个 `moc`
2. 每个 `moc` 保持单一类型
3. 通过明确协议连接这些 `moc`

协议要求：

- 输入输出契约明确
- 版本边界明确
- 错误行为可观测
- 可独立测试

## 5. 第一阶段 MVP（修正后）

第一阶段不再以“声明式顶层组合器”作为目标，而应以“可运行的单一类型 moc”作为目标。

必须具备：

1. 5 个左右核心公共 `block`
2. 统一 `block` 契约与 registry
3. 单 `block` 运行时
4. `moc.yaml` 最小描述层
5. 至少一个真实 `backend_app` 示例
6. 至少一个 `rust_lib` 或 `frontend_lib` 示例
7. 两份 AI 技能文档：
   - 如何创建 `block`
   - 如何创建 `moc`

### 第一阶段最小示例

建议示例从“hello-pipeline”调整为“hello-world-console”：

- `moc.type = backend_app`
- `moc.backend_mode = console`
- `moc.main` 在 Rust 中自由调用：
  - 一个公共 `block`，如 `core.console.write_line`
  - 或一个当前单元的内部私有 `block`
- 向控制台输出 `hello world`

这个示例更符合真实模型，因为它证明：

- `moc.main` 是自由代码
- `block` 是被调用的能力
- `moc` 自己也可以拥有内部私有能力

## 6. 当前代码与目标模型的差距

当前仓库已经完成了一轮较早的顶层原型，并已完成命名迁移，但仍与最新模型存在差距：

- 路径和命名已迁到 `mocs/`、`moc.yaml`、`blocks-moc`
- `blocks-moc` 已回到描述层，但可选 `verification.flows` 仍是过渡期能力，不应重新膨胀为运行时
- 当前已补齐 `hello-message-lib` 这样的 `rust_lib` 示例，并已支持最小跨 `moc` 协议校验
- 当前公共 Rust `block` 已具备独立 crate 入口，证明 `moc` 可直接依赖 block 代码
- 当前已提供 `blocks moc run` 作为统一入口，Rust backend `moc` 会优先分发到真实启动器
- 当前已补齐 `frontend_app` 的最小结构样例，并已提供真实 Tauri 宿主与安全的 headless probe 路径，用于固定 Tauri + TypeScript 边界
- 技能文档已经迁到 `build-moc`，并已明确“先写真实 launcher，再决定是否加验证 flow”

因此，当前最优先任务不是继续扩功能，而是先把模型迁回 `moc`。

## 7. 第二阶段 MVP（BCL 修正后）

第二阶段再引入最小 `BCL`，但它的目标不应是编译出“顶层流程图”，而应是辅助生成和校验 `moc` 结构。

建议输出：

- `moc.yaml`
- 内部执行计划
- `moc` 入口辅助代码骨架

`BCL` 的价值应放在：

- 校验 `block` 使用是否合法
- 校验 `moc` 类型边界是否合法
- 校验跨 `moc` 协议是否匹配

## 8. 架构原则

- `block` 是库能力，不是最终产品
- `moc` 是最终产品，不是流程图
- 描述文件只做描述，不做运行入口
- 主入口代码必须自由，但必须受契约约束
- 一个 `moc` 只承载一种类型
- 多端、多服务必须拆为多个 `moc`

## 9. 当前最高优先级

下一轮实现应优先完成：

1. 为前端 `moc` 增加最小本地运行路径，而不只停留在结构样例
2. 继续让统一入口覆盖更多 `moc` 类型，而不只覆盖 Rust backend
3. 从静态前端预览继续推进到真实 Tauri 宿主运行
4. 将更多示例迁到“直接依赖 block crate”的默认路径

在这一步完成前，不建议继续推进 Tauri 前端组装器或 `BCL` 实现。
