import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';

@customElement('mus-content-header')
export class MusContentHeader extends LitElement {
  static styles = css`
    :host {
        display: flex;
        width: 100%;
        padding-left: var(--mus-space-xl);
        background: var(--mus-header-bg)
    }

    h2 {
      margin: var(--mus-space-md);
    }
  `;

  @property({ type: String }) title = 'unknown';

  render() {
    return html`<h2>${this.title}</h2>`;
  }
}

declare global {
  interface HTMLElementTagNameMap { 'mus-content-header': MusContentHeader }
}