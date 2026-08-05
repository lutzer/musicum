import { describe, it, expect, beforeEach } from 'vitest';
import { ViewRegistry, SlotRegistry } from './registry';

describe('ViewRegistry', () => {
  let r: ViewRegistry;
  beforeEach(() => { r = new ViewRegistry(); });

  it('adds and lists views', () => {
    r.register('bundle-a', { id: 'a.hello', title: 'Hello', element: 'x-hello' });
    expect(r.list()).toHaveLength(1);
    expect(r.get('a.hello')?.title).toBe('Hello');
  });

  it('rejects duplicate view ids and keeps the first', () => {
    r.register('bundle-a', { id: 'a', title: 'First', element: 'x1' });
    r.register('bundle-b', { id: 'a', title: 'Second', element: 'x2' });
    expect(r.list()).toHaveLength(1);
    expect(r.get('a')?.title).toBe('First');
  });

  it('unregisters all views from a bundle', () => {
    r.register('bundle-a', { id: 'a.one', title: 'One', element: 'x1' });
    r.register('bundle-a', { id: 'a.two', title: 'Two', element: 'x2' });
    r.register('bundle-b', { id: 'b.one', title: 'B', element: 'y1' });
    r.unregisterBundle('bundle-a');
    expect(r.list().map(v => v.id)).toEqual(['b.one']);
  });

  it('notifies subscribers on change', () => {
    let count = 0;
    r.subscribe(() => count++);
    r.register('bundle-a', { id: 'x', title: 'X', element: 'x' });
    expect(count).toBe(1);
  });

  it('preserves the sidebar flag on registered views', () => {
    r.register('bundle-a', { id: 'a', title: 'A', element: 'x-a' });
    r.register('bundle-a', { id: 'b', title: 'B', element: 'x-b', sidebar: false });
    expect(r.get('a')?.sidebar).toBeUndefined();
    expect(r.get('b')?.sidebar).toBe(false);
  });
});

describe('SlotRegistry', () => {
  let r: SlotRegistry;
  beforeEach(() => { r = new SlotRegistry(); });

  it('sorts entries by order ascending', () => {
    r.register('b1', 'slot-a', { id: 'z', element: 'x-z', order: 200 });
    r.register('b1', 'slot-a', { id: 'a', element: 'x-a', order: 50 });
    r.register('b1', 'slot-a', { id: 'm', element: 'x-m' });
    expect(r.entries('slot-a').map(e => e.id)).toEqual(['a', 'm', 'z']);
  });

  it('rejects duplicate entry ids within a slot', () => {
    r.register('b1', 'slot-a', { id: 'x', element: 'x-1' });
    r.register('b2', 'slot-a', { id: 'x', element: 'x-2' });
    expect(r.entries('slot-a')).toHaveLength(1);
  });

  it('returns empty for unknown slot', () => {
    expect(r.entries('nope')).toEqual([]);
  });

  it('unregisters all slot entries from a bundle', () => {
    r.register('b1', 's1', { id: 'a', element: 'x-a' });
    r.register('b1', 's2', { id: 'b', element: 'x-b' });
    r.register('b2', 's1', { id: 'c', element: 'x-c' });
    r.unregisterBundle('b1');
    expect(r.entries('s1').map(e => e.id)).toEqual(['c']);
    expect(r.entries('s2')).toEqual([]);
  });
});
