import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';

@customElement('mus-button')
export class MusButton extends LitElement {
  static styles = css`
    :host { display: inline-block; }
    button {
      font: inherit;
      padding: var(--mus-space-sm) var(--mus-space-md);
      border-radius: var(--mus-radius-sm);
      border: 1px solid var(--mus-border);
      background: transparent;
      color: inherit;
      cursor: pointer;
    }
    button[data-variant='primary'] {
      background: var(--mus-accent);
      color: var(--mus-accent-fg);
      border-color: transparent;
    }
  `;

  @property({ type: String }) variant: 'primary' | 'secondary' | 'ghost' = 'secondary';

  render() {
    return html`<button data-variant=${this.variant}><slot></slot></button>`;
  }
}

declare global {
  interface HTMLElementTagNameMap { 'mus-button': MusButton }
}
