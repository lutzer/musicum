import { LitElement, html, css, type PropertyValues } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { ColumnResizeController } from './mus-list-view-resize';
import { ScrollSyncController } from './mus-list-view-scroll-sync';

export type ListState<T> = 'loading' | T[] | { error: string };

export interface ListColumn<T> {
  key: string;
  label: string;
  align?: 'left' | 'right';
  width?: number;
  minWidth?: number;
  sortValue?: (item: T) => string | number | null;
  render: (item: T) => unknown;
}

const DEFAULT_WIDTH = 120;
const DEFAULT_MIN_WIDTH = 40;

@customElement('mus-list-view')
export class MusListView<T = unknown> extends LitElement {
  static styles = css`
    :host {
      display: flex;
      background: var(--mus-surface);
      width: 100%;
      min-height: 0;
      flex-grow: 1;
    }
    .table-container {
      display: flex;
      flex-direction: column;
      width: 100%;
      min-height: 0;
    }
    .table-header {
      overflow-x: hidden;
      flex-shrink: 0;
    }
    .table-body {
      overflow: auto;
      min-height: 0;
      flex-grow: 1;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      table-layout: fixed;
      font-size: var(--mus-font-md);
    }
    th, td {
      text-align: left;
      padding: var(--mus-space-xs) var(--mus-space-sm);
    }
    td {
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
    th {
      font-weight: 600;
      user-select: none;
      background: var(--mus-accent-bg);
      position: relative;
    }
    th[data-sortable] {
      cursor: pointer;
    }
    th[data-align='right'], td[data-align='right'] {
      text-align: right;
      font-variant-numeric: tabular-nums;
    }
    .sort-indicator {
      margin-left: var(--mus-space-xs);
      opacity: 0.7;
    }
    .resize-handle {
      position: absolute;
      top: 0;
      right: 0;
      width: 6px;
      height: 100%;
      cursor: col-resize;
      user-select: none;
      touch-action: none;
    }
    :host(.drag-over) {
      outline: 2px dashed var(--mus-accent);
      outline-offset: -4px;
    }
    .state {
      position: absolute;
      padding: var(--mus-space-md);
    }
  `;

  @property({ attribute: false }) items: ListState<T> = 'loading';
  @property({ attribute: false }) columns: ListColumn<T>[] = [];
  @property() emptyMessage = 'No items.';
  @property({ type: Boolean, attribute: 'accept-drop' }) acceptDrop = false;

  @state() private sortKey: string | null = null;
  @state() private sortDirection: 'asc' | 'desc' = 'asc';
  @state() private isDragOver = false;
  @state() widths: number[] = [];

  private tauriUnlisten: (() => void) | null = null;
  private resize = new ColumnResizeController(this);
  private scrollSync = new ScrollSyncController(this);

  willUpdate(changed: PropertyValues) {
    if (changed.has('columns')) {
      this.widths = this.columns.map(c => c.width ?? DEFAULT_WIDTH);
    }
  }

  firstUpdated() {
    this.scrollSync.attach();
  }

  connectedCallback() {
    super.connectedCallback();
    this.addEventListener('dragenter', this.onDragEnter);
    this.addEventListener('dragover',  this.onDragOver);
    this.addEventListener('dragleave', this.onDragLeave);
    this.addEventListener('drop',      this.onDrop);
    void this.attachTauriDrop();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener('dragenter', this.onDragEnter);
    this.removeEventListener('dragover',  this.onDragOver);
    this.removeEventListener('dragleave', this.onDragLeave);
    this.removeEventListener('drop',      this.onDrop);
    this.tauriUnlisten?.();
    this.tauriUnlisten = null;
  }

  private async attachTauriDrop() {
    if (!this.acceptDrop) return;
    try {
      const mod = await import('@tauri-apps/api/webview');
      const wv = mod.getCurrentWebview();
      const unlisten = await wv.onDragDropEvent((event: any) => {
        const payload = event.payload ?? event;
        const type = payload.type as 'over' | 'drop' | 'leave' | undefined;
        const position = payload.position as { x: number; y: number } | undefined;
        const rect = this.getBoundingClientRect();
        const inside = !!position
          && position.x >= rect.left && position.x <= rect.right
          && position.y >= rect.top  && position.y <= rect.bottom;
        if (type === 'leave') {
          this.isDragOver = false;
        } else if (type === 'over') {
          this.isDragOver = inside;
        } else if (type === 'drop') {
          this.isDragOver = false;
          if (inside && Array.isArray(payload.paths)) {
            this.dispatchEvent(new CustomEvent('mus-list-drop', {
              detail: { paths: payload.paths },
              bubbles: true,
              composed: true,
            }));
          }
        }
      });
      this.tauriUnlisten = unlisten;
    } catch {
      // Not running under Tauri (dev preview / vitest). DOM fallback handles it.
    }
  }

