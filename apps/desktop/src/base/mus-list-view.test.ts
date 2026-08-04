import { describe, it, expect, beforeEach, vi } from 'vitest';
import { html } from 'lit';
import type { ListColumn } from './mus-list-view';
import './mus-list-view';

interface Row { id: string; n: number }

const columns: ListColumn<Row>[] = [
  { key: 'id', label: 'ID',    sortValue: r => r.id, render: r => r.id },
  { key: 'n',  label: 'N', align: 'right',
    sortValue: r => r.n, render: r => r.n },
  { key: 'raw', label: 'Raw', render: r => html`<span class="raw">${r.id}</span>` },
];

async function mount(props: Partial<Record<string, unknown>> = {}) {
  const el = document.createElement('mus-list-view') as any;
  el.columns = columns;
  for (const [k, v] of Object.entries(props)) el[k] = v;
  document.body.appendChild(el);
  await el.updateComplete;
  return el as HTMLElement & { updateComplete: Promise<void> };
}

describe('mus-list-view rendering', () => {
  beforeEach(() => { document.body.innerHTML = ''; });

  it('renders the loading state', async () => {
    const el = await mount({ items: 'loading' });
    expect(el.shadowRoot!.textContent).toContain('Loading');
    expect(el.shadowRoot!.querySelectorAll('thead th')).toHaveLength(columns.length);
  });

  it('renders the error state', async () => {
    const el = await mount({ items: { error: 'boom' } });
    expect(el.shadowRoot!.textContent).toContain('boom');
    expect(el.shadowRoot!.querySelectorAll('thead th')).toHaveLength(columns.length);
  });

  it('renders the empty message when items is []', async () => {
    const el = await mount({ items: [], emptyMessage: 'Nothing here.' });
    expect(el.shadowRoot!.textContent).toContain('Nothing here.');
    expect(el.shadowRoot!.querySelectorAll('tbody tr')).toHaveLength(0);
  });

  it('renders one row per item and uses column render fns', async () => {
    const el = await mount({ items: [{ id: 'a', n: 1 }, { id: 'b', n: 2 }] });
    const rows = el.shadowRoot!.querySelectorAll('tbody tr');
    expect(rows).toHaveLength(2);
    expect(rows[0]!.children).toHaveLength(columns.length);
    expect(rows[0]!.textContent).toContain('a');
    expect(rows[0]!.textContent).toContain('1');
    expect(rows[0]!.querySelector('span.raw')).not.toBeNull();
  });
});

describe('mus-list-view sorting', () => {
  beforeEach(() => { document.body.innerHTML = ''; });

  function headerTexts(el: HTMLElement): string[] {
    return Array.from(el.shadowRoot!.querySelectorAll('tbody tr'))
      .map(r => r.children[0]!.textContent!.trim());
  }

  it('clicking a sortable header sorts asc, clicking again sorts desc', async () => {
    const items = [{ id: 'c', n: 3 }, { id: 'a', n: 1 }, { id: 'b', n: 2 }];
    const el = await mount({ items });
    const idHeader = el.shadowRoot!.querySelector<HTMLElement>('th[data-sortable]')!;

    idHeader.click();
    await (el as any).updateComplete;
    expect(headerTexts(el)).toEqual(['a', 'b', 'c']);

    idHeader.click();
    await (el as any).updateComplete;
    expect(headerTexts(el)).toEqual(['c', 'b', 'a']);
  });

  it('does not mutate the input items array', async () => {
    const items = [{ id: 'c', n: 3 }, { id: 'a', n: 1 }];
    const original = [...items];
    const el = await mount({ items });
    el.shadowRoot!.querySelector<HTMLElement>('th[data-sortable]')!.click();
    await (el as any).updateComplete;
    expect(items).toEqual(original);
  });

  it('clicking a non-sortable header is a no-op', async () => {
    const items = [{ id: 'b', n: 2 }, { id: 'a', n: 1 }];
    const el = await mount({ items });
    const rawHeader = el.shadowRoot!.querySelectorAll('thead th')[2] as HTMLElement;
    expect(rawHeader.hasAttribute('data-sortable')).toBe(false);
    rawHeader.click();
    await (el as any).updateComplete;
    expect(headerTexts(el)).toEqual(['b', 'a']);
  });

  it('sorts null/undefined last regardless of direction', async () => {
    const withNulls: ListColumn<Row>[] = [
      { key: 'n', label: 'N',
        sortValue: r => (r.id === 'x' ? null : r.n),
        render: r => r.id },
    ];
    const el = document.createElement('mus-list-view') as any;
    el.columns = withNulls;
    el.items = [{ id: 'x', n: 99 }, { id: 'a', n: 1 }, { id: 'b', n: 2 }];
    document.body.appendChild(el);
    await el.updateComplete;

    el.shadowRoot!.querySelector('th[data-sortable]').click();
    await el.updateComplete;
    let ids = Array.from(el.shadowRoot!.querySelectorAll('tbody tr'))
      .map((r: any) => r.textContent.trim());
    expect(ids).toEqual(['a', 'b', 'x']);

    el.shadowRoot!.querySelector('th[data-sortable]').click();
    await el.updateComplete;
    ids = Array.from(el.shadowRoot!.querySelectorAll('tbody tr'))
      .map((r: any) => r.textContent.trim());
    expect(ids).toEqual(['b', 'a', 'x']);
  });
});

describe('mus-list-view drop', () => {
  beforeEach(() => { document.body.innerHTML = ''; });

  function fireDrop(el: Element, files: Array<{ name: string; path?: string }>) {
    const dt = { files };
    const ev = new Event('drop', { bubbles: true, cancelable: true });
    Object.defineProperty(ev, 'dataTransfer', { value: dt });
    el.dispatchEvent(ev);
    return ev;
  }

  it('dispatches mus-list-drop with paths when acceptDrop is true', async () => {
    const el = await mount({ items: [], acceptDrop: true });
    const spy = vi.fn();
    el.addEventListener('mus-list-drop', spy as EventListener);
    fireDrop(el, [{ name: 'a.wav' }, { name: 'b.wav' }]);
    expect(spy).toHaveBeenCalledTimes(1);
    expect((spy.mock.calls[0]![0] as CustomEvent).detail).toEqual({ paths: ['a.wav', 'b.wav'] });
  });

  it('does not dispatch when acceptDrop is false', async () => {
    const el = await mount({ items: [], acceptDrop: false });
    const spy = vi.fn();
    el.addEventListener('mus-list-drop', spy as EventListener);
    fireDrop(el, [{ name: 'a.wav' }]);
    expect(spy).not.toHaveBeenCalled();
  });

  it('adds and removes the drag-over host class on dragover / dragleave', async () => {
    const el = await mount({ items: [], acceptDrop: true });
    el.dispatchEvent(new Event('dragover', { bubbles: true, cancelable: true }));
    await (el as any).updateComplete;
    expect(el.classList.contains('drag-over')).toBe(true);
    el.dispatchEvent(new Event('dragleave', { bubbles: true, cancelable: true }));
    await (el as any).updateComplete;
    expect(el.classList.contains('drag-over')).toBe(false);
  });
});
