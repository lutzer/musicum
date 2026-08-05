import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { coreApi } from '../core-api';
import type { FileListItem } from '../core-api/types';
import type { ListColumn, ListState } from '../base/mus-list-view';
import '../base';

@customElement('mus-files-view')
export class MusFilesView extends LitElement {
  static styles = css`
    :host { 
      display: flex; 
      flex-direction: column;
      width: 100%;
    }
    .path {
      color: var(--mus-fg-muted);
      font-family: var(--mus-font-mono);
      font-size: var(--mus-font-sm);
    }

    mus-list-view {
      margin: var(--mus-space-md);
    }
  `;

  @state() private items: ListState<FileListItem> = 'loading';
  @state() private libraryDir = '';

  private columns: ListColumn<FileListItem>[] = [
    { key: 'name',     label: 'Name',                    width: 300,
      sortValue: i => i.file.name,        render: i => i.file.name },
    { key: 'path',     label: 'Path',                    width: 200,
      sortValue: i => relativeFolder(i.file.path, this.libraryDir),
      render:    i => html`<span class="path">${relativeFolder(i.file.path, this.libraryDir)}</span>` },
    { key: 'clips',    label: 'Clips',   width: 80,
      sortValue: i => i.clips.length,     render: i => i.clips.length },
    { key: 'duration', label: 'Length',                width: 80,
      sortValue: i => i.file.duration,    render: i => fmtDuration(i.file.duration) },
    { key: 'size',     label: 'Size',    width: 100,
      sortValue: i => i.file.size_bytes,  render: i => fmtSize(i.file.size_bytes) },
    { key: 'info',     label: 'Info',    width: 140,
      sortValue: i => i.file.sample_rate, render: i => `${i.file.sample_rate/1000} khz (${i.file.channels}) ${i.file.mime_type}` },
    

  ];

  async connectedCallback() {
    super.connectedCallback();
    try {
      const [libraryDir, items] = await Promise.all([
        coreApi.getLibraryDir(),
        coreApi.listFiles(),
      ]);
      this.libraryDir = libraryDir;
      this.items = items;
    } catch (e) {
      this.items = { error: String(e) };
    }
  }

  render() {
    return html`
      <mus-content-header
        title="Files">
      </mus-content-header>
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

function relativeFolder(filePath: string, libraryDir: string): string {
  if (!libraryDir) return '-';
  const dir = libraryDir.endsWith('/') ? libraryDir.slice(0, -1) : libraryDir;
  const lastSep = filePath.lastIndexOf('/');
  if (lastSep === -1) return '-';
  const parent = filePath.slice(0, lastSep);
  if (parent === dir) return '-';
  if (parent.startsWith(dir + '/')) return parent.slice(dir.length + 1);
  return '-';
}

function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

declare global {
  interface HTMLElementTagNameMap { 'mus-files-view': MusFilesView }
}
