# JLiverTool (Rust + GPUI)

使用 Rust 和 GPUI 框架重写的 JLiverTool。

## 项目结构

```
crates/
├── jlivertool/              # 主应用程序 (bin)
│   └── src/main.rs          # 入口点，后台服务和 UI 启动
├── jlivertool-core/         # 核心逻辑库 (lib)
│   └── src/
│       ├── bilibili/        # Bilibili API 和 WebSocket
│       │   ├── api.rs       # REST API 客户端 (房间信息、弹幕、礼物等)
│       │   ├── wbi.rs       # WBI 签名算法
│       │   └── ws.rs        # WebSocket 弹幕连接 (二进制协议解析)
│       ├── config.rs        # 配置存储 (JSON 持久化)
│       ├── events.rs        # 事件系统 (广播/订阅)
│       ├── messages.rs      # 消息类型 (弹幕、礼物、SC、舰长等)
│       └── types.rs         # 基础类型 (Cookies、RoomId、MedalInfo等)
└── jlivertool-ui/           # GPUI UI 库 (lib)
    └── src/
        ├── app.rs           # 主应用窗口
        ├── theme.rs         # 主题和样式
        └── views/           # 视图组件
            ├── main_view.rs  # 主视图
            └── danmu_item.rs # 弹幕条目
```

## 功能

- **弹幕显示**: 实时显示直播间弹幕，包含用户信息和粉丝勋章
- **礼物通知**: 显示礼物、舰长购买等信息
- **SC 显示**: SuperChat 消息展示
- **多房间合并**: 支持同时监听多个直播间 (规划中)
- **配置持久化**: 保存用户设置和窗口位置
- **WBI 签名**: 支持 Bilibili 新版 API 签名

## 依赖

- [GPUI](https://gpui.rs/) - Zed 的 GPU 加速 UI 框架，使用 GPUI Kit 发布的 `gpui-pre` 快照
- [gpui-component](https://github.com/longbridge/gpui-kit) - GPUI 组件库
- [Tokio](https://tokio.rs/) - 异步运行时
- [Reqwest](https://docs.rs/reqwest) - HTTP 客户端
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite) - WebSocket 客户端

## 构建

### 前置要求

- Rust 1.98.1（`rust-toolchain.toml` 自动选择，推荐使用 rustup 安装）
- 系统依赖 (macOS):
  - Xcode Command Line Tools

### 依赖配置

GPUI、平台后端和组件库必须使用匹配的版本；不要单独升级 `gpui-pre`。
Metal 着色器在运行时编译，无需安装完整 Xcode 的 Metal 工具。
工作区配置如下，macOS 仍固定 `core-text` 版本以解决依赖冲突：

```toml
[workspace.dependencies]
gpui = { package = "gpui-pre", version = "=0.3.8" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.8", features = ["font-kit", "x11", "wayland", "runtime_shaders"] }
gpui-component = "=0.7.1"
core-text = "=21.0.0"
```

`Cargo.lock` 纳入版本管理，用 `--locked` 构建以复现依赖。
升级改动与新功能评估见 [GPUI 升级评估](gpui-upgrade.md)。

### 编译

```bash
# 开发构建
cargo build --locked

# 发布构建
cargo build --release --locked

# 运行
cargo run --release --locked
```

### 生成文件大小

- Release 模式: ~8.4MB (macOS arm64)

## 配置

配置文件位于:

- macOS: `~/Library/Application Support/com.jlivertool.JLiverTool/config_v2.json`
- Linux: `~/.config/jlivertool/config_v2.json`
- Windows: `%APPDATA%\jlivertool\JLiverTool\config_v2.json`

## 与原版 Electron 的对比

| 特性     | Electron 版本 | Rust 版本 |
| -------- | ------------- | --------- |
| 内存占用 | ~200MB        | ~30MB     |
| 启动时间 | ~2s           | <0.3s     |
| CPU 占用 | 较高          | 极低      |
| 打包大小 | ~150MB        | ~8MB      |
| 跨平台   | ✓             | ✓         |

## 开发

### 运行测试

```bash
cargo test
```

### 检查代码

```bash
cargo clippy
cargo fmt --check
```

### 日志级别

设置 `RUST_LOG` 环境变量来调整日志级别:

```bash
RUST_LOG=debug cargo run
```

## TODO

- [ ] 设置窗口
- [ ] 礼物窗口
- [ ] SC 窗口
- [ ] 排行榜窗口
- [ ] 用户详情窗口
- [ ] 插件系统
- [ ] TTS 支持
- [ ] 扫码登录

## License

MIT License
