import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'JLiverTool',
  description: 'B 站直播弹幕工具',
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: '主页', link: '/' },
      { text: '更新笔记', link: '/changelog' },
      { text: '使用文档', link: '/document' },
    ],

    sidebar: [
      {
        text: '开始使用',
        items: [
          { text: '下载和安装', link: '/setup' },
          { text: '更新笔记', link: '/changelog' },
        ],
      },
      {
        text: '使用文档',
        items: [
          { text: '功能说明', link: '/' },
          { text: '常见问题', link: '/' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/Xinrea/JLiverTool' },
    ],
  },
})
