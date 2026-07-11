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
        meta: { titleKey: 'nav.dashboard', icon: 'dashboard', keepAlive: true },
      },
      {
        path: 'projects',
        name: 'projects',
        component: () => import('@views/projects/ProjectsPage.vue'),
        meta: { titleKey: 'nav.projects', icon: 'projects', keepAlive: true },
      },
      {
        path: 'projects/:projectId',
        name: 'project-detail',
        component: () => import('@views/projects/ProjectDetailPage.vue'),
        meta: { titleKey: 'nav.projectDetail', icon: 'projects', keepAlive: false },
      },
      {
        path: 'tunnels',
        name: 'tunnels',
        component: () => import('@views/tunnels/TunnelsPage.vue'),
        meta: { titleKey: 'nav.tunnels', icon: 'router', keepAlive: true },
      },
      {
        path: 'tunnels/http',
        name: 'http-tunnels',
        component: () => import('@views/tunnels/HttpTunnelPage.vue'),
        meta: { titleKey: 'nav.httpTunnels', icon: 'globe', keepAlive: true },
      },
      {
        path: 'tunnels/:tunnelId',
        name: 'tunnel-detail',
        component: () => import('@views/tunnels/TunnelsPage.vue'),
        meta: { titleKey: 'nav.tunnelDetail', icon: 'router', keepAlive: true },
      },
      {
        path: 'servers',
        name: 'servers',
        component: () => import('@views/servers/ServersPage.vue'),
        meta: { titleKey: 'nav.servers', icon: 'servers', keepAlive: true },
      },
      {
        path: 'servers/:serverId',
        name: 'server-detail',
        component: () => import('@views/servers/ServersPage.vue'),
        meta: { titleKey: 'nav.serverDetail', icon: 'servers', keepAlive: true },
      },
      {
        path: 'logs',
        name: 'logs',
        component: () => import('@views/logs/LogsPage.vue'),
        meta: { titleKey: 'nav.logs', icon: 'logs', keepAlive: false },
      },
      {
        path: 'certificates',
        name: 'certificates',
        component: () => import('@views/certificates/CertificatePage.vue'),
        meta: { titleKey: 'nav.certificates', icon: 'shield-check', keepAlive: false },
      },
      {
        path: 'help',
        name: 'help',
        component: () => import('@views/help/HelpCenterPage.vue'),
        meta: { titleKey: 'nav.help', icon: 'help', keepAlive: false },
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
        meta: { titleKey: 'nav.settings', icon: 'settings', keepAlive: true },
      },
      {
        path: 'about',
        name: 'about',
        component: () => import('@views/about/AboutPage.vue'),
        meta: { titleKey: 'nav.about', icon: 'about', keepAlive: false },
      },
    ],
  },
  {
    path: '/:pathMatch(.*)*',
    name: 'not-found',
    redirect: '/',
  },
]

export default routes
