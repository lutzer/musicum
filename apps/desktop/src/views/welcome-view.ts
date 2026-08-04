import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { coreApi } from '../core-api';
import { viewRegistry } from '../plugin-api/registry';
import { RegistrySubscription } from '../shell/registry-controller';
import '../base';
import '../shell/mus-slot';

@customElement('mus-welcome-view')
export class MusWelcomeView extends LitElement {
  static styles = css`
    :host { display: block; max-width: 640px; }
    h2 { margin-top: 0; }
    .meta { color: var(--mus-fg-muted); font-size: var(--mus-font-md); }
  `;

  @state() private appName = '…';
  @state() private appVersion = '';

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
    } catch (e) {
      console.error('getAppInfo failed', e);
      this.appName = 'musicum-desktop';
      this.appVersion = 'unknown';
    }
  }

  render() {
    const count = viewRegistry.list().length;
    return html`
      <mus-card>
        <h2>Welcome to ${this.appName}</h2>
        <p class="meta">version ${this.appVersion} · ${count} view(s) registered</p>
        <mus-slot slot-id="view.welcome.body"></mus-slot>
      </mus-card>
    `;
  }
}

declare global { interface HTMLElementTagNameMap { 'mus-welcome-view': MusWelcomeView } }
