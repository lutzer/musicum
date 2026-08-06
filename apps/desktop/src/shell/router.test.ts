import { describe, it, expect, beforeEach } from 'vitest';
import { Router } from './router';
import { ViewRegistry } from '../plugin-api/registry';

describe('Router.parse', () => {
  const r = new Router();

  it('returns undefined for empty hash', () => {
    expect(r.parse('')).toBeUndefined();
    expect(r.parse('#')).toBeUndefined();
  });

  it('parses a bare view id', () => {
    expect(r.parse('#files')).toEqual({ viewId: 'files' });
  });

  it('parses a view id with param', () => {
    expect(r.parse('#files/abc')).toEqual({ viewId: 'files', param: 'abc' });
  });

  it('normalizes trailing slash to no-param', () => {
    expect(r.parse('#files/')).toEqual({ viewId: 'files' });
  });

  it('keeps everything after the first slash as the param', () => {
    expect(r.parse('#files/a/b/c')).toEqual({ viewId: 'files', param: 'a/b/c' });
  });
});

describe('Router.hashFor', () => {
  const r = new Router();

  it('builds the list hash when no param', () => {
    expect(r.hashFor('files')).toBe('#files');
    expect(r.hashFor('files', '')).toBe('#files');
  });

  it('builds the detail hash when param is present', () => {
    expect(r.hashFor('files', 'abc')).toBe('#files/abc');
  });

  it('round-trips with parse', () => {
    const r2 = new Router();
    const cases: Array<[string, string | undefined]> = [
      ['files', undefined],
      ['files', 'abc'],
      ['clips', 'a/b'],
    ];
    for (const [id, param] of cases) {
      const parsed = r2.parse(r2.hashFor(id, param));
      expect(parsed).toEqual(param === undefined ? { viewId: id } : { viewId: id, param });
    }
  });
});

describe('Router.activeViewId + current + navigate', () => {
  const r = new Router();

  beforeEach(() => {
    window.location.hash = '';
  });

  it('returns undefined when hash is empty', () => {
    expect(r.activeViewId()).toBeUndefined();
    expect(r.current()).toBeUndefined();
  });

  it('returns the parent segment for a detail route', () => {
    window.location.hash = '#files/abc';
    expect(r.activeViewId()).toBe('files');
    expect(r.current()).toEqual({ viewId: 'files', param: 'abc' });
  });

  it('navigate sets window.location.hash', () => {
    r.navigate('files', 'abc');
    expect(window.location.hash).toBe('#files/abc');
    r.navigate('welcome');
    expect(window.location.hash).toBe('#welcome');
  });

  it('ensureDefault only navigates when hash is empty', () => {
    window.location.hash = '';
    r.ensureDefault();
    expect(window.location.hash).toBe('#welcome');

    window.location.hash = '#files';
    r.ensureDefault();
    expect(window.location.hash).toBe('#files');
  });
});

describe('Router.resolve', () => {
  const r = new Router();
  let registry: ViewRegistry;

  beforeEach(() => {
    registry = new ViewRegistry();
    registry.register('t', { id: 'files', title: 'Files', element: 'mus-file-list-view' });
    registry.register('t', {
      id: 'files-detail', title: 'File', element: 'mus-file-detail-view', sidebar: false,
    });
  });

  it('resolves a list route to the registered element', () => {
    expect(r.resolve({ viewId: 'files' }, registry)).toEqual({
      kind: 'view', element: 'mus-file-list-view',
    });
  });

  it('resolves a detail route via the -detail suffix', () => {
    expect(r.resolve({ viewId: 'files', param: 'abc' }, registry)).toEqual({
      kind: 'view', element: 'mus-file-detail-view', param: 'abc',
    });
  });

  it('returns missing for an unknown view', () => {
    expect(r.resolve({ viewId: 'nope' }, registry)).toEqual({
      kind: 'missing', viewId: 'nope',
    });
  });

  it('returns missing when the detail view is unregistered', () => {
    (registry as any).views.delete('files-detail');
    expect(r.resolve({ viewId: 'files', param: 'abc' }, registry)).toEqual({
      kind: 'missing', viewId: 'files', param: 'abc',
    });
  });
});

describe('Router.subscribe', () => {
  const r = new Router();

  beforeEach(() => {
    window.location.hash = '';
  });

  it('fires the handler on hashchange and returns an unsubscribe fn', () => {
    const calls: Array<ReturnType<Router['current']>> = [];
    const off = r.subscribe(route => calls.push(route));

    window.location.hash = '#files';
    window.dispatchEvent(new HashChangeEvent('hashchange'));
    expect(calls).toEqual([{ viewId: 'files' }]);

    off();
    window.location.hash = '#clips';
    window.dispatchEvent(new HashChangeEvent('hashchange'));
    expect(calls).toEqual([{ viewId: 'files' }]);
  });
});
