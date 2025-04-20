# 插件文档

插件样例：[弹幕词云](https://github.com/Xinrea/JLiverTool/tree/master/plugins/wordcloud)

## 插件开发

插件至少包含两个文件，`meta.json` 和 `index.html`；其中 `meta.json` 是插件的元数据，`index.html` 是插件的主页面，JLiverTool 将会创建一个独立窗口并加载 `index.html`，作为插件窗口展示。

`meta.json` 的内容如下所示：

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

你需要以 `index.html` 为主入口实现插件的功能；JLiverTool 提供一系列 API 用于插件的开发，定义在 [plugin_preload.ts](https://github.com/Xinrea/JLiverTool/blob/master/src/plugin_preload.ts)。JLiverTool 在加载 `index.html` 时会自动注入 `window.jliverAPI` 对象，插件可以通过这个对象访问 JLiverTool 提供的 API。

jliverAPI 对象的类型定义如下：

```typescript
export type JLiverAPI = {
  register: (channel: JEvent, callback: Function) => void
  user: {
    info: (user_id: number) => Promise<UserInfoResponse>
  }
  room: {
    info: (room_id: number) => Promise<GetInfoResponse>
  }
  util: {
    openUrl: (url: string) => Promise<any>
    fonts: () => Promise<any>
    setClipboard: (text: string) => Promise<any>
  }
}
```

你可以使用 `user.info` 和 `room.info` 方法获取指定用户和直播间的信息；`util` 下提供了一些实用的工具方法。

最主要的是 `register` 方法，它用于注册事件监听器，监听 JLiverTool 发送的事件；你可以在插件中使用 `jliverAPI.register` 方法注册事件监听器，监听 JLiverTool 发送的事件。可监听的的事件列表定义在 [event.ts](https://github.com/Xinrea/JLiverTool/blob/master/src/lib/events.ts) 中，插件能够获取的事件主要是弹幕、礼物和醒目留言事件等。
