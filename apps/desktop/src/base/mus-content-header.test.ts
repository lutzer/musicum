import { describe, it, expect, beforeEach } from 'vitest';
import './mus-content-header';

async function mount(setup: (el: HTMLElement) => void): Promise<HTMLElement> {
  const el = document.createElement('mus-content-header');
  setup(el);
  document.body.appendChild(el);
  await (el as any).updateComplete;
  return el;
}

describe('mus-content-header', () => {
  beforeEach(() => { document.body.innerHTML = ''; });

  it('renders the title as an h2 when no crumbs are given', async () => {
    const el = await mount(e => { (e as any).title = 'Files'; });
    const h2 = el.shadowRoot!.querySelector('h2');
    expect(h2?.textContent).toBe('Files');
  });

  it('renders crumbs as a trail with anchors for hrefs and text for the last', async () => {
    const el = await mount(e => {
      (e as any).crumbs = [
        { label: 'Files', href: '#files' },
        { label: 'kick.wav' },
      ];
    });
    const crumbs = el.shadowRoot!.querySelectorAll('.crumb');
    expect(crumbs).toHaveLength(2);
    const link = crumbs[0]!.querySelector('a');
    expect(link?.getAttribute('href')).toBe('#files');
    expect(link?.textContent).toBe('Files');
    expect(crumbs[1]!.querySelector('a')).toBeNull();
    expect(crumbs[1]!.textContent!.trim()).toBe('kick.wav');
  });

  it('does not render the h2 title when crumbs are present', async () => {
    const el = await mount(e => {
      (e as any).crumbs = [{ label: 'A' }];
    });
    expect(el.shadowRoot!.querySelector('h2')).toBeNull();
  });
});
