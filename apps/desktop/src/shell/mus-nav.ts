import { LitElement, html, css } from 'lit';
import { customElement } from 'lit/decorators.js';
import { viewRegistry } from '../plugin-api/registry';
import { RegistrySubscription } from './registry-controller';
import '../base';

@customElement('mus-nav')
export class MusNav extends LitElement {
  static styles = css`
    :host {
      display: flex; 
      flex-direction: column;
      padding: var(--mus-space-md);
      border-right: 1px solid var(--mus-border);
      gap: var(--mus-space-xs);
      background: var(--mus-sidebar-bg);
      color: var(--mus-sidebar-text);
    }
    a {
      display: flex; 
      align-items: center; 
      gap: var(--mus-space-sm);
      padding: var(--mus-space-sm) var(--mus-space-md);
      border-radius: var(--mus-radius-sm);
      font-size: var(--mus-font-lg);
    }
    a:hover { background: var(--mus-hover-bg); }
    .footer { margin-top: auto; }

    ul {
      list-style: none;
      text-indent: 0;
      padding-left: 0;
    }

    li {
      border-radius: var(--mus-radius-sm);
    }

    li:hover { background: rgba(255,255,255,0.1); }

    a {
      color: inherit; 
      text-decoration: none; 
    }
  `;
  constructor() {
    super();
    new RegistrySubscription(this, viewRegistry);
  }
  render() {
    const views = viewRegistry.list();
    return html`
      <ul class="sidebar-list">
      ${views.map(v => html`
        <li><a href="#${v.id}">
          ${v.icon ? html`<mus-icon name=${v.icon}></mus-icon>` : ''}
          ${v.title}
        </a></li>
      `)}
      </ul>
      <div class="footer">
        <mus-slot slot-id="app.nav.footer"></mus-slot>
      </div>
    `;
  }
}

declare global { interface HTMLElementTagNameMap { 'mus-nav': MusNav } }
