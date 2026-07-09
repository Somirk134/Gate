import type { Message, Response } from '../types'
import { ClientRequestManager } from '../request/ClientRequestManager'

export class ResponseDispatcher {
  constructor(private readonly requests: ClientRequestManager) {}

  dispatch<TBody>(message: Message<TBody>) {
    const response: Response<TBody> = {
      requestId: message.header.requestId,
      command: message.header.command,
      message,
      body: message.body,
      receivedAt: Date.now(),
      latencyMs: Math.max(0, Date.now() - message.header.timestamp),
    }

    return this.requests.resolve(response)
  }
}
