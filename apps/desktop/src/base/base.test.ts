import { describe, it, expect, beforeAll } from 'vitest';
import './index';

describe('base components', () => {
  beforeAll(() => {
    // side-effect import above registers the elements
  });

  it('defines mus-button', () => {
    expect(customElements.get('mus-button')).toBeDefined();
  });

  it('mus-button renders its slotted content', async () => {
    const el = document.createElement('mus-button');
    el.textContent = 'Click';
    document.body.appendChild(el);
    await (el as any).updateComplete;
    const slot = el.shadowRoot!.querySelector('slot');
    expect(slot).not.toBeNull();
    expect(el.textContent).toBe('Click');
  });

  it('defines mus-card and mus-icon', () => {
    expect(customElements.get('mus-card')).toBeDefined();
    expect(customElements.get('mus-icon')).toBeDefined();
  });
});
