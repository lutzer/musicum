import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { coreApi } from '../core-api';
import type { FileListItem } from '../core-api/types';
import type { ListColumn, ListState } from '../base/mus-list-view';
import { fmtDuration, fmtSize, relativeFolder } from './file-format';
import '../base';

@customElement('mus-file-list-view')
export class MusFileListView extends LitElement {
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
    a { color: inherit; text-decoration: none; }
    a:hover { text-decoration: underline; }

    mus-list-view {
      margin: var(--mus-space-md);
    }
  `;

  @state() private items: ListState<FileListItem> = 'loading';
  @state() private libraryDir = '';

  private columns: ListColumn<FileListItem>[] = [
    { key: 'name', label: 'Name', width: 300,
      sortValue: i => i.file.name,
      render:    i => html`${i.file.name}` },
    { key: 'path', label: 'Path', width: 200,
      sortValue: i => relativeFolder(i.file.path, this.libraryDir),
      render:    i => html`<span class="path">${relativeFolder(i.file.path, this.libraryDir)}</span>` },
    { key: 'clips', label: 'Clips', width: 80,
      sortValue: i => i.clips.length, render: i => i.clips.length },
    { key: 'duration', label: 'Length', width: 80,
      sortValue: i => i.file.duration, render: i => fmtDuration(i.file.duration) },
    { key: 'size', label: 'Size', width: 100,
      sortValue: i => i.file.size_bytes, render: i => fmtSize(i.file.size_bytes) },
    { key: 'info', label: 'Info', width: 140,
      sortValue: i => i.file.sample_rate,
      render:    i => `${i.file.sample_rate / 1000} khz (${i.file.channels}) ${i.file.mime_type}` }
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

  handleItemClick(item: FileListItem) {
    window.location.href = `#files/${item.file.slug}`;
  }

  render() {
    return html`
      <mus-content-header title="Files"></mus-content-header>
      <mus-list-view
        .items=${this.items}
        .columns=${this.columns}
        .onItemClicked=${this.handleItemClick}
        emptyMessage="No files yet."
        accept-drop
        @mus-list-drop=${this.onDrop}>
      </mus-list-view>
    `;
  }

  private onDrop(e: CustomEvent<{ paths: string[] }>) {
    console.log('file-list-view drop', e.detail.paths);
  }
}

declare global {
  interface HTMLElementTagNameMap { 'mus-file-list-view': MusFileListView }
}
