import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { coreApi } from '../core-api';
import type { ClipListItem } from '../core-api/types';
import type { ListColumn, ListState } from '../base/mus-list-view';
import '../base';

@customElement('mus-clips-view')
export class MusClipsView extends LitElement {
  static styles = css`
    :host { 
      display: flex; 
      flex-direction: column;
      width: 100%;
    }
  `;

  @state() private items: ListState<ClipListItem> = 'loading';

  private columns: ListColumn<ClipListItem>[] = [
    { key: 'title',    label: 'Title',                   width: 240,
      sortValue: i => i.clip.title, render: i => i.clip.title },
    { key: 'file',     label: 'Source file',             width: 240,
      sortValue: i => i.file.name,  render: i => i.file.name },
    { key: 'duration', label: 'Duration', align: 'right', width: 80,
      sortValue: i => i.clip.duration,
      render:    i => fmtDuration(i.clip.duration) },
  ];

  async connectedCallback() {
    super.connectedCallback();
    try {
      this.items = await coreApi.listClips();
    } catch (e) {
      this.items = { error: String(e) };
    }
  }

  render() {
    return html`
      <mus-content-header
        title="Clips">
      </mus-content-header>
      <mus-list-view
        .items=${this.items}
        .columns=${this.columns}
        emptyMessage="No clips yet."
        accept-drop
        @mus-list-drop=${this.onDrop}>
      </mus-list-view>
    `;
  }

  private onDrop(e: CustomEvent<{ paths: string[] }>) {
    // TODO: wire to coreApi.createClipFromPath once available.
    console.log('clips-view drop', e.detail.paths);
  }
}

function fmtDuration(seconds: number | null): string {
  if (seconds === null) return '—';
  const s = Math.max(0, Math.floor(seconds));
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, '0')}`;
}

declare global {
  interface HTMLElementTagNameMap { 'mus-clips-view': MusClipsView }
}
