# Quick-SSH 文档

**Quick-SSH** 是一个跨平台 SSH 连接管理工具，提供 **TUI 界面**与 **Docker 风格 CLI** 双模式操作。基于 Rust 实现，无需 Node.js 运行时，即下即用。

<Callout title="项目状态">
  Quick-SSH v2.0 已完成 Rust 完整重写，支持 Windows / Linux / macOS 三大平台，提供 6 种包管理器安装方式。
</Callout>

## 核心特性

<CardGroup cols={2}>
  <Card title="TUI 界面" icon="LayoutPanelLeft">
    事件驱动的终端 UI，支持主机列表浏览、搜索、标记、连接和 Ping 检测，支持新增/编辑主机弹窗表单。
  </Card>
  <Card title="CLI 命令" icon="Terminal">
    Docker 风格的命令行接口，支持 ps/add/rm/connect/export/import 等子命令，一键连接主机。
  </Card>
  <Card title="文件拖拽上传" icon="Upload">
    SSH 连接后，将文件或目录拖入终端窗口即可自动启动 SFTP 上传，显示每文件进度条和总进度。
  </Card>
  <Card title="渐进式配置解析" icon="FileJson">
    兼容标准 OpenSSH 格式，只管理 Host/HostName/User/Port/IdentityFile，其余指令完整保留。
  </Card>
  <Card title="多包管理器支持" icon="Blocks">
    支持 Scoop、WinGet、Homebrew、AUR、APT 等 6 种包管理器，一键安装。
  </Card>
  <Card title="纯 Rust 实现" icon="Code">
    单一二进制文件，无外部运行时依赖，高性能低内存，跨平台编译。
  </Card>
</CardGroup>

## 快速导航

<CardGroup cols={3}>
  <Card title="安装指南" icon="Download" href="/installation">
    通过包管理器或直接下载安装 Quick-SSH
  </Card>
  <Card title="快速入门" icon="Play" href="/getting-started">
    5 分钟上手 TUI 和 CLI 基本操作
  </Card>
  <Card title="CLI 命令参考" icon="Terminal" href="/cli-reference">
    完整的子命令参考文档
  </Card>
  <Card title="架构设计" icon="Layers" href="/architecture">
    Rust 模块化架构设计详解
  </Card>
  <Card title="发布流程" icon="Rocket" href="/release-process">
    版本发布与包管理器维护指南
  </Card>
  <Card title="路线图" icon="Map" href="/roadmap">
    当前版本与未来规划
  </Card>
</CardGroup>

## 技术栈

<Properties>
  <Property name="语言" type="Rust">
    基于 Rust 2021 Edition 编写，使用 workspace 管理多 crate
  </Property>
  <Property name="TUI 框架" type="ratatui">
    使用 ratatui 0.29 + crossterm 0.28 构建事件驱动终端界面
  </Property>
  <Property name="SSH 协议" type="OpenSSH">
    复用系统 OpenSSH 客户端（ssh/scp），不自行实现 SSH 协议
  </Property>
  <Property name="CLI 框架" type="clap">
    使用 clap 4 的 derive 模式定义命令行接口，支持自动补全
  </Property>
  <Property name="配置格式" type="OpenSSH">
    兼容 ~/.ssh/config 标准格式，渐进式解析
  </Property>
</Properties>
