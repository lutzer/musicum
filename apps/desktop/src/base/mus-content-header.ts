import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';

export interface Crumb { label: string; href?: string }

@customElement('mus-content-header')
export class MusContentHeader extends LitElement {
  static styles = css`
    :host {
      display: flex;
      width: 100%;
      padding-left: var(--mus-space-xl);
      background: var(--mus-header-bg);
    }
    h2 { margin: var(--mus-space-md); }
    nav.crumbs {
      display: flex;
      align-items: center;
      gap: var(--mus-space-sm);
      margin: var(--mus-space-md);
      font-size: var(--mus-font-lg);
      font-weight: 600;
    }
    .crumb + .crumb::before {
      content: '›';
      color: var(--mus-fg-muted);
      margin-right: var(--mus-space-sm);
    }
    .crumb a { color: inherit; text-decoration: none; }
    .crumb a:hover { text-decoration: underline; }
  `;

  @property({ type: String })     title  = '';
  @property({ attribute: false }) crumbs: Crumb[] = [];

  render() {
    if (this.crumbs.length > 0) {
      return html`
        <nav class="crumbs">
          ${this.crumbs.map(c => html`
            <span class="crumb">
              ${c.href
                ? html`<a href=${c.href}>${c.label}</a>`
                : html`${c.label}`}
            </span>
          `)}
        </nav>
      `;
    }
    return html`<h2>${this.title}</h2>`;
  }
}

declare global {
  interface HTMLElementTagNameMap { 'mus-content-header': MusContentHeader }
}
