import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'JLiverTool',
  description: 'B 站直播弹幕工具',
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: '主页', link: '/' },
      { text: '功能文档', link: '/user-document' },
      { text: '插件文档', link: '/plugin-document' },
      { text: '插件列表', link: '/plugin-list' },
    ],

    sidebar: [
      {
        text: '介绍',
        items: [
          { text: '开始使用', link: '/setup' },
          { text: '更新笔记', link: '/changelog' },
        ],
      },
      {
        text: '使用说明',
        items: [
          { text: '功能文档', link: '/user-document' },
          { text: '常见问题', link: '/qa' },
        ],
      },
      {
        text: '插件开发',
        items: [
          { text: '插件文档', link: '/plugin-document' },
          { text: '插件列表', link: '/plugin-list' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/Xinrea/JLiverTool' },
    ],
  },
})
