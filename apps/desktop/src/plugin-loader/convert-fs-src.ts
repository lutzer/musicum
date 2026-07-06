import { convertFileSrc as tauriConvertFileSrc } from '@tauri-apps/api/core';

export function convertFileSrc(path: string): string {
  return tauriConvertFileSrc(path);
}
