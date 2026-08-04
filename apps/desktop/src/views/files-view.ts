import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { coreApi } from '../core-api';
import type { FileListItem } from '../core-api/types';
import type { ListColumn, ListState } from '../base/mus-list-view';
import '../base';

@customElement('mus-files-view')
export class MusFilesView extends LitElement {
  static styles = css`
    :host { display: block; height: 100%; }
    .path {
      color: var(--mus-fg-muted);
      font-family: var(--mus-font-mono);
      font-size: var(--mus-font-sm);
    }
  `;

  @state() private items: ListState<FileListItem> = 'loading';

  private columns: ListColumn<FileListItem>[] = [
    { key: 'name',     label: 'Name',
      sortValue: i => i.file.name,        render: i => i.file.name },
    { key: 'duration', label: 'Duration',
      sortValue: i => i.file.duration,    render: i => fmtDuration(i.file.duration) },
    { key: 'path',     label: 'Path',
      sortValue: i => i.file.path,
      render: i => html`<span class="path">${fmtPath(i.file.path)}</span>` },
    { key: 'info',       label: 'Info', align: 'right',
      sortValue: i => i.file.sample_rate, render: i => `${i.file.sample_rate/1000} khz (${i.file.channels})` },
    { key: 'size',     label: 'Size', align: 'right',
      sortValue: i => i.file.size_bytes,  render: i => fmtSize(i.file.size_bytes) },
    { key: 'clips',    label: 'Clips', align: 'right',
      sortValue: i => i.clips.length,     render: i => i.clips.length },
  ];

  async connectedCallback() {
    super.connectedCallback();
    try {
      this.items = await coreApi.listFiles();
    } catch (e) {
      this.items = { error: String(e) };
    }
  }

  render() {
    return html`
      <mus-list-view
        .items=${this.items}
        .columns=${this.columns}
        emptyMessage="No files yet."
        accept-drop
        @mus-list-drop=${this.onDrop}>
      </mus-list-view>
    `;
  }

  private onDrop(e: CustomEvent<{ paths: string[] }>) {
    // TODO: wire to coreApi.importPaths once available.
    console.log('files-view drop', e.detail.paths);
  }
}

function fmtDuration(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, '0')}`;
}

function fmtPath(path: string) {
  return "path"
}

function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

declare global {
  interface HTMLElementTagNameMap { 'mus-files-view': MusFilesView }
}
