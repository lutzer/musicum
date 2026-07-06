import { LitElement, html, css } from 'lit';
import { customElement } from 'lit/decorators.js';
import { viewRegistry } from '../plugin-api/registry';
import { RegistrySubscription } from './registry-controller';
import '../base';

@customElement('mus-nav')
export class MusNav extends LitElement {
  static styles = css`
    :host {
      display: flex; flex-direction: column;
      width: 220px; padding: 1rem;
      border-right: 1px solid var(--mus-border);
      gap: 0.25rem;
      background: color-mix(in srgb, var(--mus-bg) 90%, transparent);
    }
    a {
      display: flex; align-items: center; gap: 0.5em;
      padding: 0.5em 0.75em; border-radius: 6px;
      color: inherit; text-decoration: none; font-size: 0.95em;
    }
    a:hover { background: color-mix(in srgb, var(--mus-fg) 8%, transparent); }
    .footer { margin-top: auto; }
  `;
  constructor() {
    super();
    new RegistrySubscription(this, viewRegistry);
  }
  render() {
    const views = viewRegistry.list();
    return html`
      ${views.map(v => html`
        <a href="#${v.id}">
          ${v.icon ? html`<mus-icon name=${v.icon}></mus-icon>` : ''}
          ${v.title}
        </a>
      `)}
      <div class="footer">
        <mus-slot slot-id="app.nav.footer"></mus-slot>
      </div>
    `;
  }
}

declare global { interface HTMLElementTagNameMap { 'mus-nav': MusNav } }
