import { LitElement, html, css } from 'lit';
import { customElement } from 'lit/decorators.js';

@customElement('mus-card')
export class MusCard extends LitElement {
  static styles = css`
    :host {
      display: block;
      padding: var(--mus-space-lg);
      border: 1px solid var(--mus-border);
      border-radius: var(--mus-radius-md);
      background: var(--mus-accent-bg);
    }
  `;
  render() { return html`<slot></slot>`; }
}

declare global {
  interface HTMLElementTagNameMap { 'mus-card': MusCard }
}
