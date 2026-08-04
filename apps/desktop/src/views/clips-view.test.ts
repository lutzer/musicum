import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { ClipListItem } from '../core-api/types';

const mockCoreApi = {
  getAppInfo: vi.fn(),
  listPlugins: vi.fn(),
  listFiles: vi.fn(),
  listClips: vi.fn<() => Promise<ClipListItem[]>>(),
};

vi.mock('../core-api', () => ({ coreApi: mockCoreApi }));

await import('./clips-view');

function makeItem(title: string, duration: number | null, sourceName = 'src.wav'): ClipListItem {
  return {
    clip: {
      id: title, slug: title, file_id: 'f1', title, processors: null,
      duration, notes: '', created_at: '', updated_at: '',
    },
    file: {
      id: 'f1', slug: 'f1', name: sourceName, path: `/tmp/${sourceName}`,
      duration: 10, sample_rate: 44100, channels: 2,
      mime_type: 'audio/wav', hash: '', mtime: '', size_bytes: 0,
      created_at: '', updated_at: '',
    },
  };
}

async function mount(): Promise<HTMLElement> {
  const el = document.createElement('mus-clips-view');
  document.body.appendChild(el);
  await (el as any).updateComplete;
  await new Promise(r => setTimeout(r, 0));
  await (el as any).updateComplete;
  const list = el.shadowRoot!.querySelector('mus-list-view')!;
  await (list as any).updateComplete;
  return el;
}

function listRoot(el: HTMLElement): ShadowRoot {
  return el.shadowRoot!.querySelector('mus-list-view')!.shadowRoot!;
}

describe('mus-clips-view', () => {
  beforeEach(() => {
    mockCoreApi.listClips.mockReset();
    document.body.innerHTML = '';
  });

  it('shows loading before the promise resolves', async () => {
    mockCoreApi.listClips.mockReturnValue(new Promise(() => {}));
    const el = await mount();
    expect(listRoot(el).textContent).toContain('Loading');
  });

  it('renders one row per clip with source file name', async () => {
    mockCoreApi.listClips.mockResolvedValue([
      makeItem('kick', 1.5, 'drums.wav'),
      makeItem('snare', null, 'drums.wav'),
    ]);
    const el = await mount();
    const rows = listRoot(el).querySelectorAll('tbody tr');
    expect(rows).toHaveLength(2);
    expect(rows[0]!.textContent).toContain('kick');
    expect(rows[0]!.textContent).toContain('drums.wav');
    expect(rows[1]!.textContent).toContain('—');
  });

  it('renders the empty state when no clips', async () => {
    mockCoreApi.listClips.mockResolvedValue([]);
    const el = await mount();
    expect(listRoot(el).textContent).toContain('No clips yet');
  });

  it('renders the error state when the call rejects', async () => {
    mockCoreApi.listClips.mockRejectedValue('kaboom');
    const el = await mount();
    expect(listRoot(el).textContent).toContain('kaboom');
  });
});
