import { LitElement, html, css } from 'lit';
import { customElement } from 'lit/decorators.js';
import './mus-nav';
import './mus-view-outlet';
import './mus-slot';

@customElement('mus-app')
export class MusApp extends LitElement {
  static styles = css`
    :host { display: grid; grid-template-rows: auto 1fr; height: 100%; }
    header {
      display: flex; align-items: center;
      padding: 0.75rem 1rem;
      border-bottom: 1px solid var(--mus-border);
      gap: 0.75rem;
    }
    header h1 { font-size: 1rem; margin: 0; font-weight: 600; }
    .actions { margin-left: auto; }
    .body { display: grid; grid-template-columns: auto 1fr; min-height: 0; }
  `;
  render() {
    return html`
      <header>
        <h1>Musicum</h1>
        <div class="actions"><mus-slot slot-id="app.header.actions"></mus-slot></div>
      </header>
      <div class="body">
        <mus-nav></mus-nav>
        <mus-view-outlet></mus-view-outlet>
      </div>
    `;
  }
}

declare global { interface HTMLElementTagNameMap { 'mus-app': MusApp } }
