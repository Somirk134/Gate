import type { TimeoutConfig } from '../types'

export const defaultTimeoutConfig: TimeoutConfig = {
  requestTimeoutMs: 30_000,
  heartbeatTimeoutMs: 15_000,
  connectionTimeoutMs: 10_000,
  readTimeoutMs: 30_000,
  writeTimeoutMs: 30_000,
}

export const mergeTimeoutConfig = (timeoutConfig: Partial<TimeoutConfig> = {}): TimeoutConfig => ({
  ...defaultTimeoutConfig,
  ...timeoutConfig,
})
