import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('@layouts/DesktopLayout.vue'),
    children: [
      {
        path: '',
        name: 'dashboard',
        component: () => import('@views/dashboard/DashboardPage.vue'),
        meta: { title: '首页', icon: 'dashboard', keepAlive: true },
      },
      {
        path: 'projects',
        name: 'projects',
        component: () => import('@views/projects/ProjectsPage.vue'),
        meta: { title: '项目', icon: 'projects', keepAlive: true },
      },
      {
        path: 'projects/:projectId',
        name: 'project-detail',
        component: () => import('@views/projects/ProjectDetailPage.vue'),
        meta: { title: '项目详情', icon: 'projects', keepAlive: false },
      },
      {
        path: 'tunnels',
        name: 'tunnels',
        component: () => import('@views/tunnels/TunnelsPage.vue'),
        meta: { title: '隧道', icon: 'router', keepAlive: true },
      },
      {
        path: 'tunnels/http',
        name: 'http-tunnels',
        component: () => import('@views/tunnels/HttpTunnelPage.vue'),
        meta: { title: 'HTTP 隧道', icon: 'globe', keepAlive: true },
      },
      {
        path: 'tunnels/:tunnelId',
        name: 'tunnel-detail',
        component: () => import('@views/tunnels/TunnelsPage.vue'),
        meta: { title: '隧道详情', icon: 'router', keepAlive: true },
      },
      {
        path: 'servers',
        name: 'servers',
        component: () => import('@views/servers/ServersPage.vue'),
        meta: { title: '服务器', icon: 'servers', keepAlive: true },
      },
      {
        path: 'servers/:serverId',
        name: 'server-detail',
        component: () => import('@views/servers/ServersPage.vue'),
        meta: { title: '服务器详情', icon: 'servers', keepAlive: true },
      },
      {
        path: 'logs',
        name: 'logs',
        component: () => import('@views/logs/LogsPage.vue'),
        meta: { title: '日志', icon: 'logs', keepAlive: false },
      },
      {
        path: 'certificates',
        name: 'certificates',
        component: () => import('@views/certificates/CertificatePage.vue'),
        meta: { title: '证书', icon: 'shield-check', keepAlive: false },
      },
      {
        path: 'help',
        name: 'help',
        component: () => import('@views/help/HelpCenterPage.vue'),
        meta: { title: '帮助', icon: 'help', keepAlive: false },
      },
      {
        path: 'diagnostics',
        redirect: '/help',
      },
      {
        path: 'feedback',
        redirect: '/help',
      },
      {
        path: 'settings',
        name: 'settings',
        component: () => import('@views/settings/SettingsPage.vue'),
        meta: { title: '设置', icon: 'settings', keepAlive: true },
      },
      {
        path: 'about',
        name: 'about',
        component: () => import('@views/about/AboutPage.vue'),
        meta: { title: '关于', icon: 'about', keepAlive: false },
      },
    ],
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'not-found',
    component: () => import('@views/about/AboutPage.vue'),
    meta: { title: '404 未找到', icon: 'alert-circle' },
  },
]

export default routes
