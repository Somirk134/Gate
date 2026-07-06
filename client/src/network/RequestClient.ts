export type RequestMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE"

export interface RequestOptions<TBody = unknown> {
  method: RequestMethod
  url: string
  headers?: Record<string, string>
  body?: TBody
  signal?: AbortSignal
  timeout?: number
}

export interface RequestResponse<TData = unknown> {
  status: number
  data: TData
  headers: Record<string, string>
}

export interface RequestClient {
  request<TData = unknown, TBody = unknown>(
    options: RequestOptions<TBody>,
  ): Promise<RequestResponse<TData>>
}

export class FetchRequestClient implements RequestClient {
  async request<TData = unknown, TBody = unknown>(
    options: RequestOptions<TBody>,
  ): Promise<RequestResponse<TData>> {
    const controller = new AbortController()
    const timeout = options.timeout
      ? window.setTimeout(() => controller.abort(), options.timeout)
      : undefined

    try {
      const response = await fetch(options.url, {
        method: options.method,
        headers: {
          "content-type": "application/json",
          ...options.headers,
        },
        body: options.body === undefined ? undefined : JSON.stringify(options.body),
        signal: options.signal ?? controller.signal,
      })
      const contentType = response.headers.get("content-type") ?? ""
      const data = contentType.includes("application/json")
        ? ((await response.json()) as TData)
        : ((await response.text()) as TData)
      const headers: Record<string, string> = {}
      response.headers.forEach((value, key) => {
        headers[key] = value
      })

      return {
        status: response.status,
        data,
        headers,
      }
    } finally {
      if (timeout) {
        window.clearTimeout(timeout)
      }
    }
  }
}
