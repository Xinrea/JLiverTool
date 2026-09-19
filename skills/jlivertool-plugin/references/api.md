# 插件 API 参考

本文件是 JLiverTool 插件可用的全部接口。实现真相来源是仓库 `crates/jlivertool-plugin/src/` 下的 `jliver-api.js`、`events.rs`、`ws_server.rs`、`http_server.rs`、`plugin.rs`、`manager.rs`；发现不一致时以代码为准，并更新本文件。

## 插件目录与文件服务

```
plugins/
  my-plugin/
    meta.json      # 必需，元数据
    index.html     # 必需，入口页面（文件名固定为 index.html）
    plugin.js      # 可选，逻辑
    style.css      # 可选，样式
    lib/xxx.js     # 可选，第三方库（自己 vendored 进来）
```

- 应用启动时扫描数据目录下的 `plugins/`，遇到含 `meta.json` 的子目录就加载；加载失败只在日志里报错（`Failed to load plugin at ...`），不影响其他插件。
- HTTP 服务器（默认 `http://127.0.0.1:8080`）的路由：
  - `GET /jliver-api.js`：返回 API 脚本本体。
  - `GET /{插件目录名}/{文件路径}`：返回插件目录下的文件，MIME 由扩展名推断，所有响应带 `Cache-Control: no-cache`。
  - HTML 响应会把 `jliver-api.js` 的完整内容包成 `<script>` 插入到 `<head>` 之后（没有 `<head>` 就插到文件开头），所以插件**不要**再手动引入。
  - 文件路径会 canonicalize 后校验是否在插件目录内，越界返回 403（符号链接到目录外也属于越界）。
- 打开插件时应用访问的地址固定为：

  ```
  http://127.0.0.1:<plugin_http_port>/<插件目录名>/index.html?ws_port=<plugin_ws_port>
  ```

  默认端口：HTTP `8080`、WebSocket `8081`（配置项 `plugin_http_port` / `plugin_ws_port`，可在「设置 → 插件管理」修改，修改后需重启应用）。端口被占用时插件服务完全不会启动。

## meta.json

```json
{
  "id": "jlivertool.xinrea.wordcloud",
  "name": "弹幕词云",
  "author": "Xinrea",
  "desc": "生成弹幕词云",
  "version": "0.0.1",
  "index": "index.html",
  "url": "https://github.com/Xinrea"
}
```

| 字段 | 类型 | 必需 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 插件唯一标识，建议反向域名格式；同 id 会覆盖已加载的插件 |
| `name` | string | 是 | 显示名称 |
| `author` | string | 是 | 作者 |
| `desc` | string | 是 | 一句话描述 |
| `version` | string | 是 | 版本号，字符串（如 `1.0.0`） |
| `index` | string | 是 | 入口 HTML 文件名，加载时校验该文件存在 |
| `url` | string | 否 | 插件主页/仓库地址，缺省为 `null` |

## jliverAPI

`jliver-api.js` 被注入后提供 `window.jliverAPI`。它内部维护一条到应用 WebSocket 服务器的连接（地址取自 URL 的 `ws_port` 参数），并把事件分发给注册过的回调。

### 加载时机

注入的脚本位于 `<head>` 之后、插件自己的脚本之前，因此插件脚本运行时 `window.jliverAPI` 已经存在（前提是 URL 带 `ws_port`）。真正需要等待的是 WebSocket 连接本身：连接建立前 `isConnected()` 为 `false`。

页面不是由应用打开（例如 `file://`，没有 `ws_port`）时 `window.jliverAPI` 为 `undefined`，所以代码要能降级；轮询等待可以同时覆盖两种情况：

```javascript
function waitForApi(cb) {
  if (window.jliverAPI) cb();
  else setTimeout(() => waitForApi(cb), 100);
}
```

缺少 `ws_port` 参数时控制台输出 `JLiverTool: No WebSocket port provided`，`jliverAPI` 为 `undefined`；插件应在这种情况下显示「未连接」而不是报错。

### register(channel, callback) → unregister

监听事件。`channel` 大小写不敏感，`'*'` 表示全部事件；返回值为取消监听的函数。

```javascript
const off = jliverAPI.register('NewDanmu', (event) => {
  console.log(event.type, event.data.uname, event.data.msg);
});
off(); // 取消监听
```

回调参数形状固定为 `{ type: '事件类型', data: { ... } }`。回调抛出的异常会被 `jliver-api.js` 捕获并打印，不影响其他回调。

### util.openUrl(url) → Promise

用系统默认浏览器打开 URL，成功时 resolve `{ success: true }`。

```javascript
await jliverAPI.util.openUrl('https://live.bilibili.com/12345');
```

### util.getServerInfo() → Promise

返回 `{ version: '<应用版本>', name: 'JLiverTool Plugin Server' }`。

### isConnected() → boolean

WebSocket 是否处于 `OPEN` 状态。适合定时刷新页面上的连接指示（见模板里每秒轮询的写法）。

### reconnect()

重置退避时间并重新建连。一般不需要手动调用：断线后会自动重连（初始 200ms 指数退避，上限 3s）。

### 请求与超时

`util.*` 是基于请求/响应的 Promise 封装，**30 秒**没有响应会 reject `Request timeout`。连接未建立时请求只会打印 `JLiverTool: WebSocket not connected`，然后超时。

## 事件频道

