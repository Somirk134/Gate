import { createCommunicationId } from "../shared/id"
import type {
  Command,
  Message,
  Request,
  RequestId,
  RequestOptions,
  Response,
} from "../types"

interface PendingRequest {
  request: Request
  resolve: (response: Response) => void
  reject: (error: Error) => void
  timeout: ReturnType<typeof setTimeout>
}

const createMessage = <TBody>(
  messageType: "request",
  command: Command,
  body: TBody,
  requestId = createCommunicationId("req"),
): Message<TBody> => ({
  header: {
    protocolVersion: "v1",
    messageType,
    command,
    requestId,
    traceId: createCommunicationId("trace"),
    timestamp: Date.now(),
    sequence: 0,
  },
  body,
  metadata: {},
})

export class ClientRequestManager {
  private readonly pending = new Map<RequestId, PendingRequest>()

  createRequest<TBody>(
    command: Command,
    body: TBody,
    options: RequestOptions = {},
  ): Request<TBody> {
    const message = createMessage("request", command, body)

    return {
      id: message.header.requestId,
      command,
      message,
      timeoutMs: options.timeoutMs ?? 30_000,
      createdAt: Date.now(),
      retryPolicy: options.retryPolicy,
    }
  }

  register<TBody = unknown>(request: Request): Promise<Response<TBody>> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pending.delete(request.id)
        reject(new Error(`Request timed out: ${request.command}`))
      }, request.timeoutMs)

      this.pending.set(request.id, {
        request,
        resolve: (response) => resolve(response as Response<TBody>),
        reject,
        timeout,
      })
    })
  }

  resolve<TBody>(response: Response<TBody>) {
    const pending = this.pending.get(response.requestId)

    if (!pending) {
      return false
    }

    clearTimeout(pending.timeout)
    pending.resolve(response)
    this.pending.delete(response.requestId)
    return true
  }

  reject(requestId: RequestId, error: Error) {
    const pending = this.pending.get(requestId)

    if (!pending) {
      return false
    }

    clearTimeout(pending.timeout)
    pending.reject(error)
    this.pending.delete(requestId)
    return true
  }

  cancel(requestId: RequestId) {
    return this.reject(requestId, new Error(`Request canceled: ${requestId}`))
  }

  clear() {
    for (const pending of this.pending.values()) {
      clearTimeout(pending.timeout)
      pending.reject(new Error("Request manager cleared"))
    }

    this.pending.clear()
  }

  pendingCount() {
    return this.pending.size
  }
}
