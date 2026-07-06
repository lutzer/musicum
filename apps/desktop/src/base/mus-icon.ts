import { LitElement, html, css, svg, nothing } from 'lit';
import { customElement, property } from 'lit/decorators.js';

const ICONS: Record<string, ReturnType<typeof svg>> = {
  home: svg`<path d="M3 11 12 4l9 7v9a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1z" fill="currentColor"/>`,
  info: svg`<circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="2"/><path d="M12 10v6M12 7v.5" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>`,
  plugin: svg`<path d="M9 3v4H5v4a4 4 0 0 0 4 4v6h6v-6a4 4 0 0 0 4-4V7h-4V3h-2v4h-2V3z" fill="currentColor"/>`,
  settings: svg`<circle cx="12" cy="12" r="3" fill="none" stroke="currentColor" stroke-width="2"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.9l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.9-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1A1.7 1.7 0 0 0 9 19.4a1.7 1.7 0 0 0-1.9.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.9 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1A1.7 1.7 0 0 0 4.6 9a1.7 1.7 0 0 0-.3-1.9l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.9.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.9-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.9V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" fill="none" stroke="currentColor" stroke-width="2"/>`,
};

@customElement('mus-icon')
export class MusIcon extends LitElement {
  static styles = css`
    :host { display: inline-flex; width: 1em; height: 1em; }
    svg { width: 100%; height: 100%; }
  `;

  @property({ type: String }) name = '';

  render() {
    const g = ICONS[this.name];
    if (!g) return nothing;
    return html`<svg viewBox="0 0 24 24" aria-hidden="true">${g}</svg>`;
  }
}

declare global {
  interface HTMLElementTagNameMap { 'mus-icon': MusIcon }
}
