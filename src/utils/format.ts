// 字节大小的人类可读格式化工具(与后端 fsutil::format_size 对齐)。
// 1024 进位,GB 及以上保留两位小数,MB/KB 取整,不足 1 KB 直接显示字节数。

const KB = 1024;
const MB = KB * 1024;
const GB = MB * 1024;
const TB = GB * 1024;

/**
 * 将字节数转换为人类可读字符串。
 *
 * - `>= 1 TB`:`x.xx TB`(两位小数)
 * - `>= 1 GB`:`x.xx GB`(两位小数)
 * - `>= 1 MB`:`x MB`(整数)
 * - `>= 1 KB`:`x KB`(整数)
 * - 否则:`x B`
 *
 * 与后端 `fsutil::format_size` 输出一致,便于前后端展示对齐。
 */
export function formatSize(bytes: number): string {
  if (bytes >= TB) return `${(bytes / TB).toFixed(2)} TB`;
  if (bytes >= GB) return `${(bytes / GB).toFixed(2)} GB`;
  if (bytes >= MB) return `${Math.round(bytes / MB)} MB`;
  if (bytes >= KB) return `${Math.round(bytes / KB)} KB`;
  return `${bytes} B`;
}
