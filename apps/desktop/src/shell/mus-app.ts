import { LitElement, html, css } from 'lit';
import { customElement } from 'lit/decorators.js';
import './mus-nav';
import './mus-content';
import './mus-slot';

@customElement('mus-app')
export class MusApp extends LitElement {
  static styles = css`
    .layout { 
      display: flex;
      height: 100%;
      width: 100%;
      overflow: hidden;
    }
    .sidebar {
      background: var(--sidebar-bg);
      color: var(--sidebar-text);
      width: 200px;
      min-width: 160px;
      max-width: 480px;
      flex-shrink: 0;
      padding: 20px 16px;
      overflow-y: auto;
    }

    .content {
      flex: 1;
      background: var(--content-bg);
      padding: 32px;
      overflow-y: auto;
    }

    /* --- the drag handle --- */

    .resizer {
      width: 6px;
      flex-shrink: 0;
      cursor: col-resize;
      background: var(--handle-idle);
      position: relative;
      transition: background 0.15s;
    }

    .resizer:hover,
    .resizer.dragging {
      background: var(--handle-hover);
    }

    .resizer::after {
      content: "";
      position: absolute;
      top: 0; bottom: 0;
      left: -4px; right: -4px;
    }

  `;
  render() {
    return html`
      <!-- <header>
        <h1>Musicum</h1>
        <div class="actions"><mus-slot slot-id="app.header.actions"></mus-slot></div>
      </header> -->
      <div class="layout">
        <mus-nav class="sidebar"></mus-nav>
        <div class="resizer" id="resizer"></div>
        <mus-content class="content"></mus-content>
      </div>
    `;
  }
}

declare global { interface HTMLElementTagNameMap { 'mus-app': MusApp } }
