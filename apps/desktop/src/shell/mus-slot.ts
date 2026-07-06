import { LitElement, html } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { unsafeStatic, html as staticHtml } from 'lit/static-html.js';
import { slotRegistry } from '../plugin-api/registry';
import { RegistrySubscription } from './registry-controller';

@customElement('mus-slot')
export class MusSlot extends LitElement {
  @property({ type: String, attribute: 'slot-id' }) slotId = '';
  constructor() {
    super();
    new RegistrySubscription(this, slotRegistry);
  }
  render() {
    const entries = slotRegistry.entries(this.slotId);
    return html`${entries.map(e => staticHtml`<${unsafeStatic(e.element)}></${unsafeStatic(e.element)}>`)}`;
  }
}

declare global { interface HTMLElementTagNameMap { 'mus-slot': MusSlot } }
