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
      background: var(--mus-sidebar-bg);
      width: var(--mus-sidebar-width);
      min-width: var(--mus-sidebar-min);
      max-width: var(--mus-sidebar-max);
      flex-shrink: 0;
      overflow-y: auto;
    }

    .content {
      flex: 1;
      background: var(--mus-content-bg);
      overflow-y: auto;
      /* padding: 0; */
    }

    /* --- the drag handle --- */

    .resizer {
      width: var(--mus-resizer-width);
      flex-shrink: 0;
      cursor: col-resize;
      background: var(--mus-resizer-idle);
      position: relative;
      transition: background var(--mus-transition-fast);
    }

    .resizer:hover,
    .resizer.dragging {
      background: var(--mus-resizer-hover);
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
