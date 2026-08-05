import { LitElement, html, css, type PropertyValues } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { coreApi } from '../core-api';
import type { ClipModel, FileModel } from '../core-api/types';
import type { ListColumn, ListState } from '../base/mus-list-view';
import { fmtDuration, fmtSize, relativeFolder } from './file-format';
import '../base';

type Loaded = {
  file: FileModel;
  clips: ClipModel[];
  notes: string;
  tags: string;
};

type Data = 'loading' | Loaded | { error: string };

@customElement('mus-file-detail-view')
export class MusFileDetailView extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      width: 100%;
    }
    .body {
      margin: var(--mus-space-md);
      display: flex;
      flex-direction: column;
      gap: var(--mus-space-md);
    }
    dl.meta {
      display: grid;
      grid-template-columns: max-content 1fr;
      gap: var(--mus-space-xs) var(--mus-space-md);
      margin: 0;
    }
    dl.meta dt {
      color: var(--mus-fg-muted);
      font-size: var(--mus-font-sm);
    }
    dl.meta dd {
      margin: 0;
      font-family: var(--mus-font-mono);
      font-size: var(--mus-font-sm);
    }
    .field { display: flex; flex-direction: column; gap: var(--mus-space-xs); }
    .field label {
      color: var(--mus-fg-muted);
      font-size: var(--mus-font-sm);
    }
    textarea, input[type="text"] {
      font-family: inherit;
      font-size: var(--mus-font-md);
      padding: var(--mus-space-sm);
    }
    textarea { min-height: 5em; resize: vertical; }
    mus-list-view { min-height: 8em; }
  `;

  @property({ type: String }) slug = '';
  @state() private data: Data = 'loading';
  @state() private libraryDir = '';
  private loadedSlug = '';
  private savedNotes = '';
  private savedTags = '';

  private clipColumns: ListColumn<ClipModel>[] = [
    { key: 'title', label: 'Title', width: 240,
      sortValue: c => c.title, render: c => c.title },
    { key: 'duration', label: 'Duration', width: 100,
      sortValue: c => c.duration,
      render:    c => c.duration == null ? '—' : fmtDuration(c.duration) },
    { key: 'notes', label: 'Notes', width: 200,
      sortValue: c => c.notes, render: c => c.notes },
  ];

  connectedCallback() {
    super.connectedCallback();
    void this.load();
  }

  updated(changed: PropertyValues) {
    if (changed.has('slug') && this.slug && this.slug !== this.loadedSlug) {
      void this.load();
    }
  }

  private async load() {
    this.data = 'loading';
    const slug = this.slug;
    try {
      const [libraryDir, listItem] = await Promise.all([
        coreApi.getLibraryDir(),
        coreApi.getFileBySlug(slug),
      ]);
      const metadata = await coreApi.getFileMetadata(listItem.file.id);
      // Ignore stale response if the slug changed while awaiting.
      if (slug !== this.slug) return;
      this.libraryDir = libraryDir;
      const notes = metadata?.notes ?? '';
      const tags  = metadata?.tags  ?? '';
      this.savedNotes = notes;
      this.savedTags  = tags;
      this.loadedSlug = slug;
      this.data = { file: listItem.file, clips: listItem.clips, notes, tags };
    } catch (e) {
      if (slug !== this.slug) return;
      this.data = { error: String(e) };
    }
  }

  render() {
    if (this.data === 'loading') {
      return html`
        <mus-content-header title="File"></mus-content-header>
        <div class="body"><p>Loading…</p></div>
      `;
    }
    if ('error' in this.data) {
      return html`
        <mus-content-header title="File"></mus-content-header>
        <div class="body"><p>Error: ${this.data.error}</p></div>
      `;
    }
    const { file, clips, notes, tags } = this.data;
    const clipsState: ListState<ClipModel> = clips;
    const crumbs = [
      { label: 'Files', href: '#files' },
      { label: file.name },
    ];
    return html`
      <mus-content-header .crumbs=${crumbs}></mus-content-header>
      <div class="body">
        <mus-card>
          <dl class="meta">
            <dt>Name</dt>          <dd>${file.name}</dd>
            <dt>Folder</dt>        <dd>${relativeFolder(file.path, this.libraryDir)}</dd>
            <dt>Path</dt>          <dd>${file.path}</dd>
            <dt>Duration</dt>      <dd>${fmtDuration(file.duration)}</dd>
            <dt>Audio</dt>         <dd>${file.sample_rate / 1000} kHz · ${file.channels} ch · ${file.mime_type}</dd>
            <dt>Size</dt>          <dd>${fmtSize(file.size_bytes)}</dd>
            <dt>Hash</dt>          <dd>${file.hash.slice(0, 12)}</dd>
            <dt>Created</dt>       <dd>${file.created_at}</dd>
            <dt>Updated</dt>       <dd>${file.updated_at}</dd>
          </dl>
        </mus-card>

        <mus-card>
          <div class="field">
            <label for="notes">Notes</label>
            <textarea id="notes"
              .value=${notes}
              @input=${this.onNotesInput}
              @blur=${this.saveNotes}></textarea>
          </div>
          <div class="field">
            <label for="tags">Tags (comma-separated)</label>
            <input id="tags" type="text" data-field="tags"
              .value=${tags}
              @input=${this.onTagsInput}
              @blur=${this.saveTags}>
          </div>
        </mus-card>

        <mus-card>
          <h3>Clips</h3>
          <mus-list-view
            .items=${clipsState}
            .columns=${this.clipColumns}
            emptyMessage="No clips for this file yet.">
          </mus-list-view>
        </mus-card>
      </div>
    `;
  }

  private onNotesInput = (e: Event) => {
    if (this.data === 'loading' || 'error' in this.data) return;
    this.data = { ...this.data, notes: (e.target as HTMLTextAreaElement).value };
  };
  private onTagsInput = (e: Event) => {
    if (this.data === 'loading' || 'error' in this.data) return;
    this.data = { ...this.data, tags: (e.target as HTMLInputElement).value };
  };

  private saveNotes = async () => {
    if (this.data === 'loading' || 'error' in this.data) return;
    const next = this.data.notes;
    if (next === this.savedNotes) return;
    try {
      await coreApi.setFileNotes(this.slug, next);
      this.savedNotes = next;
    } catch (e) {
      console.error('setFileNotes failed', e);
    }
  };

  private saveTags = async () => {
    if (this.data === 'loading' || 'error' in this.data) return;
    const next = this.data.tags;
    if (next === this.savedTags) return;
    try {
      await coreApi.setFileTags(this.slug, next);
      this.savedTags = next;
    } catch (e) {
      console.error('setFileTags failed', e);
    }
  };
}

declare global {
  interface HTMLElementTagNameMap { 'mus-file-detail-view': MusFileDetailView }
}
