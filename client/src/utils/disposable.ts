export interface Disposable {
  dispose(): void | Promise<void>
}

export type DisposeCallback = () => void | Promise<void>

export function toDisposable(callback: DisposeCallback): Disposable {
  return {
    dispose: callback,
  }
}

export class DisposableStore implements Disposable {
  private readonly disposables = new Set<Disposable>()

  add<T extends Disposable>(disposable: T): T {
    this.disposables.add(disposable)
    return disposable
  }

  addCallback(callback: DisposeCallback): Disposable {
    return this.add(toDisposable(callback))
  }

  async dispose() {
    const pending = Array.from(this.disposables)
    this.disposables.clear()

    for (const disposable of pending) {
      await disposable.dispose()
    }
  }
}
