import type { ReactiveController, ReactiveControllerHost } from 'lit';

export interface ScrollSyncHost extends ReactiveControllerHost {
  renderRoot: HTMLElement | DocumentFragment;
}

export class ScrollSyncController implements ReactiveController {
  private observer: ResizeObserver | null = null;
  private headerEl: HTMLElement | null = null;
  private bodyEl: HTMLElement | null = null;

  constructor(private host: ScrollSyncHost) {
    host.addController(this);
  }

  hostConnected() {}

  hostDisconnected() {
    this.detach();
  }

  attach() {
    if (this.observer) return;   // already attached
    const root = this.host.renderRoot;
    this.headerEl = root.querySelector('.table-header') as HTMLElement | null;
    this.bodyEl = root.querySelector('.table-body') as HTMLElement | null;
    if (!this.headerEl || !this.bodyEl) return;

    this.bodyEl.addEventListener('scroll', this.onBodyScroll);

    if (typeof ResizeObserver !== 'undefined') {
      this.observer = new ResizeObserver(() => this.updateGutter());
      this.observer.observe(this.bodyEl);
    }
    this.updateGutter();
  }

  private detach() {
    this.bodyEl?.removeEventListener('scroll', this.onBodyScroll);
    this.observer?.disconnect();
    this.observer = null;
    this.headerEl = null;
    this.bodyEl = null;
  }

  private onBodyScroll = () => {
    if (this.headerEl && this.bodyEl) {
      this.headerEl.scrollLeft = this.bodyEl.scrollLeft;
    }
  };

  private updateGutter() {
    if (!this.headerEl || !this.bodyEl) return;
    const gutter = this.bodyEl.offsetWidth - this.bodyEl.clientWidth;
    this.headerEl.style.paddingRight = `${gutter}px`;
  }
}
