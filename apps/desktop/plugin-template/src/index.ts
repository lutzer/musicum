import { LitElement, html } from 'lit';
import type { PluginModule } from '@musicum/plugin-api';

class MyView extends LitElement {
  render() { return html`<h2>Hello from my plugin</h2>`; }
}
customElements.define('my-plugin-view', MyView);

export default {
  async register(ctx) {
    ctx.registerView({ id: 'my.view', title: 'My Plugin', element: 'my-plugin-view' });
  },
} satisfies PluginModule;
