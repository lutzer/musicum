import type { ReactiveController, ReactiveControllerHost } from 'lit';

interface Emitter { subscribe(fn: () => void): () => void }

export class RegistrySubscription implements ReactiveController {
  private unsub?: () => void;
  constructor(private host: ReactiveControllerHost, private emitter: Emitter) {
    host.addController(this);
  }
  hostConnected() {
    this.unsub = this.emitter.subscribe(() => this.host.requestUpdate());
  }
  hostDisconnected() {
    this.unsub?.();
  }
}