| 频道（`register` 用的名称） | 触发时机 |
| --- | --- |
| `NewDanmu` | 收到弹幕 |
| `NewGift` | 收到礼物 |
| `NewGuard` | 有人上舰/续费舰长（含提督、总督） |
| `NewSuperChat` | 收到醒目留言 |
| `NewInteract` | 观众进入直播间、关注、分享 |
| `UpdateRoom` | 直播间信息变化（标题、开播状态等） |
| `UpdateOnline` | 在线人数/人气变化 |
| `LiveStart` | 开播 |
| `LiveEnd` | 下播 |
| `*` | 以上全部 |

事件只实时推送、不重放：插件打开或刷新页面之前发生的事件不会补发。弹幕、礼物、舰长、醒目留言、互动事件都不携带房间号，只适合单直播间场景；需要区分直播间时只能用 `UpdateRoom` 里的 `room_id` 维护当前房间状态。

### NewDanmu

```javascript
{
  type: 'NewDanmu',
  data: {
    uid: 475210,
    uname: 'Xinrea',
    msg: '弹幕内容',
    timestamp: 1784649531, // 秒级时间戳（应用收到消息的时间）
    medal_name: '轴芯',    // 无粉丝勋章时为 null
    medal_level: 42,       // 无粉丝勋章时为 null
    medal_room_id: 12345   // 无粉丝勋章时为 null
  }
}
```

### NewGift

```javascript
{
  type: 'NewGift',
  data: {
    uid: 475210,
    uname: 'Xinrea',
    gift_name: '小心心',
    num: 1,
    price: 0,              // 本次总价值（元，整数，按单价×数量/1000 向下取整）
    timestamp: 1784649531
  }
}
```

### NewGuard

```javascript
{ type: 'NewGuard', data: { uid, uname, guard_level: 3, num: 1, price: 198, timestamp } }
```

`guard_level`：`1` 总督、`2` 提督、`3` 舰长；`num` 为月数；`price` 为本次价格（元，整数）。

### NewSuperChat

```javascript
{ type: 'NewSuperChat', data: { uid, uname, message: '留言内容', price: 30, timestamp } }
```

### NewInteract

```javascript
{ type: 'NewInteract', data: { uid, uname, msg_type: 1, timestamp } }
```

`msg_type`：`1` 进入直播间、`2` 关注、`3` 分享。

### UpdateRoom

```javascript
{ type: 'UpdateRoom', data: { room_id: 12345, title: '直播间标题', live_status: 1 } }
```

`room_id` 是真实房间号（长号）；`live_status`：`0` 未开播、`1` 直播中、`2` 轮播中。

### UpdateOnline

```javascript
{ type: 'UpdateOnline', data: { count: 1234 } }
```

### LiveStart / LiveEnd

```javascript
{ type: 'LiveStart' } // 没有 data 字段，event.data 是 undefined
```

这两个事件不带任何数据（serde 相邻标记下的单元变体），判断时用 `if (event.data)` 而不是 `event.data === null`。

## 底层 WebSocket 协议（可选）

一般不需要直接用：`jliverAPI` 已经封装。自己实现客户端或排查连接问题时参考：

- 连接地址：`ws://127.0.0.1:<plugin_ws_port>`，无鉴权，只监听回环地址。
- 服务端 → 客户端：
  - `{ "type": "Welcome", "port": <number> }`：建连后立即发送，仅用于确认通道可用（`port` 字段来自连接的对端端口，不要当成服务端口）。
  - `{ "type": "Event", "Event": { "type": "NewDanmu", "data": { ... } } }`：事件推送；服务端默认给每个连接订阅 `*`。
  - `{ "type": "Response", "id": "<请求 id>", "data": { ... } }`：请求响应。
  - `{ "type": "Error", "id": "<请求 id> 或 null", "message": "..." }`：错误。
- 客户端 → 服务端：
  - `{ "type": "Subscribe", "channels": ["new_danmu"] }`
  - `{ "type": "Unsubscribe", "channels": ["new_danmu"] }`
  - `{ "type": "Request", "id": "1", "method": "openUrl", "params": { "url": "https://..." } }`
- 可用 `method` 只有 `openUrl` 和 `getServerInfo`（见 `ws_server.rs` 的 `handle_api_request`），其他值返回 `Unknown method: ...`。
- 事件缓冲上限 1000 条；消费过慢的连接会丢事件（日志 `Client ... lagged by N events`）。

## 排查对照表

| 现象 | 原因 |
| --- | --- |
| 插件列表里没有插件 | `meta.json` 缺失或 JSON 非法；`index` 指向的文件不存在；插件目录不在数据目录的 `plugins/` 下 |
| 点「打开」浏览器 404 | 入口文件不叫 `index.html`：应用固定请求 `<插件目录名>/index.html`，与 `meta.json` 里的 `index` 无关 |
| 页面 403 | 插件目录/文件是符号链接，指向插件目录之外 |
| `jliverAPI` is undefined | 少了 `ws_port` 参数（例如直接以 `file://` 打开），或看到的不是应用注入过的页面 |
| `isConnected()` 始终为 false | 应用没在运行，或插件服务未启动（端口被占用，应用日志 `Failed to start plugin WebSocket server`） |
| 收不到事件 | 回调注册太早（`jliverAPI` 还没就绪）、频道名拼错，或所有频道都被 `Unsubscribe` 掉了；先用 `register('*')` 验证 |
| `Request timeout` | 30 秒内没收到响应：检查应用是否在运行，或方法名不存在（只支持 `openUrl`、`getServerInfo`） |
