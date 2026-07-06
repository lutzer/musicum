import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { coreApi } from '../core-api';
import type { FileListItem } from '../core-api/types';
import '../base';

type State = 'loading' | FileListItem[] | { error: string };

@customElement('mus-files-view')
export class MusFilesView extends LitElement {
  static styles = css`
    :host { display: block; }
    h2 { margin-top: 0; }
    table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
    th, td { text-align: left; padding: 0.4em 0.6em;
             border-bottom: 1px solid var(--mus-border); }
    th { font-weight: 600; }
    .path { color: color-mix(in srgb, var(--mus-fg) 65%, transparent);
            font-family: ui-monospace, monospace; font-size: 0.85em; }
    .num { text-align: right; font-variant-numeric: tabular-nums; }
  `;

  @state() private items: State = 'loading';

  async connectedCallback() {
    super.connectedCallback();
    try {
      this.items = await coreApi.listFiles();
    } catch (e) {
      this.items = { error: String(e) };
    }
  }

  render() {
    if (this.items === 'loading') {
      return html`<mus-card><p>Loading…</p></mus-card>`;
    }
    if (!Array.isArray(this.items)) {
      return html`<mus-card><p>Error: ${this.items.error}</p></mus-card>`;
    }
    if (this.items.length === 0) {
      return html`<mus-card><p>No files yet.</p></mus-card>`;
    }
    return html`
      <mus-card>
        <h2>Files</h2>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Duration</th>
              <th>Path</th>
              <th class="num">Sample rate</th>
              <th class="num">Ch</th>
              <th class="num">Size</th>
              <th class="num">Clips</th>
            </tr>
          </thead>
          <tbody>
            ${this.items.map(item => html`
              <tr>
                <td>${item.file.name}</td>
                <td>${fmtDuration(item.file.duration)}</td>
                <td class="path">${item.file.path}</td>
                <td class="num">${item.file.sample_rate}</td>
                <td class="num">${item.file.channels}</td>
                <td class="num">${fmtSize(item.file.size_bytes)}</td>
                <td class="num">${item.clips.length}</td>
              </tr>
            `)}
          </tbody>
        </table>
      </mus-card>
    `;
  }
}

function fmtDuration(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, '0')}`;
}

function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

declare global {
  interface HTMLElementTagNameMap { 'mus-files-view': MusFilesView }
}
