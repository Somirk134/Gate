export function isValidPort(port: number): boolean {
    return Number.isInteger(port) && port > 0 && port <= 65535
}

export function isValidHostname(host: string): boolean {
    const pattern = /^([a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$/
    return pattern.test(host) || host === 'localhost'
}
