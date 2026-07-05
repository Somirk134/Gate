import type { RetryPolicyConfig } from "../types"

export class RetryPolicy {
  constructor(private readonly config: RetryPolicyConfig = RetryPolicy.defaultConfig()) {}

  static defaultConfig(): RetryPolicyConfig {
    return {
      kind: "exponential",
      baseDelayMs: 500,
      maxDelayMs: 30_000,
      factor: 2,
      maxAttempts: 5,
    }
  }

  delayForAttempt(attempt: number): number | undefined {
    if (attempt <= 0) {
      return undefined
    }

    switch (this.config.kind) {
      case "none":
        return undefined
      case "linear":
        if (attempt > this.config.maxAttempts) {
          return undefined
        }

        return Math.min(this.config.initialDelayMs, this.config.maxDelayMs)
      case "exponential":
        if (attempt > this.config.maxAttempts) {
          return undefined
        }

        return Math.min(
          this.config.baseDelayMs * this.config.factor ** (attempt - 1),
          this.config.maxDelayMs,
        )
      case "custom":
        return this.config.delaysMs[attempt - 1]
    }
  }

  shouldRetry(attempt: number) {
    return this.delayForAttempt(attempt) !== undefined
  }
}
