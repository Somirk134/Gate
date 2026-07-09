export interface ConnectionDTO {
  id: string
  clientId: string
  remoteAddr: string
  connectedAt: string
  bytesReceived: number
  bytesSent: number
}
