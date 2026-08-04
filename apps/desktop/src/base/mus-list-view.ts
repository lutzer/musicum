import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';

export type ListState<T> = 'loading' | T[] | { error: string };

export interface ListColumn<T> {
  key: string;
  label: string;
  align?: 'left' | 'right';
  sortValue?: (item: T) => string | number | null;
  render: (item: T) => unknown;
}

@customElement('mus-list-view')
export class MusListView<T = unknown> extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;
      border: 1px solid var(--mus-border);
      border-radius: 10px;
      background: color-mix(in srgb, var(--mus-bg) 90%, transparent);
      overflow: hidden;
    }
    .scroll { flex: 1; overflow-y: auto; }
    table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
    th, td {
      text-align: left;
      padding: 0.4em 0.6em;
      border-bottom: 1px solid var(--mus-border);
    }
    th {
      font-weight: 600;
      position: sticky;
      top: 0;
      background: var(--mus-bg);
      user-select: none;
    }
    th[data-sortable] { cursor: pointer; }
    th[data-align='right'], td[data-align='right'] {
      text-align: right;
      font-variant-numeric: tabular-nums;
    }
    .sort-indicator { margin-left: 0.4em; opacity: 0.7; }
    :host(.drag-over) { outline: 2px dashed var(--mus-accent); outline-offset: -4px; }
    .state { padding: 1rem; }
  `;

  @property({ attribute: false }) items: ListState<T> = 'loading';
  @property({ attribute: false }) columns: ListColumn<T>[] = [];
  @property() emptyMessage = 'No items.';
  @property({ type: Boolean, attribute: 'accept-drop' }) acceptDrop = false;

  @state() private sortKey: string | null = null;
  @state() private sortDirection: 'asc' | 'desc' = 'asc';
  @state() private isDragOver = false;

  private tauriUnlisten: (() => void) | null = null;

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

  private renderHeader() {
    return html`
      <thead>
        <tr>
          ${this.columns.map(c => {
            const sortable = !!c.sortValue;
            const active = sortable && this.sortKey === c.key;
            const indicator = active
              ? html`<span class="sort-indicator">${this.sortDirection === 'asc' ? '▲' : '▼'}</span>`
              : null;
            return html`
              <th data-align=${c.align ?? 'left'}
                  ?data-sortable=${sortable}
                  @click=${sortable ? () => this.onHeaderClick(c.key) : null}>
                ${c.label}${indicator}
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
    return html`
      <div class="scroll">
        <table>
          ${this.renderHeader()}
          ${this.renderBody()}
        </table>
        ${this.renderStateOverlay()}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap { 'mus-list-view': MusListView }
}
