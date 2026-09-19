import { describe, expect, it } from 'vitest';
import { formatSize } from '@/utils/format';

// 与后端 fsutil::format_size 对齐:1024 进位,GB 及以上保留两位小数,MB/KB 取整。
describe('formatSize', () => {
  it('formats 0 bytes as "0 B"', () => {
    expect(formatSize(0)).toBe('0 B');
  });

  it('formats bytes below 1 KB as raw bytes', () => {
    expect(formatSize(1023)).toBe('1023 B');
  });

  it('formats exactly 1 KB as "1 KB"', () => {
    expect(formatSize(1024)).toBe('1 KB');
  });

  it('rounds 1536 bytes to the nearest KB (1.5 -> 2)', () => {
    // 实现使用 Math.round:1536 / 1024 = 1.5 -> 2 KB(非 1.5 MB)。
    expect(formatSize(1536)).toBe('2 KB');
  });

  it('formats exactly 1 MB as "1 MB"', () => {
    expect(formatSize(1048576)).toBe('1 MB');
  });

  it('formats exactly 1 GB with two decimals as "1.00 GB"', () => {
    // GB 及以上分支使用 toFixed(2),故 1 GB -> "1.00 GB"。
    expect(formatSize(1073741824)).toBe('1.00 GB');
  });

  it('formats a fractional GB value with two decimals', () => {
    // 1328165593.6 / 1024^3 ≈ 1.2369 -> toFixed(2) -> "1.24 GB"。
    expect(formatSize(1328165593.6)).toBe('1.24 GB');
  });
});
