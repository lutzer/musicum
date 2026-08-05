import { describe, it, expect, beforeEach } from 'vitest';
import { viewRegistry } from '../plugin-api/registry';
import { LitElement, html } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import './mus-content';

@customElement('mus-test-list')  class MusTestList   extends LitElement {
  render() { return html`<span>list</span>`; }
}
@customElement('mus-test-detail') class MusTestDetail extends LitElement {
  @property({ type: String }) slug = '';
  render() { return html`<span>detail:${this.slug}</span>`; }
}

async function mount(): Promise<HTMLElement> {
  const el = document.createElement('mus-content');
  document.body.appendChild(el);
  await (el as any).updateComplete;
  await new Promise(r => setTimeout(r, 0));
  await (el as any).updateComplete;
  return el;
}

async function shadowText(el: Element | null): Promise<string> {
  if (!el) return '';
  await (el as any).updateComplete;
  return (el as HTMLElement).shadowRoot?.textContent ?? '';
}

describe('mus-content routing', () => {
  beforeEach(() => {
    document.body.innerHTML = '';
    (viewRegistry as any).views.clear();
    viewRegistry.register('t', { id: 'thing', title: 'Thing', element: 'mus-test-list' });
    viewRegistry.register('t', {
      id: 'thing-detail', title: 'Thing detail', element: 'mus-test-detail',
      sidebar: false,
    });
  });

  it('renders the list view when the hash has no slash', async () => {
    window.location.hash = '#thing';
    const el = await mount();
    const list = el.shadowRoot!.querySelector('mus-test-list');
    expect(list).not.toBeNull();
    expect(await shadowText(list)).toContain('list');
  });

  it('renders the detail view with the slug attribute when hash is thing/<slug>', async () => {
    window.location.hash = '#thing/abc';
    const el = await mount();
    const detail = el.shadowRoot!.querySelector('mus-test-detail')!;
    expect(detail).not.toBeNull();
    expect(detail.getAttribute('slug')).toBe('abc');
    expect(await shadowText(detail)).toContain('detail:abc');
  });

  it('normalizes trailing slash to no-param (renders list, not detail)', async () => {
    window.location.hash = '#thing/';
    const el = await mount();
    const list = el.shadowRoot!.querySelector('mus-test-list');
    expect(list).not.toBeNull();
    expect(await shadowText(list)).toContain('list');
    expect(el.shadowRoot!.querySelector('mus-test-detail')).toBeNull();
  });

  it('shows an unknown-view message when detail is unregistered', async () => {
    (viewRegistry as any).views.delete('thing-detail');
    window.location.hash = '#thing/abc';
    const el = await mount();
    expect(el.shadowRoot!.textContent).toContain('Unknown view');
  });
});

declare global {
  interface HTMLElementTagNameMap {
    'mus-test-list':   MusTestList;
    'mus-test-detail': MusTestDetail;
  }
}
