# Rust Workspace 架构草图（平台迁移阶段）

## 1. 目标

固定迁移阶段的 Rust workspace 边界，确保仓库从“本地 descriptor/toolchain”平滑过渡到“包管理 + Rust 原生运行平台 + BCL 语言”模型：

- `block` 是库能力与可发布 package
- `moc` 是当前交付单元与迁移桥梁
- `BCL` 是长期主源码语言
- Rust 平台层负责统一运行时胶水与宿主边界

## 2. 当前推荐边界

- `blocks-contract`
  - 负责 `block.yaml` 和相关契约模型、字段校验、错误类型
  - 不负责文件扫描、解析依赖或执行
- `blocks-package`（规划中）
  - 负责 package id、version req、source、lockfile、resolved graph 模型
  - 不负责实际执行
- `blocks-registry`
  - 当前负责本地公共 `block` 发现
  - 下一阶段应扩为包索引/注册表访问层，而不再只是本地扫描器
- `blocks-runtime`
  - 当前负责单 `block` 执行胶水
  - 下一阶段应上升为 Rust 原生运行平台契约层，向不同 Rust 宿主暴露统一边界
- `blocks-runtime-host-*`（规划中）
  - 承载具体 Rust host 实现，例如 CLI host、`tokio` service host、Tauri host
- `blocks/*/rust`
  - 对可直接复用的 Rust `block`，继续提供独立 crate 入口
  - `block.yaml` 保留为 AI、校验和 package metadata 的描述层
- `blocks-moc`
  - 当前承载 `moc.yaml` 描述层
  - 负责 `moc.yaml` 解析、类型校验、依赖 `block` 校验、最小协议校验、`internal_blocks` 布局校验
  - 长期应转为迁移兼容层，而不是最终源码中心
- `blocks-bcl`
  - 当前承载受限 BCL MVP
  - 下一阶段应升级为真正的语言/编译器前端，并向包解析与运行平台降级输出
- `blocks-cli`
  - 负责 package/runtime/language 的公共命令入口
  - 当前仍兼容 `moc run` 与 `moc verify`
  - 长期应把 resolver、compiler、runtime host 管理收口为统一平台命令面
- `mocs/*`
  - 当前承载真实 `moc` 代码入口
  - 在迁移阶段继续有效，但长期将更多承担 BCL 编译输出或兼容载体角色

## 3. 关键依赖方向

允许：

```text
blocks-cli -> blocks-package
blocks-cli -> blocks-registry
blocks-cli -> blocks-runtime
blocks-cli -> blocks-moc
blocks-cli -> blocks-bcl
blocks-registry -> blocks-contract
blocks-registry -> blocks-package
blocks-runtime -> blocks-contract
blocks-runtime -> blocks-package
blocks-moc -> blocks-contract
blocks-moc -> blocks-registry
blocks-bcl -> blocks-package
blocks-bcl -> blocks-moc
blocks-bcl -> blocks-registry
mocs/* -> blocks-package (optional, package lock/resolve only)
mocs/* -> blocks/*/rust (preferred direct dependency during migration)
mocs/* -> blocks-registry (optional, descriptor lookup only)
mocs/* -> blocks-moc (optional, descriptor validation only)
```

禁止：

- `blocks-runtime` 直接依赖 `blocks-registry` 并把发现逻辑嵌入执行层
- `blocks-moc` 长期承担最终源码权威
- `CLI` 回收 `moc` 主逻辑
- 新增 Rust `block` 永久只接入单一 build-time 中央分发层

## 4. 当前实现与目标差距

当前仓库处于迁移态，仍有几个过渡问题：

- 目录和命名已切到 `mocs/`、`moc.yaml`、`blocks-moc`
- 描述层已经支持类型、协议和可选校验流，但 `verification.flows` 仍是过渡能力
- 示例已同时覆盖 `backend_app(console)`、`backend_app(service)`、`rust_lib` 与前端样例
- 当前公共 Rust block 已具备独立 crate 入口，主要示例已直接依赖这些 crate
- `moc run` 已可分发到 descriptor-only Rust backend moc 的真实入口
- 当前已包含最小 `frontend_app` 结构样例，并已提供本地静态预览与真实 Tauri 宿主运行入口
- 仍未形成真正的包解析/锁定模型
- 仍未形成高层 runtime host 抽象
- `BCL` 仍未成为主源码语言

因此，下一轮整改应优先补齐 package/runtime/language 三个缺口，而不是继续在本地 descriptor 工具体验上横向堆命令。

## 5. 迁移阶段约束

- `moc.main` 是当前自由代码入口，不是长期语言替身
- `moc` 仍可自由调用公共和内部私有 `block`
- 一个 `moc` 只允许一种交付类型
- 多服务、多终端应拆成多个 `moc`
- 长期新能力设计不应继续把 `moc.yaml` 当作唯一中心，而应服务于 BCL 编译与平台运行边界

## 6. 非目标

当前不在这一轮直接引入：

- 部署控制平面
- 非 Rust 为核心的运行平台
- 一步到位的全生态远程中心化服务

先把包管理、Rust 原生运行平台、BCL 语言三条主线补齐，再逐步退出过渡性的 `moc` 中心化思维。
