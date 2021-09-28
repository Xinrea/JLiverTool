# JDanmaku

简单的弹幕姬，通过提取直播间弹幕栏显示而实现。核心代码其实就是注入运行的一段 js，如下所示：

```js
const historyPanel = document.querySelector('.chat-history-panel')
document.body.innerHTML = "<div id='panel' style='position:absolute;'></div>"
document.querySelector('#panel').appendChild(historyPanel)
```

同时注入 CSS 修改样式，实现暗色模式：

```css
.chat-history-panel {
  background-color: #1f1f1f !important;
}
.chat-history-panel .chat-history-list .chat-item.danmaku-item {
  color: #e8e8e8 !important;
}
body {
  background-color: #1f1f1f !important;
}
```
