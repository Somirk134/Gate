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

export class NoopRequestClient implements RequestClient {
  async request<TData = unknown, TBody = unknown>(
    _options: RequestOptions<TBody>,
  ): Promise<RequestResponse<TData>> {
    throw new Error("Request client is not implemented in the application foundation layer.")
  }
}
