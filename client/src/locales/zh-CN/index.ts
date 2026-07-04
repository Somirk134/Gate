import type { LocaleMessages } from 'vue-i18n'

const messages: LocaleMessages = {
    common: {
        appName: 'Gate',
        loading: '加载中...',
        save: '保存',
        cancel: '取消',
        confirm: '确认',
        delete: '删除',
        edit: '编辑',
        create: '新建',
        search: '搜索',
        reset: '重置',
    },
    nav: {
        dashboard: '仪表盘',
        tunnels: '隧道',
        connections: '连接',
        clients: '客户端',
        settings: '设置',
        about: '关于',
    },
    tunnel: {
        create: '创建隧道',
        edit: '编辑隧道',
        delete: '删除隧道',
        status: '状态',
        localPort: '本地端口',
        remotePort: '远程端口',
        protocol: '协议',
    },
}

export default messages
