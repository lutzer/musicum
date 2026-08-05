import type { ReactiveController, ReactiveControllerHost } from 'lit';

const DEFAULT_MIN_WIDTH = 40;

export interface ResizeHost extends ReactiveControllerHost {
  widths: number[];
  columns: { minWidth?: number }[];
}

export class ColumnResizeController implements ReactiveController {
  constructor(private host: ResizeHost) {
    host.addController(this);
  }

  hostConnected() {}
  hostDisconnected() {}

  start = (event: PointerEvent, index: number) => {
    event.stopPropagation();
    event.preventDefault();
    const target = event.currentTarget as HTMLElement;
    const startX = event.clientX;
    const startWidth = this.host.widths[index] ?? 0;
    const minWidth = this.host.columns[index]?.minWidth ?? DEFAULT_MIN_WIDTH;
    target.setPointerCapture(event.pointerId);

    const onMove = (ev: PointerEvent) => {
      const next = Math.max(minWidth, startWidth + (ev.clientX - startX));
      const w = [...this.host.widths];
      w[index] = next;
      this.host.widths = w;
      this.host.requestUpdate();
    };
    const onEnd = (ev: PointerEvent) => {
      try { target.releasePointerCapture(ev.pointerId); } catch {}
      target.removeEventListener('pointermove', onMove);
      target.removeEventListener('pointerup', onEnd);
      target.removeEventListener('pointercancel', onEnd);
    };
    target.addEventListener('pointermove', onMove);
    target.addEventListener('pointerup', onEnd);
    target.addEventListener('pointercancel', onEnd);
  };
}
