import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { coreApi } from '../core-api';
import type { ClipListItem } from '../core-api/types';
import '../base';

type State = 'loading' | ClipListItem[] | { error: string };

@customElement('mus-clips-view')
export class MusClipsView extends LitElement {
  static styles = css`
    :host { display: block; }
    h2 { margin-top: 0; }
    table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
    th, td { text-align: left; padding: 0.4em 0.6em;
             border-bottom: 1px solid var(--mus-border); }
    th { font-weight: 600; }
    .num { text-align: right; font-variant-numeric: tabular-nums; }
  `;

  @state() private items: State = 'loading';

  async connectedCallback() {
    super.connectedCallback();
    try {
      this.items = await coreApi.listClips();
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
      return html`<mus-card><p>No clips yet.</p></mus-card>`;
    }
    return html`
      <mus-card>
        <h2>Clips</h2>
        <table>
          <thead>
            <tr>
              <th>Title</th>
              <th>Source file</th>
              <th class="num">Duration</th>
            </tr>
          </thead>
          <tbody>
            ${this.items.map(item => html`
              <tr>
                <td>${item.clip.title}</td>
                <td>${item.file.name}</td>
                <td class="num">${fmtDuration(item.clip.duration)}</td>
              </tr>
            `)}
          </tbody>
        </table>
      </mus-card>
    `;
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
