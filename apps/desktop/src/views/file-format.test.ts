import { describe, it, expect } from 'vitest';
import { fmtDuration, fmtSize, relativeFolder } from './file-format';

describe('fmtDuration', () => {
  it('formats seconds as m:ss', () => {
    expect(fmtDuration(0)).toBe('0:00');
    expect(fmtDuration(9)).toBe('0:09');
    expect(fmtDuration(65)).toBe('1:05');
    expect(fmtDuration(3600)).toBe('60:00');
  });

  it('floors fractional seconds', () => {
    expect(fmtDuration(65.9)).toBe('1:05');
  });

  it('clamps negatives to 0', () => {
    expect(fmtDuration(-5)).toBe('0:00');
  });
});

describe('fmtSize', () => {
  it('renders bytes below 1 KB as B', () => {
    expect(fmtSize(500)).toBe('500 B');
  });
  it('renders below 1 MB as KB with one decimal', () => {
    expect(fmtSize(2048)).toBe('2.0 KB');
  });
  it('renders 1 MB and up as MB with one decimal', () => {
    expect(fmtSize(3 * 1024 * 1024)).toBe('3.0 MB');
  });
});

describe('relativeFolder', () => {
  it('returns "-" when file is directly under library dir', () => {
    expect(relativeFolder('/lib/a.wav', '/lib')).toBe('-');
    expect(relativeFolder('/lib/a.wav', '/lib/')).toBe('-');
  });

  it('returns the sub-folder when nested', () => {
    expect(relativeFolder('/lib/sub/a.wav', '/lib')).toBe('sub');
    expect(relativeFolder('/lib/deep/nested/a.wav', '/lib')).toBe('deep/nested');
  });

  it('returns "-" when path is outside the library dir', () => {
    expect(relativeFolder('/other/a.wav', '/lib')).toBe('-');
  });

  it('returns "-" when library dir is empty', () => {
    expect(relativeFolder('/lib/a.wav', '')).toBe('-');
  });
});
