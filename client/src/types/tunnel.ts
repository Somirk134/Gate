export interface TunnelDTO {
    id: string
    localPort: number
    remotePort: number
    protocol: 'tcp' | 'udp' | 'http' | 'https'
    status: 'active' | 'inactive' | 'closed'
    createdAt: string
}

export interface CreateTunnelDTO {
    localPort: number
    remotePort: number
    protocol: string
}
