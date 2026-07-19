import { defineConfig } from '@clarify-labs/cli'

export default defineConfig({
  routePrefix: '/Quick-SSH/',
  title: 'Quick-SSH Docs',
  description: '跨平台 SSH 连接管理工具 — 完整文档',
  logo: '/logo.svg',
  favicon: '/favicon.svg',
  theme: {
    preset: 'default',
    tokens: {
      colors: {
        primary: '#0EA5E9',
        accent: '#06B6D4',
      },
    },
  },
  navigation: {
    links: [
      { label: '指南', href: '/getting-started' },
      { label: '开发', href: '/architecture' },
      { label: 'GitHub', href: 'https://github.com/CCE-Li/Quick-SSH', external: true },
    ],
    tabs: [
      {
        tab: '文档',
        icon: 'BookOpen',
        pages: [
          {
            group: '快速开始',
            icon: 'Rocket',
            pages: [
              { page: 'index', title: '概述', icon: 'Sparkles' },
              { page: 'installation', title: '安装指南', icon: 'Download' },
              { page: 'getting-started', title: '快速入门', icon: 'Play' },
            ],
          },
          {
            group: '用户指南',
            icon: 'BookOpen',
            pages: [
              { page: 'cli-reference', title: 'CLI 命令参考', icon: 'Terminal' },
              { page: 'tui-guide', title: 'TUI 界面指南', icon: 'LayoutPanelLeft' },
              { page: 'configuration', title: '配置说明', icon: 'Settings2' },
              { page: 'file-upload', title: '文件上传', icon: 'Upload' },
              { page: 'shell-completions', title: 'Shell 补全', icon: 'Code' },
            ],
          },
          {
            group: '开发文档',
            icon: 'Wrench',
            pages: [
              { page: 'architecture', title: '架构设计', icon: 'Layers' },
              { page: 'config-module', title: '配置模块', icon: 'FileJson' },
              { page: 'ssh-module', title: 'SSH 连接模块', icon: 'Network' },
              { page: 'tui-module', title: 'TUI 模块', icon: 'LayoutPanelLeft' },
              { page: 'network-module', title: '网络工具模块', icon: 'Signal' },
              { page: 'uploader', title: '上传器', icon: 'Upload' },
            ],
          },
          {
            group: '打包与发布',
            icon: 'Package',
            pages: [
              { page: 'overview', title: '打包总览', icon: 'Package' },
              { page: 'release-process', title: '发布流程', icon: 'Rocket' },
              { page: 'package-managers', title: '包管理器参考', icon: 'Blocks' },
            ],
          },
          {
            group: '参考',
            icon: 'Library',
            pages: [
              { page: 'api', title: 'API 参考', icon: 'FileCode' },
              { page: 'roadmap', title: '路线图', icon: 'Map' },
              { page: 'changelog', title: '变更日志', icon: 'History' },
            ],
          },
        ],
      },
    ],
  },
  footer: {
    copyright: '© 2025 CCE-Li. Built with Clarify.',
    links: [
      { label: '快速入门', href: '/getting-started' },
      { label: '安装指南', href: '/installation' },
      { label: 'CLI 参考', href: '/cli-reference' },
      { label: 'GitHub', href: 'https://github.com/CCE-Li/Quick-SSH' },
    ],
    socials: {
      GitHub: 'https://github.com/CCE-Li/Quick-SSH',
    },
  },
})
