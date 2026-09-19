# 插件文档

插件样例：[测试插件](https://github.com/Xinrea/JLiverTool/tree/master/plugins/test-plugin)

## 使用 Agent Skill 开发插件

仓库内置了一份 Agent Skill：[`skills/jlivertool-plugin`](https://github.com/Xinrea/JLiverTool/tree/master/skills/jlivertool-plugin)。它把本页的插件规范（`meta.json`、`jliverAPI`、事件字段）连同本地调试、发布、验收的完整流程整理成 Agent 可以直接执行的步骤，并附带一个最小插件模板。

一键安装（需要 Node.js，在任意目录执行一次即可对所有项目生效）：

```bash
npx skills add Xinrea/JLiverTool --skill jlivertool-plugin -g -y
```

装好后直接向 Agent 描述需求即可，例如「帮我写一个统计弹幕关键词并实时显示词云的 JLiverTool 插件」；Agent 会按本页规范生成 `meta.json` 和 `index.html`，并告诉你如何放进插件目录验证。

其他常用命令：

| 目的 | 命令 |
| --- | --- |
| 先看看仓库里有哪些 skill | `npx skills add Xinrea/JLiverTool --list` |
| 只装到当前目录的 Agent 目录，不装到全局 | `npx skills add Xinrea/JLiverTool --skill jlivertool-plugin` |
| 不安装，只让指定 Agent 用一次 | `npx skills use Xinrea/JLiverTool@jlivertool-plugin --agent claude-code` |
| 更新 / 卸载 | `npx skills update jlivertool-plugin` / `npx skills remove jlivertool-plugin -g` |

`skills` 是通用的 Agent Skill 管理工具，会自动识别本机已安装的编码 Agent（Claude Code、Codex、Cursor 等），把 skill 安装到对应目录（如 `~/.claude/skills/`）。不想用工具时，直接把 `skills/jlivertool-plugin` 目录复制到你的 Agent 的 skills 目录同样可以使用。

> Skill 内容随应用版本更新；如果它和本页或源码不一致，以 `crates/jlivertool-plugin/` 下的实现为准。

## 安装插件

插件是纯静态网页，安装就是把插件目录放进应用数据目录的 `plugins/` 下，两种方式都可以：

1. **从 GitHub 导入**：在「设置 -> 插件管理 -> 从 GitHub 导入插件」中粘贴插件目录的链接（形如 `https://github.com/Xinrea/JLiverTool/tree/master/plugins/wordcloud`）后点「导入」。应用会递归下载该目录下的所有文件，保存为 `plugins/<链接最后一段>/`；重复导入会覆盖同名目录，所以插件更新时重新导入同一链接即可。
2. **手动安装**：把插件目录（其中必须包含 `meta.json`）复制到 `plugins/` 下，再点「刷新」。

安装成功后列表会显示插件名称、作者、版本与描述，点「打开」即在系统默认浏览器中使用，点「删除」会同时删除磁盘上的插件目录。如果插件没有出现在列表里，检查 `meta.json` 是否为合法 JSON、`index` 指向的入口文件是否存在。

## 插件开发

插件至少包含两个文件，`meta.json` 和 `index.html`；其中 `meta.json` 是插件的元数据，`index.html` 是插件的主页面。点击打开插件时，JLiverTool 会在系统默认浏览器中打开插件页面。

### 插件运行方式

JLiverTool 内置了一个 HTTP 服务器来提供插件文件服务。当你打开一个插件时：

1. 插件页面通过 `http://127.0.0.1:{http_port}/{plugin_folder}/index.html?ws_port={ws_port}` 在浏览器中打开
2. HTTP 服务器会自动在 HTML 文件中注入 `jliver-api.js` 脚本，无需手动引入
3. `jliver-api.js` 会自动从 URL 参数中读取 `ws_port` 并连接到 WebSocket 服务器

**默认端口：**
- HTTP 服务端口：8080
- WebSocket 服务端口：8081

端口可在设置 -> 插件管理中修改，修改后需重启应用生效。

### 插件目录结构

```
plugins/
  my-plugin/
    meta.json       # 插件元数据（必需）
    index.html      # 入口页面（必需）
    script.js       # 插件逻辑（可选）
    style.css       # 样式文件（可选）
```

插件目录位于应用数据目录下的 `plugins` 文件夹中，在「设置 -> 插件管理 -> 插件说明」里点「打开插件目录」可以直接打开该目录：

- macOS: `~/Library/Application Support/com.jlivertool.JLiverTool/plugins/`
- Windows: `%APPDATA%\jlivertool\JLiverTool\config\plugins\`
- Linux: `~/.config/jlivertool/plugins/`

### meta.json

`meta.json` 是插件的元数据文件，内容如下所示：

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
|------|------|------|------|
| id | string | 是 | 插件唯一标识符，建议使用反向域名格式 |
| name | string | 是 | 插件显示名称 |
| author | string | 是 | 插件作者 |
| desc | string | 是 | 插件描述 |
| version | string | 是 | 插件版本号 |
| index | string | 是 | 入口 HTML 文件名 |
| url | string | 否 | 插件主页或仓库地址 |

## jliverAPI

JLiverTool 会自动在插件的 HTML 页面中注入 `jliver-api.js` 脚本，该脚本提供 `window.jliverAPI` 对象供插件使用。

**注意：** 脚本是自动注入的，无需在 HTML 中手动引入。

### API 概览

```javascript
window.jliverAPI = {
    // 注册事件监听器
    register: function(channel, callback) { ... },

    // 工具方法
    util: {
        openUrl: function(url) { ... },      // 打开 URL
        getServerInfo: function() { ... }    // 获取服务器信息
    },

    // 连接状态
    isConnected: function() { ... },

    // 手动重连
    reconnect: function() { ... }
};
```

### register(channel, callback)

注册事件监听器，监听 JLiverTool 发送的事件。

**参数：**
- `channel` (string): 事件频道名称，不区分大小写
- `callback` (function): 事件回调函数，接收事件对象作为参数

**返回值：**
- 返回一个取消注册的函数，调用后可以取消监听

**示例：**
```javascript
// 监听新弹幕事件
const unregister = jliverAPI.register('NewDanmu', (event) => {
    console.log('收到弹幕:', event.data.uname, event.data.msg);
});

// 监听所有事件
jliverAPI.register('*', (event) => {
    console.log('收到事件:', event.type, event.data);
});

// 取消监听
unregister();
```

### util.openUrl(url)

在系统默认浏览器中打开指定 URL。

**参数：**
- `url` (string): 要打开的 URL

**返回值：**
- Promise，成功时返回 `{success: true}`

**示例：**
```javascript
await jliverAPI.util.openUrl('https://github.com/Xinrea/JLiverTool');
```

### util.getServerInfo()

获取插件服务器信息。

**返回值：**
- Promise，返回服务器信息对象

**示例：**
```javascript
const info = await jliverAPI.util.getServerInfo();
console.log('服务器版本:', info.version);
```

### isConnected()

检查与 JLiverTool 的连接状态。

**返回值：**
- `true` 如果已连接，`false` 如果未连接

**示例：**
```javascript
if (jliverAPI.isConnected()) {
    console.log('已连接到 JLiverTool');
}
```

### reconnect()

手动重新连接到 JLiverTool。

**示例：**
```javascript
jliverAPI.reconnect();
```

## 事件列表

以下是可监听的事件频道：

| 频道名称 | 说明 |
|----------|------|
| `NewDanmu` | 新弹幕 |
| `NewGift` | 新礼物 |
| `NewGuard` | 新舰长/提督/总督 |
| `NewSuperChat` | 新醒目留言 |
| `NewInteract` | 新互动（进入、关注等） |
| `UpdateRoom` | 直播间信息更新 |
| `UpdateOnline` | 在线人数更新 |
| `LiveStart` | 直播开始 |
| `LiveEnd` | 直播结束 |
| `*` | 所有事件 |

## 事件数据格式

所有事件回调接收的参数格式为：

```javascript
{
    type: "事件类型",  // 如 "NewDanmu"
    data: { ... }      // 事件数据
}
```

### NewDanmu - 新弹幕

```javascript
{
    type: "NewDanmu",
    data: {
        uid: 12345,              // 用户 UID
        uname: "用户名",          // 用户名
        msg: "弹幕内容",          // 弹幕文本
        timestamp: 1234567890,   // 时间戳（秒）
        medal_name: "勋章名",     // 粉丝勋章名称（可选）
        medal_level: 20,         // 粉丝勋章等级（可选）
        medal_room_id: 12345     // 粉丝勋章对应房间号（可选）
    }
}
```

### NewGift - 新礼物

```javascript
{
    type: "NewGift",
    data: {
        uid: 12345,              // 用户 UID
        uname: "用户名",          // 用户名
        gift_name: "礼物名称",    // 礼物名称
        num: 1,                  // 礼物数量
        price: 100,              // 总价值（元）
        timestamp: 1234567890    // 时间戳（秒）
    }
}
```

### NewGuard - 新舰长

```javascript
{
    type: "NewGuard",
    data: {
        uid: 12345,              // 用户 UID
        uname: "用户名",          // 用户名
        guard_level: 3,          // 舰长等级：1=总督, 2=提督, 3=舰长
        num: 1,                  // 数量（月数）
        price: 198,              // 价格（元）
        timestamp: 1234567890    // 时间戳（秒）
    }
}
```

### NewSuperChat - 新醒目留言

```javascript
{
    type: "NewSuperChat",
    data: {
        uid: 12345,              // 用户 UID
        uname: "用户名",          // 用户名
        message: "留言内容",      // SC 内容
        price: 30,               // 价格（元）
        timestamp: 1234567890    // 时间戳（秒）
    }
}
```

### NewInteract - 新互动

```javascript
{
    type: "NewInteract",
    data: {
        uid: 12345,              // 用户 UID
        uname: "用户名",          // 用户名
        msg_type: 1,             // 互动类型：1=进入, 2=关注, 3=分享
        timestamp: 1234567890    // 时间戳（秒）
    }
}
```

### UpdateRoom - 直播间信息更新

```javascript
{
    type: "UpdateRoom",
    data: {
        room_id: 12345,          // 房间号
        title: "直播间标题",      // 直播间标题
        live_status: 1           // 直播状态：0=未开播, 1=直播中, 2=轮播中
    }
}
```

### UpdateOnline - 在线人数更新

```javascript
{
    type: "UpdateOnline",
    data: {
        count: 1234              // 在线人数
    }
}
```

### LiveStart - 直播开始

```javascript
{
    type: "LiveStart"
}
```

### LiveEnd - 直播结束

```javascript
{
    type: "LiveEnd"
}
```

`LiveStart` 和 `LiveEnd` 不带任何数据，事件里没有 `data` 字段（`event.data` 为 `undefined`）。

## 完整示例

以下是一个完整的插件示例，展示如何监听事件并更新 UI：

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <title>弹幕统计插件</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
            background: #1a1a2e;
            color: #eee;
            padding: 16px;
        }
        .stat { font-size: 24px; margin: 8px 0; }
        .danmu-list { max-height: 300px; overflow-y: auto; }
        .danmu-item { padding: 4px 8px; margin: 2px 0; background: #16213e; border-radius: 4px; }
    </style>
</head>
<body>
    <h1>弹幕统计</h1>
    <div class="stat">弹幕数: <span id="danmu-count">0</span></div>
    <div class="stat">礼物数: <span id="gift-count">0</span></div>
    <h2>最近弹幕</h2>
    <div class="danmu-list" id="danmu-list"></div>

    <script>
        let danmuCount = 0;
        let giftCount = 0;

        // 等待 API 加载（jliver-api.js 会自动注入）
        function waitForApi(callback) {
            if (window.jliverAPI) {
                callback();
            } else {
                setTimeout(() => waitForApi(callback), 100);
            }
        }

        waitForApi(function() {
            // 监听新弹幕
            jliverAPI.register('NewDanmu', (event) => {
                danmuCount++;
                document.getElementById('danmu-count').textContent = danmuCount;

                // 添加到列表
                const list = document.getElementById('danmu-list');
                const item = document.createElement('div');
                item.className = 'danmu-item';
                item.textContent = `[${event.data.uname}] ${event.data.msg}`;
                list.insertBefore(item, list.firstChild);

                // 保持最多 50 条
                while (list.children.length > 50) {
                    list.removeChild(list.lastChild);
                }
            });

            // 监听新礼物
            jliverAPI.register('NewGift', (event) => {
                giftCount++;
                document.getElementById('gift-count').textContent = giftCount;
            });

            console.log('插件已加载');
        });
    </script>
</body>
</html>
```

## 调试技巧

1. **使用浏览器开发者工具**：插件在浏览器中运行，可以直接使用浏览器的开发者工具（F12）进行调试。

2. **检查连接状态**：使用 `jliverAPI.isConnected()` 检查是否已连接到 WebSocket 服务器。

3. **监听所有事件**：使用 `jliverAPI.register('*', callback)` 监听所有事件，方便调试。

4. **查看控制台日志**：`jliver-api.js` 会在控制台输出连接状态和错误信息。

5. **检查 URL 参数**：确保 URL 中包含正确的 `ws_port` 参数，例如 `?ws_port=8081`。

## 注意事项

1. **自动注入脚本**：`jliver-api.js` 会自动注入到 HTML 文件中，无需手动引入。

2. **API 可能不存在**：注入脚本在 `<head>` 之后、插件自己的脚本之前执行，正常情况下 `window.jliverAPI` 已经可用；但如果页面不是由 JLiverTool 打开的（例如直接用 `file://` 打开），它就不存在，代码需要做降级处理（示例里的 `waitForApi` 兼容了这两种情况）。

3. **事件频道名称不区分大小写**：`NewDanmu`、`newdanmu`、`NEWDANMU` 都是有效的。

4. **自动重连**：插件会自动尝试重新连接，无需手动处理断线重连。

5. **资源路径**：插件中的资源文件（如图片、CSS、JS）使用相对路径即可，会相对于插件目录解析。

6. **端口配置**：默认 HTTP 端口为 8080，WebSocket 端口为 8081。可在设置 -> 插件管理中修改。
