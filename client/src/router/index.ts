import type { RouteRecordRaw } from "vue-router"

const routes: RouteRecordRaw[] = [
    {
        path: "/",
        component: () => import("@layouts/DefaultLayout.vue"),
        children: [
            {
                path: "",
                name: "dashboard",
                component: () => import("@views/dashboard/DashboardPage.vue"),
                meta: { title: "Dashboard", icon: "dashboard" },
            },
            {
                path: "projects",
                name: "projects",
                component: () => import("@views/projects/ProjectsPage.vue"),
                meta: { title: "Projects", icon: "projects" },
            },
            {
                path: "projects/:projectId",
                name: "project-detail",
                component: () => import("@views/projects/ProjectDetailPage.vue"),
                meta: { title: "Project Detail", icon: "projects" },
            },
            {
                path: "servers",
                name: "servers",
                component: () => import("@views/servers/ServersPage.vue"),
                meta: { title: "Servers", icon: "servers" },
            },
            {
                path: "servers/:serverId",
                name: "server-detail",
                component: () => import("@views/servers/ServerDetailPage.vue"),
                meta: { title: "Server Detail", icon: "servers" },
            },
            {
                path: "logs",
                name: "logs",
                component: () => import("@views/logs/LogsPage.vue"),
                meta: { title: "Logs", icon: "logs" },
            },
            {
                path: "settings",
                name: "settings",
                component: () => import("@views/settings/SettingsPage.vue"),
                meta: { title: "Settings", icon: "settings" },
            },
            {
                path: "about",
                name: "about",
                component: () => import("@views/about/AboutPage.vue"),
                meta: { title: "About", icon: "about" },
            },
        ],
    },
]

export default routes
