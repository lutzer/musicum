import { describe, it, expect, beforeEach, vi } from 'vitest';
import type { FileListItem, FileMetadataModel } from '../core-api/types';

const mockCoreApi = {
  getAppInfo: vi.fn(),
  getLibraryDir: vi.fn<() => Promise<string>>(),
  listPlugins: vi.fn(),
  listFiles: vi.fn(),
  listClips: vi.fn(),
  getFileBySlug:   vi.fn<(slug: string) => Promise<FileListItem>>(),
  getFileMetadata: vi.fn<(fileId: string) => Promise<FileMetadataModel | null>>(),
  setFileNotes:    vi.fn<(slug: string, notes: string) => Promise<void>>(),
  setFileTags:     vi.fn<(slug: string, tags: string) => Promise<void>>(),
};
vi.mock('../core-api', () => ({ coreApi: mockCoreApi }));

await import('./file-detail-view');

function makeItem(slug: string, clips = 0): FileListItem {
  return {
    file: {
      id: `id-${slug}`, slug, name: slug, path: `/lib/${slug}.wav`,
      duration: 65, sample_rate: 44100, channels: 2,
      mime_type: 'audio/wav', hash: 'abcdef0123456789', mtime: '',
      size_bytes: 1024, created_at: '', updated_at: '',
    },
    clips: Array.from({ length: clips }, (_, i) => ({
      id: `c${i}`, slug: `c${i}`, file_id: `id-${slug}`, title: `Clip ${i}`,
      processors: null, duration: null, notes: '',
      created_at: '', updated_at: '',
    })),
  };
}

async function mount(slug = 'alpha'): Promise<HTMLElement> {
  const el = document.createElement('mus-file-detail-view');
  (el as any).slug = slug;
  document.body.appendChild(el);
  await (el as any).updateComplete;
  await new Promise(r => setTimeout(r, 0));
  await (el as any).updateComplete;
  return el;
}

describe('mus-file-detail-view', () => {
  beforeEach(() => {
    Object.values(mockCoreApi).forEach(fn => (fn as any).mockReset?.());
    mockCoreApi.getLibraryDir.mockResolvedValue('/lib');
    mockCoreApi.getFileBySlug.mockResolvedValue(makeItem('alpha', 1));
    mockCoreApi.getFileMetadata.mockResolvedValue({
      file_id: 'id-alpha', bpm: null, key: null, rating: null, color: null,
      notes: 'initial notes', tags: 'kick,drums',
    });
    document.body.innerHTML = '';
  });

  it('shows a loading placeholder before promises resolve', async () => {
    mockCoreApi.getFileBySlug.mockReturnValue(new Promise(() => {}));
    const el = await mount();
    expect(el.shadowRoot!.textContent).toContain('Loading');
  });

  it('renders metadata + editable notes + tags after load', async () => {
    const el = await mount();
    const root = el.shadowRoot!;
    expect(root.textContent).toContain('alpha');
    const notes = root.querySelector('textarea') as HTMLTextAreaElement;
    const tags  = root.querySelector('input[data-field="tags"]') as HTMLInputElement;
    expect(notes.value).toBe('initial notes');
    expect(tags.value).toBe('kick,drums');
  });

  it('renders "No clips for this file yet." when clips is empty', async () => {
    mockCoreApi.getFileBySlug.mockResolvedValue(makeItem('alpha', 0));
    const el = await mount();
    // mus-list-view empty state text
    const listRoot = el.shadowRoot!.querySelector('mus-list-view')!.shadowRoot!;
    expect(listRoot.textContent).toContain('No clips for this file yet.');
  });

  it('saves notes on blur when they changed', async () => {
    const el = await mount();
    const notes = el.shadowRoot!.querySelector('textarea') as HTMLTextAreaElement;
    notes.value = 'edited';
    notes.dispatchEvent(new Event('input'));
    notes.dispatchEvent(new Event('blur'));
    await new Promise(r => setTimeout(r, 0));
    expect(mockCoreApi.setFileNotes).toHaveBeenCalledWith('alpha', 'edited');
  });

  it('does not save notes on blur when unchanged', async () => {
    const el = await mount();
    const notes = el.shadowRoot!.querySelector('textarea') as HTMLTextAreaElement;
    notes.dispatchEvent(new Event('blur'));
    await new Promise(r => setTimeout(r, 0));
    expect(mockCoreApi.setFileNotes).not.toHaveBeenCalled();
  });

  it('saves tags on blur when they changed', async () => {
    const el = await mount();
    const tags = el.shadowRoot!.querySelector('input[data-field="tags"]') as HTMLInputElement;
    tags.value = 'kick,drums,808';
    tags.dispatchEvent(new Event('input'));
    tags.dispatchEvent(new Event('blur'));
    await new Promise(r => setTimeout(r, 0));
    expect(mockCoreApi.setFileTags).toHaveBeenCalledWith('alpha', 'kick,drums,808');
  });

  it('refetches when the slug property changes', async () => {
    const el = await mount('alpha');
    mockCoreApi.getFileBySlug.mockClear();
    mockCoreApi.getFileBySlug.mockResolvedValue(makeItem('beta', 0));
    (el as any).slug = 'beta';
    await (el as any).updateComplete;
    await new Promise(r => setTimeout(r, 0));
    await (el as any).updateComplete;
    expect(mockCoreApi.getFileBySlug).toHaveBeenCalledWith('beta');
    expect(el.shadowRoot!.textContent).toContain('beta');
  });

  it('renders an error message when the load rejects', async () => {
    mockCoreApi.getFileBySlug.mockRejectedValue('boom');
    const el = await mount();
    expect(el.shadowRoot!.textContent).toContain('boom');
  });

  it('treats null metadata as empty notes and tags', async () => {
    mockCoreApi.getFileMetadata.mockResolvedValue(null);
    const el = await mount();
    const notes = el.shadowRoot!.querySelector('textarea') as HTMLTextAreaElement;
    const tags  = el.shadowRoot!.querySelector('input[data-field="tags"]') as HTMLInputElement;
    expect(notes.value).toBe('');
    expect(tags.value).toBe('');
  });
});
