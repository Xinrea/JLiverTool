---
name: jlivertool-plugin
description: 开发、修改或调试 JLiverTool 插件（应用数据目录 plugins/ 下的浏览器端 HTML/JS 扩展，通过自动注入的 jliverAPI 接收弹幕、礼物、舰长、醒目留言和直播间状态事件）。默认直接写入用户数据目录。当用户要求为 JLiverTool 写插件、编写或检查 meta.json、用 jliverAPI 监听事件、或排查插件打不开/连不上/收不到事件时使用。Build, modify, or debug JLiverTool plugins.
---

# JLiverTool 插件开发

JLiverTool 插件就是一个**静态网页**：没有构建步骤、不引入 npm 依赖、也不运行在应用内嵌的 webview 中。插件目录由应用内置的 HTTP 服务器（默认 `http://127.0.0.1:8080`）提供，页面在**系统默认浏览器**里打开，`jliver-api.js` 由服务器自动注入，它连上应用的 WebSocket 服务器（默认 `ws://127.0.0.1:8081`）并把直播事件转发给 `window.jliverAPI`。

所以插件的全部能力边界就是本文档记录的 API：接收事件、打开 URL、查询服务器信息。没有发送弹幕、读写应用配置、访问文件系统之类的接口；需要新能力必须改应用（`crates/jlivertool-plugin`）。

## 插件写到哪里

应用只扫描**用户数据目录**下的 `plugins/`。

默认写作目录（按操作系统）：

- macOS：`~/Library/Application Support/com.jlivertool.JLiverTool/plugins/`
- Windows：`%APPDATA%\jlivertool\JLiverTool\config\plugins\`
- Linux：`~/.config/jlivertool/plugins/`

不确定实际路径时，让用户在应用里点「设置 → 插件管理 → 插件说明 → 打开插件目录」，以打开的位置为准。

## 参考资料

| 需要什么                                                                    | 去哪里看                                                                                                             |
| --------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `meta.json` 字段、`jliverAPI` 全部方法、事件频道与字段、底层 WebSocket 协议 | [references/api.md](references/api.md)                                                                               |
| 可直接复制的最小插件                                                        | [template/meta.json](template/meta.json)、[template/index.html](template/index.html)                                 |
| 在线文档                                                                    | [https://xinrea.github.io/JLiverTool/plugin-document.html](https://xinrea.github.io/JLiverTool/plugin-document.html) |

## 工作流程

1. **先定清楚三件事再写代码**：插件要展示或记录什么；需要哪些事件（频道名 + 字段，见 `references/api.md`）；是否需要观众用弹幕命令交互（例如 `#draw 3 4 #ff0000`）。
2. **建目录**：在数据目录的 `plugins/<folder>/` 下创建。`<folder>` 用纯 ASCII + 短横线（如 `my-plugin`），因为它会成为 URL 路径段。修改已安装插件时，同样改数据目录里的那份。
3. **从模板起步**：复制 `template/` 的两个文件，改 `meta.json` 的 `id`/`name`/`desc`/`version`，再改 `index.html`。
4. **本地跑通**，按下文步骤和验收清单确认，不要只交代码。

## 本地调试

因为默认就写在数据目录，写完后在插件管理里点「刷新」即可。

- **不要用 symlink 指到插件目录之外**：HTTP 服务器会 canonicalize 真实路径并拒绝越界访问，页面只会得到 403。
- 点「打开」会在浏览器打开 `http://127.0.0.1:<http_port>/<folder>/index.html?ws_port=<ws_port>`。
- 插件管理里的「删除」会直接删掉磁盘上的插件目录，不是禁用开关，调试时不要用它清理列表。
- 改了插件文件直接刷新浏览器页面（所有响应都是 `no-cache`）；只有改端口需要重启应用。
- 浏览器控制台应出现 `JLiverTool Plugin API loaded` 与 `Connected to plugin server`，`jliverAPI.isConnected()` 应为 `true`。
- 不确定事件名和字段时，先 `jliverAPI.register('*', e => console.log(e))` 打一遍真实事件。
- 没有应用在跑时（例如直接以 `file://` 打开 HTML）`jliverAPI` 不存在：代码必须能降级（等待 API、显示「未连接」），模板里的 `?demo=1` 演示了这种做法。

## 交付给用户

插件已经在数据目录的 `plugins/` 下时，在插件管理里点「刷新」，列表里就会出现。

## 硬性规则与常见坑

- **入口文件必须叫** `index.html`：应用的打开按钮固定请求 `<folder>/index.html`；`meta.json` 的 `index` 只在加载时校验文件是否存在。
- **不要自己引入** `jliver-api.js`：服务器会把脚本内容注入到 `<head>` 之后（没有 `<head>` 则插到文件开头），再手动引一次会注册两遍、连两条 WebSocket。
- `jliverAPI` **可能不存在**：注入脚本先于插件脚本执行，正常由应用打开时它已经就绪；但页面被直接打开（`file://`）时没有它，用模板里的 `waitForApi()` 兜住这两种情况再 `register`。
- **频道名不区分大小写**，`'*'` 监听全部；`register()` 返回取消注册的函数。
- **事件不重放**：页面刷新后只会收到之后的事件；需要累积状态就自己存 `localStorage`（页面与应用同为 `http://127.0.0.1:<http_port>` 源）。
- **弹幕内容是不可信的用户输入**：一律用 `textContent` 渲染或显式转义，禁止直接 `innerHTML`，否则观众可以往页面里注入脚本。
- **字段可能是** `null`：例如没有粉丝勋章时 `medal_name`/`medal_level` 为 `null`，用前先判断。
- **高频事件要批量处理**：弹幕峰值每秒可达数十条，不要在回调里做强制重排或整块画布重绘；用 `requestAnimationFrame` 或定时聚合（参考 `plugins/wordcloud` 的定时更新、`plugins/constellation-map` 的动画循环）。
- **端口不要硬编码**：用 `location.origin` 和 URL 里的 `ws_port` 参数；端口可在设置里改（默认 8080/8081）。端口被占用时插件服务根本不会启动（日志里是 `Failed to start plugin WebSocket server`），此时换端口再重启。
- **资源用相对路径**：插件目录下的文件都通过 `/<folder>/...` 提供。
- 插件页面没有特殊权限，跨域请求按普通网页处理（受浏览器和目标服务器 CORS 限制）。

## 验收清单

- [ ] 「设置 → 插件管理」列表里能看到插件的名称/作者/描述/版本，说明 `meta.json` 合法且入口文件存在。
- [ ] 点「打开」后页面无控制台报错，`jliverAPI.isConnected() === true`。
- [ ] 真实弹幕/礼物/醒目留言能按预期渲染，或 `register('*')` 能看到对应事件。
- [ ] 重启应用后插件页能自动重连，密集弹幕下页面不卡顿。
- [ ] `meta.json` 的 `version` 已递增，插件目录名与首次发布保持一致。
