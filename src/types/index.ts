// Tauri 命令返回的共享类型(Rust 侧 serde 序列化后 camelCase 对应)。

/// 安全等级(kebab-case 序列化)。
export type SafetyLevel = 'safe' | 'caution' | 'danger';

/// 单个扫描结果项。
export interface ScanItem {
  id: string;
  path: string;
  sizeBytes: number;
  modifiedAt: number;
  safety: SafetyLevel;
  category: string;
  label: string | null;
}

/// 一次扫描的汇总结果。
export interface ScanResult {
  items: ScanItem[];
  totalSizeBytes: number;
  scannedPaths: number;
}

/// 重复文件组:相同 SHA-256 的文件集合。
export interface DuplicateGroup {
  hash: string;
  sizeBytes: number;
  files: ScanItem[];
}

/// 隐私清理扫描项。
export interface PrivacyTarget {
  id: string;
  browser: string;
  label: string;
  path: string;
  sizeBytes: number;
  isInstalled: boolean;
  isRunning: boolean;
  safety: SafetyLevel;
}

/// 浏览器运行状态。
export interface BrowserStatus {
  browser: string;
  isRunning: boolean;
}

/// 单个扫描结果项(Task 4 / 6 通用)。
export interface ScanItem {
  id: string;
  path: string;
  sizeBytes: number;
  modifiedAt: number;
  safety: SafetyLevel;
  category: string;
  label: string | null;
}

/// 一次扫描的汇总结果(Task 4 / 6)。
export interface ScanResult {
  items: ScanItem[];
  totalSizeBytes: number;
  scannedPaths: number;
}

/// 重复文件组(Task 6):相同 SHA-256 的文件集合。
export interface DuplicateGroup {
  hash: string;
  sizeBytes: number;
  files: ScanItem[];
}

/// 应用卸载器:单个关联路径(Task 5)。
export interface RelatedPath {
  /// 关联路径(绝对路径)。
  path: string;
  /// 占用字节数(目录递归累加,不存在则为 0)。
  sizeBytes: number;
  /// 路径是否存在。
  exists: boolean;
  /// 安全等级。
  safety: SafetyLevel;
}

/// 应用卸载器:扫描到的单个 .app bundle(Task 5)。
export interface AppEntry {
  /// 应用名(.app 文件名去扩展名)。
  name: string;
  /// .app bundle 绝对路径。
  path: string;
  /// CFBundleIdentifier(解析失败则为 null)。
  bundleId: string | null;
  /// 应用本体大小(.app bundle 递归累加字节数)。
  sizeBytes: number;
  /// 关联文件清单(8 个标准位置)。
  related: RelatedPath[];
  /// 是否为系统应用(/System/Applications 下,只读列出)。
  isSystem: boolean;
  /// 应用是否正在运行。
  isRunning: boolean;
}

/// 清理成功的单项结果。
export interface CleanedItem {
  id: string;
  path: string;
  sizeBytes: number;
}

/// 清理失败的单项结果。
export interface FailedItem {
  id: string;
  path: string;
  reason: string;
}

/// 一次清理操作的汇总结果。
export interface CleanOutcome {
  success: CleanedItem[];
  failed: FailedItem[];
}

/// 清理历史单条记录(Task 8.4):与后端 CleanHistoryEntry 对齐(camelCase)。
export interface CleanHistoryEntry {
  /// 发生时间(unix 秒)。
  timestamp: number;
  /// 来源模块名,如 `system_junk`。
  module: string;
  /// 释放字节数。
  freedBytes: number;
  /// 成功项数。
  successCount: number;
  /// 失败项数。
  failedCount: number;
}

/// 应用配置(Task 9.3):与后端 AppConfig 对齐(camelCase),落盘 `~/.macmate/config.json`。
export interface AppConfig {
  /// 界面语言:`zh-CN` / `en-US`。
  locale: string;
  /// 主题模式:`system` / `light` / `dark`。
  theme: string;
  /// 是否开启深度清理。
  deepClean: boolean;
  /// 大文件扫描阈值(MB)。
  largeFileThresholdMb: number;
}