  updated() {
    this.classList.toggle('drag-over', this.isDragOver);
  }

  private onDragEnter = (e: Event) => {
    if (!this.acceptDrop) return;
    e.preventDefault();
    this.isDragOver = true;
  };

  private onDragOver = (e: Event) => {
    if (!this.acceptDrop) return;
    e.preventDefault();
    this.isDragOver = true;
  };

  private onDragLeave = (e: Event) => {
    if (!this.acceptDrop) return;
    const to = (e as DragEvent).relatedTarget as Node | null;
    if (to && this.contains(to)) return;
    this.isDragOver = false;
  };

  private onDrop = (e: Event) => {
    if (!this.acceptDrop) return;
    e.preventDefault();
    this.isDragOver = false;
    const files = (e as DragEvent).dataTransfer?.files;
    if (!files) return;
    const paths = Array.from(files).map(f => (f as any).path || f.name);
    this.dispatchEvent(new CustomEvent('mus-list-drop', {
      detail: { paths },
      bubbles: true,
      composed: true,
    }));
  };

  private renderColgroup() {
    return html`<colgroup>${this.widths.map((w, i) =>
      i === this.widths.length - 1
        ? html`<col>`
        : html`<col style="width: ${w}px">`
    )}</colgroup>`;
  }

  private renderHeader() {
    return html`
      <thead>
        <tr>
          ${this.columns.map((c, i) => {
            const sortable = !!c.sortValue;
            const active = sortable && this.sortKey === c.key;
            const indicator = active
              ? html`<span class="sort-indicator">${this.sortDirection === 'asc' ? '▲' : '▼'}</span>`
              : null;
            const isLast = i === this.columns.length - 1;
            return html`
              <th data-align=${c.align ?? 'left'}
                  ?data-sortable=${sortable}
                  @click=${sortable ? () => this.onHeaderClick(c.key) : null}>
                ${c.label}${indicator}
                ${isLast ? null : html`
                  <span class="resize-handle"
                        @pointerdown=${(e: PointerEvent) => this.resize.start(e, i)}></span>
                `}
              </th>
            `;
          })}
        </tr>
      </thead>
    `;
  }

  private onHeaderClick(key: string) {
    if (this.sortKey === key) {
      this.sortDirection = this.sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      this.sortKey = key;
      this.sortDirection = 'asc';
    }
  }

  private sortedItems(items: T[]): T[] {
    if (this.sortKey === null) return items;
    const col = this.columns.find(c => c.key === this.sortKey);
    if (!col?.sortValue) return items;
    const sortValue = col.sortValue;
    const dir = this.sortDirection === 'asc' ? 1 : -1;
    return [...items].sort((a, b) => {
      const va = sortValue(a);
      const vb = sortValue(b);
      const aNil = va === null || va === undefined;
      const bNil = vb === null || vb === undefined;
      if (aNil && bNil) return 0;
      if (aNil) return 1;
      if (bNil) return -1;
      if (typeof va === 'string' && typeof vb === 'string') {
        return va.localeCompare(vb) * dir;
      }
      if (va < vb) return -1 * dir;
      if (va > vb) return 1 * dir;
      return 0;
    });
  }

  private renderBody() {
    if (!Array.isArray(this.items) || this.items.length === 0) return null;
    const rows = this.sortedItems(this.items);
    return html`<tbody>${rows.map(item => html`
      <tr>
        ${this.columns.map(c => html`
          <td data-align=${c.align ?? 'left'}>${c.render(item)}</td>
        `)}
      </tr>
    `)}</tbody>`;
  }

  private renderStateOverlay() {
    if (this.items === 'loading') {
      return html`<div class="state">Loading…</div>`;
    }
    if (!Array.isArray(this.items)) {
      return html`<div class="state">Error: ${this.items.error}</div>`;
    }
    if (this.items.length === 0) {
      return html`<div class="state">${this.emptyMessage}</div>`;
    }
    return null;
  }

  render() {
    const totalWidth = this.widths.reduce((a, b) => a + b, 0);
    const tableStyle = `min-width: ${totalWidth}px`;
    return html`
      <div class="table-container">
        <div class="table-header">
          <table style=${tableStyle}>
            ${this.renderColgroup()}
            ${this.renderHeader()}
          </table>
        </div>
        <div class="table-body">
          <table style=${tableStyle}>
            ${this.renderColgroup()}
            ${this.renderBody()}
          </table>
        </div>
        ${this.renderStateOverlay()}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap { 'mus-list-view': MusListView }
}
