import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { FileListItem } from '../core-api/types';

const mockCoreApi = {
  getAppInfo: vi.fn(),
  getLibraryDir: vi.fn<() => Promise<string>>(),
  listPlugins: vi.fn(),
  listFiles: vi.fn<() => Promise<FileListItem[]>>(),
  listClips: vi.fn(),
};

vi.mock('../core-api', () => ({ coreApi: mockCoreApi }));

await import('./file-list-view');

function makeItem(name: string, clips = 0): FileListItem {
  return {
    file: {
      id: name, slug: name, name, path: `/tmp/${name}.wav`,
      duration: 65, sample_rate: 44100, channels: 2,
      mime_type: 'audio/wav', hash: '', mtime: '', size_bytes: 1024,
      created_at: '', updated_at: '',
    },
    clips: Array.from({ length: clips }, (_, i) => ({
      id: `c${i}`, slug: `c${i}`, file_id: name, title: `c${i}`,
      processors: null, duration: null, notes: '',
      created_at: '', updated_at: '',
    })),
  };
}

async function mount(): Promise<HTMLElement> {
  const el = document.createElement('mus-file-list-view');
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

describe('mus-file-list-view', () => {
  beforeEach(() => {
    mockCoreApi.listFiles.mockReset();
    mockCoreApi.getLibraryDir.mockReset();
    mockCoreApi.getLibraryDir.mockResolvedValue('/tmp');
    document.body.innerHTML = '';
    window.location.hash = '';
  });

  it('shows a loading placeholder before the promise resolves', async () => {
    mockCoreApi.listFiles.mockReturnValue(new Promise(() => {}));
    const el = await mount();
    expect(listRoot(el).textContent).toContain('Loading');
  });

  it('renders one row per file with clip count derived from clips.length', async () => {
    mockCoreApi.listFiles.mockResolvedValue([makeItem('alpha', 2), makeItem('beta', 0)]);
    const el = await mount();
    const rows = listRoot(el).querySelectorAll('tbody tr');
    expect(rows).toHaveLength(2);
    expect(rows[0]!.textContent).toContain('alpha');
    expect(rows[0]!.textContent).toContain('2');
    expect(rows[1]!.textContent).toContain('beta');
    expect(rows[1]!.textContent).toContain('0');
  });

  it('renders the empty state when no files', async () => {
    mockCoreApi.listFiles.mockResolvedValue([]);
    const el = await mount();
    expect(listRoot(el).textContent).toContain('No files yet');
  });

  it('renders the error state when the call rejects', async () => {
    mockCoreApi.listFiles.mockRejectedValue('boom');
    const el = await mount();
    expect(listRoot(el).textContent).toContain('boom');
  });

  it('navigates to #files/<slug> when a row is clicked', async () => {
    mockCoreApi.listFiles.mockResolvedValue([makeItem('alpha', 0)]);
    const el = await mount();
    const row = listRoot(el).querySelector('tbody tr.data-row') as HTMLElement;
    row.click();
    expect(window.location.hash).toBe('#files/alpha');
  });
});
