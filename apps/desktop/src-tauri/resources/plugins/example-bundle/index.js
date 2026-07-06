import { LitElement, html, css } from 'lit';

class ExWelcomeView extends LitElement {
  static styles = css`
    :host { display: block; }
    p { color: color-mix(in srgb, currentColor 70%, transparent); }
  `;
  render() {
    return html`
      <h2>Example bundle: welcome</h2>
      <p>This view was registered by <code>example-bundle</code>.</p>
    `;
  }
}
customElements.define('ex-welcome-view', ExWelcomeView);

class ExAboutView extends LitElement {
  render() {
    return html`
      <h2>Example bundle: about</h2>
      <p>Two views + one slot entry, all from a single register() call.</p>
    `;
  }
}
customElements.define('ex-about-view', ExAboutView);

class ExHelloBadge extends LitElement {
  static styles = css`
    :host {
      display: inline-block;
      padding: 0.15em 0.6em;
      border-radius: 999px;
      background: var(--mus-accent, #3a5dff);
      color: white;
      font-size: 0.75em;
      margin-top: 0.75em;
    }
  `;
  render() { return html`<slot>plugin loaded</slot>`; }
}
customElements.define('ex-hello-badge', ExHelloBadge);

export default {
  async register(ctx) {
    ctx.registerView({ id: 'example.welcome', title: 'Example',   icon: 'plugin', element: 'ex-welcome-view' });
    ctx.registerView({ id: 'example.about',   title: 'About Ex.', icon: 'info',   element: 'ex-about-view' });
    ctx.registerSlot('view.welcome.body', { id: 'ex.badge', element: 'ex-hello-badge' });
  },
};
