import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { coreApi } from '../core-api';
import { viewRegistry } from '../plugin-api/registry';
import { RegistrySubscription } from '../shell/registry-controller';
import '../base';
import '../shell/mus-slot';
import { core } from '@tauri-apps/api';

@customElement('mus-welcome-view')
export class MusWelcomeView extends LitElement {
  static styles = css`
    :host { 
      display: flex; 
      width: 100%;
      flex-direction: column;
    }
    h2 { 
      margin-top: 0;
    }
    .meta { 
      color: var(--mus-fg-muted); 
      font-size: var(--mus-font-md); 
    }

    .center {
      display: flex;
      justify-content: center;
      align-items: center;
      flex: 1;
      padding: var(--mus-space-md);
      text-align: center;
    }
  `;

  @state() private appName = '…';
  @state() private appVersion = '';
  @state() private fileCount = 0;
  @state() private clipCount = 0;
  @state() private libraryFolder = '';

  constructor() {
    super();
    new RegistrySubscription(this, viewRegistry);
  }

  async connectedCallback() {
    super.connectedCallback();
    try {
      const info = await coreApi.getAppInfo();
      this.appName = info.name;
      this.appVersion = info.version;
      this.fileCount = (await coreApi.listFiles()).length;
      this.clipCount = (await coreApi.listClips()).length;
      this.libraryFolder = await coreApi.getLibraryDir();
    } catch (e) {
      console.error('getAppInfo failed', e);
      this.appName = 'musicum-desktop';
      this.appVersion = 'unknown';
    }
  }

  render() {
    const count = viewRegistry.list().length;
    return html`
      <mus-content-header
        title="Welcome">
      </mus-content-header>
      <div class="center">
        <mus-card>
          <h2>Welcome to ${this.appName}</h2>
          <p class="meta">App version ${this.appVersion} · ${count} view(s) registered</p>
          <p class="meta">Library location: ${this.libraryFolder}</p>
          <p class="meta">Database contains ${this.fileCount} files and  ${this.clipCount} clips</p>
          <mus-slot slot-id="view.welcome.body"></mus-slot>
        </mus-card>
      </div>
    `;
  }
}

declare global { interface HTMLElementTagNameMap { 'mus-welcome-view': MusWelcomeView } }
