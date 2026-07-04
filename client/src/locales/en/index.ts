import type { LocaleMessages } from 'vue-i18n'

const messages: LocaleMessages = {
    common: {
        appName: 'Gate',
        loading: 'Loading...',
        save: 'Save',
        cancel: 'Cancel',
        confirm: 'Confirm',
        delete: 'Delete',
        edit: 'Edit',
        create: 'Create',
        search: 'Search',
        reset: 'Reset',
    },
    nav: {
        dashboard: 'Dashboard',
        tunnels: 'Tunnels',
        connections: 'Connections',
        clients: 'Clients',
        settings: 'Settings',
        about: 'About',
    },
    tunnel: {
        create: 'Create Tunnel',
        edit: 'Edit Tunnel',
        delete: 'Delete Tunnel',
        status: 'Status',
        localPort: 'Local Port',
        remotePort: 'Remote Port',
        protocol: 'Protocol',
    },
}

export default messages
