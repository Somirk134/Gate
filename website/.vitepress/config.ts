import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

const enSidebar = [
  {
    text: 'Start',
    items: [
      { text: 'Overview', link: '/' },
      { text: 'Quick Start', link: '/guide/quick-start' },
      { text: 'Install', link: '/guide/install' },
      { text: 'Configuration', link: '/guide/configuration' },
    ],
  },
  {
    text: 'Operate',
    items: [
      { text: 'Tunnel', link: '/guide/tunnel' },
      { text: 'Authentication', link: '/guide/authentication' },
      { text: 'Heartbeat', link: '/guide/heartbeat' },
      { text: 'Monitoring', link: '/guide/monitoring' },
      { text: 'Deployment', link: '/guide/deployment' },
      { text: 'Docker', link: '/guide/docker' },
      { text: 'Troubleshooting', link: '/guide/troubleshooting' },
    ],
  },
  {
    text: 'Develop',
    items: [
      { text: 'Architecture', link: '/guide/architecture' },
      { text: 'Development Guide', link: '/guide/development' },
      { text: 'Plugin Guide', link: '/guide/plugin-guide' },
      { text: 'API', link: '/guide/api' },
    ],
  },
]

const zhSidebar = [
  {
    text: '开始',
    items: [
      { text: '概览', link: '/zh/' },
      { text: '快速开始', link: '/zh/guide/quick-start' },
      { text: '安装', link: '/zh/guide/install' },
      { text: '配置', link: '/zh/guide/configuration' },
    ],
  },
  {
    text: '运维',
    items: [
      { text: '隧道', link: '/zh/guide/tunnel' },
      { text: '认证', link: '/zh/guide/authentication' },
      { text: '心跳', link: '/zh/guide/heartbeat' },
      { text: '监控', link: '/zh/guide/monitoring' },
      { text: '部署', link: '/zh/guide/deployment' },
      { text: 'Docker', link: '/zh/guide/docker' },
      { text: '故障排查', link: '/zh/guide/troubleshooting' },
    ],
  },
  {
    text: '开发',
    items: [
      { text: '架构', link: '/zh/guide/architecture' },
      { text: '开发指南', link: '/zh/guide/development' },
      { text: '插件指南', link: '/zh/guide/plugin-guide' },
      { text: 'API', link: '/zh/guide/api' },
    ],
  },
]

export default withMermaid(
  defineConfig({
    title: 'Gate',
    description: 'Enterprise-grade self-hosted tunnel runtime',
    cleanUrls: true,
    lastUpdated: true,
    appearance: 'dark',
    head: [
      ['link', { rel: 'icon', href: '/favicon.svg' }],
      ['meta', { property: 'og:title', content: 'Gate' }],
      ['meta', { property: 'og:description', content: 'Enterprise-grade self-hosted tunnel runtime' }],
      ['meta', { property: 'og:image', content: '/social-card.svg' }],
    ],
    markdown: {
      theme: {
        light: 'github-light',
        dark: 'github-dark',
      },
      lineNumbers: true,
    },
    themeConfig: {
      logo: '/logo.svg',
      search: {
        provider: 'local',
      },
      nav: [
        { text: 'Guide', link: '/guide/quick-start' },
        { text: 'Examples', link: '/guide/examples' },
        {
          text: 'Version',
          items: [
            { text: 'v0.1.x', link: '/guide/versions#v01x' },
            { text: 'v1 Roadmap', link: '/guide/versions#v1' },
            { text: 'v2 Roadmap', link: '/guide/versions#v2' },
          ],
        },
        { text: 'GitHub', link: 'https://github.com/lancemorii-git/gate' },
      ],
      sidebar: enSidebar,
      socialLinks: [{ icon: 'github', link: 'https://github.com/lancemorii-git/gate' }],
      footer: {
        message: 'Released under the MIT License.',
        copyright: 'Copyright © 2026 Gate Contributors',
      },
    },
    locales: {
      root: {
        label: 'English',
        lang: 'en-US',
        themeConfig: {
          nav: [
            { text: 'Guide', link: '/guide/quick-start' },
            { text: 'Examples', link: '/guide/examples' },
            {
              text: 'Version',
              items: [
                { text: 'v0.1.x', link: '/guide/versions#v01x' },
                { text: 'v1 Roadmap', link: '/guide/versions#v1' },
                { text: 'v2 Roadmap', link: '/guide/versions#v2' },
              ],
            },
            { text: 'GitHub', link: 'https://github.com/lancemorii-git/gate' },
          ],
          sidebar: enSidebar,
        },
      },
      zh: {
        label: '简体中文',
        lang: 'zh-CN',
        link: '/zh/',
        themeConfig: {
          nav: [
            { text: '指南', link: '/zh/guide/quick-start' },
            { text: '示例', link: '/zh/guide/examples' },
            {
              text: '版本',
              items: [
                { text: 'v0.1.x', link: '/zh/guide/versions#v01x' },
                { text: 'v1 路线图', link: '/zh/guide/versions#v1' },
                { text: 'v2 路线图', link: '/zh/guide/versions#v2' },
              ],
            },
            { text: 'GitHub', link: 'https://github.com/lancemorii-git/gate' },
          ],
          sidebar: zhSidebar,
          outline: {
            label: '页面导航',
          },
          docFooter: {
            prev: '上一篇',
            next: '下一篇',
          },
          darkModeSwitchLabel: '外观',
          lightModeSwitchTitle: '切换到浅色模式',
          darkModeSwitchTitle: '切换到深色模式',
          returnToTopLabel: '返回顶部',
          sidebarMenuLabel: '菜单',
        },
      },
    },
    mermaid: {
      theme: 'dark',
    },
  }),
)
