export function fmtDuration(seconds: number): string {
  const s = Math.max(0, Math.floor(seconds));
  const m = Math.floor(s / 60);
  return `${m}:${String(s % 60).padStart(2, '0')}`;
}

export function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function relativeFolder(filePath: string, libraryDir: string): string {
  if (!libraryDir) return '-';
  const dir = libraryDir.endsWith('/') ? libraryDir.slice(0, -1) : libraryDir;
  const lastSep = filePath.lastIndexOf('/');
  if (lastSep === -1) return '-';
  const parent = filePath.slice(0, lastSep);
  if (parent === dir) return '-';
  if (parent.startsWith(`${dir}/`)) return parent.slice(dir.length + 1);
  return '-';
}
