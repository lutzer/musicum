import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';

@customElement('mus-content-header')
export class MusContentHeader extends LitElement {
  static styles = css`
    :host {
        display: block;
        width: 100%;
        padding: 0 var(--mus-space-xl);
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
