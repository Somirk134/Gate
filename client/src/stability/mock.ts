import {
  ConnectionMonitorService,
  HealthService,
  HeartbeatService,
  ReconnectService,
  SessionRecoveryService,
} from './services'
import type {
  ConnectionHealth,
  HealthReport,
  HeartbeatSnapshot,
  ReconnectSnapshot,
  RecoveryResult,
} from './types'

/** Mock heartbeat scenarios for timeout and network jitter simulation. */
export class MockHeartbeat {
  readonly service = new HeartbeatService({
    intervalMs: 50,
    timeoutMs: 50,
    retryCount: 2,
    retryDelayMs: 10,
    maxMissedHeartbeat: 2,
  })

  /** Simulate heartbeat timeout. */
  async simulateTimeout(tunnelId: string): Promise<HeartbeatSnapshot> {
    await this.service.start(tunnelId)
    await this.service.ping(tunnelId)
    return this.service.timeout(tunnelId)
  }

  /** Simulate jitter where pong still arrives successfully. */
  async simulateNetworkJitter(tunnelId: string): Promise<HeartbeatSnapshot> {
    await this.service.start(tunnelId)
    const ping = await this.service.ping(tunnelId)
    return this.service.pong(tunnelId, ping.sequence)
  }
}

/** Mock reconnect scenarios for disconnection and recovery simulation. */
export class MockReconnect {
  readonly service = new ReconnectService({
    kind: 'fixed_interval',
    intervalMs: 1,
    maxAttempts: 5,
  })

  /** Simulate disconnection and schedule one reconnect attempt. */
  async simulateDisconnected(
    tunnelId: string,
    connectionId: string,
  ): Promise<ReconnectSnapshot | undefined> {
    await this.service.autoReconnect(tunnelId, connectionId, 'mock disconnected')
    await this.service.scheduleNext()
    return this.service.snapshot(tunnelId)
  }

  /** Simulate consecutive reconnect failures. */
  async simulateConsecutiveFailures(
    tunnelId: string,
    connectionId: string,
    failures: number,
  ): Promise<ReconnectSnapshot | undefined> {
    await this.service.autoReconnect(tunnelId, connectionId, 'mock failures')

    for (let index = 0; index < failures; index += 1) {
      await this.service.scheduleNext()
      await this.service.markFailed(tunnelId, `mock failure ${index}`)
      if (index + 1 < failures) {
        await this.service.autoReconnect(tunnelId, connectionId, 'retry mock failure')
      }
    }

    return this.service.snapshot(tunnelId)
  }

  /** Simulate successful reconnect. */
  async simulateSuccessfulRecovery(
    tunnelId: string,
    connectionId: string,
  ): Promise<ReconnectSnapshot | undefined> {
    await this.service.manualReconnect(tunnelId, connectionId, 'mock manual recovery')
    await this.service.scheduleNext()
    return this.service.markSucceeded(tunnelId)
  }
}

/** Mock recovery scenarios. */
export class MockRecovery {
  readonly service = new SessionRecoveryService()

  /** Simulate successful runtime metadata recovery. */
  async simulateRecovered(tunnelId: string): Promise<RecoveryResult | undefined> {
    await this.service.capture({
      tunnelId,
      sessionId: `session-${tunnelId}`,
      statistics: { heartbeatCount: 1 },
      subscriptions: [{ name: 'events', cursor: '0' }],
      attributes: { mock: 'true' },
      capturedAt: Date.now(),
    })
    return this.service.recover(tunnelId)
  }
}

/** Mock health scenarios. */
export class MockHealth {
  readonly service = new HealthService()

  /** Simulate offline health. */
  async simulateOffline(): Promise<HealthReport> {
    await this.service.checkConnection(false, 0)
    return this.service.report()
  }
}

/** Mock connection monitor scenarios. */
export class MockConnectionMonitor {
  readonly service = new ConnectionMonitorService()

  /** Simulate a connection loss. */
  async simulateConnectionLost(
    tunnelId: string,
    connectionId: string,
  ): Promise<ConnectionHealth | undefined> {
    await this.service.register(tunnelId, connectionId)
    return this.service.markLost(connectionId)
  }
}
