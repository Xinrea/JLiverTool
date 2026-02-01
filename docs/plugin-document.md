# 插件文档

插件样例：[测试插件](https://github.com/Xinrea/JLiverTool/tree/master/plugins/test-plugin)

## 插件开发

插件至少包含两个文件，`meta.json` 和 `index.html`；其中 `meta.json` 是插件的元数据，`index.html` 是插件的主页面，JLiverTool 将会创建一个独立窗口并加载 `index.html`，作为插件窗口展示。

### 插件目录结构

```
plugins/
  my-plugin/
    meta.json       # 插件元数据（必需）
    index.html      # 入口页面（必需）
    script.js       # 插件逻辑（可选）
    style.css       # 样式文件（可选）
```

插件目录位于应用数据目录下的 `plugins` 文件夹中：
- macOS: `~/Library/Application Support/com.jlivertool.JLiverTool/plugins/`
- Windows: `%APPDATA%/JLiverTool/plugins/`

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

JLiverTool 在加载 `index.html` 时会自动注入 `window.jliverAPI` 对象，插件可以通过这个对象访问 JLiverTool 提供的 API。

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
    type: "LiveStart",
    data: null
}
```

### LiveEnd - 直播结束

```javascript
{
    type: "LiveEnd",
    data: null
}
```

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

        // 等待 API 加载
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

1. **使用浏览器开发者工具**：插件窗口支持右键菜单打开开发者工具进行调试。

2. **检查连接状态**：使用 `jliverAPI.isConnected()` 检查是否已连接。

3. **监听所有事件**：使用 `jliverAPI.register('*', callback)` 监听所有事件，方便调试。

4. **查看控制台日志**：JLiverTool 会在控制台输出连接状态和错误信息。

## 注意事项

1. **等待 API 加载**：`jliverAPI` 对象是异步注入的，需要等待其可用后再使用。

2. **事件频道名称不区分大小写**：`NewDanmu`、`newdanmu`、`NEWDANMU` 都是有效的。

3. **自动重连**：插件会自动尝试重新连接，无需手动处理断线重连。

4. **资源路径**：插件中的资源文件（如图片、CSS、JS）使用相对路径即可。

5. **跨域限制**：插件运行在本地文件协议下，可能存在跨域限制，建议使用 `jliverAPI.util.openUrl()` 打开外部链接。
