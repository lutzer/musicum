import { describe, it, expect, beforeEach } from 'vitest';
import { viewRegistry } from '../plugin-api/registry';
import './mus-nav';

async function mount(): Promise<HTMLElement> {
  const el = document.createElement('mus-nav');
  document.body.appendChild(el);
  await (el as any).updateComplete;
  return el;
}

function navItems(el: HTMLElement): HTMLElement[] {
  return Array.from(el.shadowRoot!.querySelectorAll('li'));
}

describe('mus-nav', () => {
  beforeEach(() => {
    document.body.innerHTML = '';
    // Clear registry state between tests.
    (viewRegistry as any).views.clear();
  });

  it('renders one <li> per visible view', async () => {
    viewRegistry.register('t', { id: 'x', title: 'X', element: 'x-el' });
    viewRegistry.register('t', { id: 'y', title: 'Y', element: 'y-el' });
    const el = await mount();
    expect(navItems(el)).toHaveLength(2);
  });

  it('omits views with sidebar: false', async () => {
    viewRegistry.register('t', { id: 'a', title: 'A', element: 'a-el' });
    viewRegistry.register('t', {
      id: 'a-detail', title: 'A detail', element: 'a-detail-el', sidebar: false,
    });
    const el = await mount();
    const labels = navItems(el).map(li => li.textContent!.trim());
    expect(labels).toEqual(['A']);
  });

  it('marks the parent list entry active for detail routes', async () => {
    viewRegistry.register('t', { id: 'files', title: 'Files', element: 'f-el' });
    viewRegistry.register('t', {
      id: 'files-detail', title: 'File', element: 'f-detail', sidebar: false,
    });
    window.location.hash = '#files/abc';
    const el = await mount();
    // Give hashchange handler a tick to settle.
    await new Promise(r => setTimeout(r, 0));
    await (el as any).updateComplete;
    const activeLabels = navItems(el)
      .filter(li => li.classList.contains('active'))
      .map(li => li.textContent!.trim());
    expect(activeLabels).toEqual(['Files']);
  });
});
