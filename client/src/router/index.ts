import type { RouteRecordRaw } from "vue-router"


const routes: RouteRecordRaw[] = [
    {
        path: '/',
        component: () => import('@layouts/DesktopLayout.vue'),
        children: [
            {
                path: '',
                name: 'dashboard',
                component: () => import('@views/dashboard/DashboardPage.vue'),
                meta: { title: 'Dashboard', icon: 'dashboard', keepAlive: true },
            },
            {
                path: 'projects',
                name: 'projects',
                component: () => import('@views/projects/ProjectsPage.vue'),
                meta: { title: 'Projects', icon: 'projects', keepAlive: true },
            },
            {
                path: 'projects/:projectId',
                name: 'project-detail',
                component: () => import('@views/projects/ProjectDetailPage.vue'),
                meta: { title: 'Project Detail', icon: 'projects', keepAlive: false },
            },
            {
                path: 'tunnels',
                name: 'tunnels',
                component: () => import('@views/tunnels/TunnelsPage.vue'),
                meta: { title: 'Tunnels', icon: 'router', keepAlive: true },
            },
            {
                path: 'tunnels/http',
                name: 'http-tunnels',
                component: () => import('@views/tunnels/HttpTunnelPage.vue'),
                meta: { title: 'HTTP Tunnels', icon: 'globe', keepAlive: true },
            },
            {
                path: 'tunnels/:tunnelId',
                name: 'tunnel-detail',
                component: () => import('@views/tunnels/TunnelsPage.vue'),
                meta: { title: 'Tunnel Detail', icon: 'router', keepAlive: true },
            },
            {
                path: 'servers',
                name: 'servers',
                component: () => import('@views/servers/ServersPage.vue'),
                meta: { title: 'Servers', icon: 'servers', keepAlive: true },
            },
            {
                path: 'servers/:serverId',
                name: 'server-detail',
                component: () => import('@views/servers/ServersPage.vue'),
                meta: { title: 'Server Detail', icon: 'servers', keepAlive: true },
            },
            {
                path: 'logs',
                name: 'logs',
                component: () => import('@views/logs/LogsPage.vue'),
                meta: { title: 'Logs', icon: 'logs', keepAlive: false },
            },
            {
                path: 'diagnostics',
                name: 'diagnostics',
                component: () => import('@views/diagnostics/DiagnosticsPage.vue'),
                meta: { title: 'Diagnostics', icon: 'activity', keepAlive: false },
            },
            {
                path: 'feedback',
                name: 'feedback',
                component: () => import('@views/feedback/FeedbackPage.vue'),
                meta: { title: 'Feedback', icon: 'message', keepAlive: false },
            },
            {
                path: 'settings',
                name: 'settings',
                component: () => import('@views/settings/SettingsPage.vue'),
                meta: { title: 'Settings', icon: 'settings', keepAlive: true },
            },
            {
                path: 'about',
                name: 'about',
                component: () => import('@views/about/AboutPage.vue'),
                meta: { title: 'About', icon: 'about', keepAlive: false },
            },
        ],
    },
    {
        path: '/:pathMatch(.*)*',
        name: 'not-found',
        component: () => import('@views/about/AboutPage.vue'),
        meta: { title: '404 Not Found', icon: 'alert-circle' },
    },
]

export default routes
