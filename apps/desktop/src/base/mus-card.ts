import { LitElement, html, css } from 'lit';
import { customElement } from 'lit/decorators.js';

@customElement('mus-card')
export class MusCard extends LitElement {
  static styles = css`
    :host {
      display: block;
      padding: 1.25rem;
      border: 1px solid var(--mus-border);
      border-radius: 10px;
      background: color-mix(in srgb, var(--mus-bg) 90%, transparent);
    }
  `;
  render() { return html`<slot></slot>`; }
}

declare global {
  interface HTMLElementTagNameMap { 'mus-card': MusCard }
}
