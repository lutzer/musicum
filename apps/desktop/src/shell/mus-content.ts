import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { unsafeStatic, html as staticHtml } from 'lit/static-html.js';
import { viewRegistry } from '../plugin-api/registry';
import { RegistrySubscription } from './registry-controller';

@customElement('mus-content')
export class MusContent extends LitElement {
  static styles = css`
    :host { display: flex; }
  `;

  @state() private activeId: string | undefined = readHashViewId();

  constructor() {
    super();
    new RegistrySubscription(this, viewRegistry);
    window.addEventListener('hashchange', () => {
      this.activeId = readHashViewId();
    });
  }

  render() {
    const target = this.activeId ?? viewRegistry.list()[0]?.id;
    if (!target) return html`<p>No views registered.</p>`;
    const view = viewRegistry.get(target);
    if (!view) return html`<p>Unknown view: ${target}</p>`;
    return staticHtml`<${unsafeStatic(view.element)}></${unsafeStatic(view.element)}>`;
  }
}

function readHashViewId(): string | undefined {
  const h = window.location.hash.replace(/^#/, '').trim();
  return h.length ? h : undefined;
}

declare global { interface HTMLElementTagNameMap { 'mus-content': MusContent } }
