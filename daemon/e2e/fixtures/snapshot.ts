/**
 * Filesystem Snapshot Utility for E2E Testing
 *
 * Captures the state of the filesystem after each CLI command.
 * Supports comparing snapshots and serializing them for storage.
 */

import { readdir, readFile, stat, writeFile, mkdir } from 'node:fs/promises';
import { join, relative } from 'node:path';
import { createHash } from 'node:crypto';
import { existsSync } from 'node:fs';

/**
 * Represents a single file in a snapshot.
 */
export interface SnapshotFile {
  /** Relative path from project root */
  path: string;
  /** File type: 'file' or 'directory' */
  type: 'file' | 'directory';
  /** SHA-256 hash of file contents (only for files) */
  hash?: string;
  /** File size in bytes (only for files) */
  size?: number;
  /** File contents (only for text files, optional) */
  content?: string;
}

/**
 * Represents a complete filesystem snapshot.
 */
export interface Snapshot {
  /** Timestamp when snapshot was taken */
  timestamp: string;
  /** Root directory that was captured */
  rootPath: string;
  /** Descriptive label for this snapshot */
  label?: string;
  /** List of all files and directories */
  files: SnapshotFile[];
}

/**
 * Options for taking a snapshot.
 */
export interface SnapshotOptions {
  /** Root directory to snapshot */
  rootPath: string;
  /** Label for this snapshot (e.g., "after-init") */
  label?: string;
  /** Patterns to exclude (glob-like, simple matching) */
  exclude?: string[];
  /** Include file contents for text files */
  includeContents?: boolean;
  /** Maximum file size to include contents (default: 10KB) */
  maxContentSize?: number;
  /** Only snapshot the .centy directory */
  centyOnly?: boolean;
}

/**
 * Difference between two snapshots.
 */
export interface SnapshotDiff {
  /** Files added in the new snapshot */
  added: SnapshotFile[];
  /** Files removed from the old snapshot */
  removed: SnapshotFile[];
  /** Files that changed */
  modified: Array<{
    path: string;
    oldHash?: string;
    newHash?: string;
    oldContent?: string;
    newContent?: string;
  }>;
}

const DEFAULT_EXCLUDE = [
  'node_modules',
  '.git',
  '.DS_Store',
  'target',
  '*.log',
];

const TEXT_EXTENSIONS = [
  '.json',
  '.md',
  '.txt',
  '.yml',
  '.yaml',
  '.toml',
  '.ts',
  '.js',
  '.rs',
];

/**
 * Check if a path should be excluded based on patterns.
 */
function shouldExclude(path: string, patterns: string[]): boolean {
  for (const pattern of patterns) {
    if (pattern.startsWith('*.')) {
      // Extension pattern
      if (path.endsWith(pattern.slice(1))) {
        return true;
      }
    } else if (path.includes(pattern)) {
      // Simple contains match
      return true;
    }
  }
  return false;
}

/**
 * Check if a file is a text file based on extension.
 */
function isTextFile(path: string): boolean {
  return TEXT_EXTENSIONS.some((ext) => path.endsWith(ext));
}

/**
 * Compute SHA-256 hash of a buffer.
 */
function computeHash(content: Buffer): string {
  return createHash('sha256').update(content).digest('hex');
}

/**
 * Recursively collect all files in a directory.
 */
async function collectFiles(
  rootPath: string,
  currentPath: string,
  options: SnapshotOptions,
  files: SnapshotFile[]
): Promise<void> {
  const entries = await readdir(currentPath, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = join(currentPath, entry.name);
    const relativePath = relative(rootPath, fullPath);

    // Check exclusions
    const exclude = options.exclude ?? DEFAULT_EXCLUDE;
    if (shouldExclude(relativePath, exclude)) {
      continue;
    }

    // If centyOnly, only include .centy directory and its contents
    if (options.centyOnly) {
      const isCentyPath = relativePath === '.centy' || relativePath.startsWith('.centy/');
      if (!isCentyPath) {
        continue;
      }
    }

    if (entry.isDirectory()) {
      files.push({
        path: relativePath,
        type: 'directory',
      });
      await collectFiles(rootPath, fullPath, options, files);
    } else if (entry.isFile()) {
      const content = await readFile(fullPath);
      const fileStats = await stat(fullPath);
      const hash = computeHash(content);

      const file: SnapshotFile = {
        path: relativePath,
        type: 'file',
        hash,
        size: fileStats.size,
      };

      // Include content for text files if requested
      if (
        options.includeContents &&
        isTextFile(relativePath) &&
        fileStats.size <= (options.maxContentSize ?? 10240)
      ) {
        file.content = content.toString('utf-8');
      }

      files.push(file);
    }
  }
}

/**
 * Take a snapshot of a directory.
 */
export async function takeSnapshot(options: SnapshotOptions): Promise<Snapshot> {
  const files: SnapshotFile[] = [];

  if (existsSync(options.rootPath)) {
    await collectFiles(options.rootPath, options.rootPath, options, files);
  }

  // Sort files by path for consistent ordering
  files.sort((a, b) => a.path.localeCompare(b.path));

  return {
    timestamp: new Date().toISOString(),
    rootPath: options.rootPath,
    label: options.label,
    files,
  };
}

/**
 * Compare two snapshots and return the differences.
 */
export function compareSnapshots(
  oldSnapshot: Snapshot,
  newSnapshot: Snapshot
): SnapshotDiff {
  const oldMap = new Map(oldSnapshot.files.map((f) => [f.path, f]));
  const newMap = new Map(newSnapshot.files.map((f) => [f.path, f]));

  const added: SnapshotFile[] = [];
  const removed: SnapshotFile[] = [];
  const modified: SnapshotDiff['modified'] = [];

  // Find added and modified files
  for (const [path, newFile] of newMap) {
    const oldFile = oldMap.get(path);
    if (!oldFile) {
      added.push(newFile);
    } else if (newFile.type === 'file' && oldFile.hash !== newFile.hash) {
      modified.push({
        path,
        oldHash: oldFile.hash,
        newHash: newFile.hash,
        oldContent: oldFile.content,
        newContent: newFile.content,
      });
    }
  }

  // Find removed files
  for (const [path, oldFile] of oldMap) {
    if (!newMap.has(path)) {
      removed.push(oldFile);
    }
  }

  return { added, removed, modified };
}

/**
 * Check if a snapshot matches expected files.
 */
export function snapshotContains(
  snapshot: Snapshot,
  expectedFiles: string[]
): { found: string[]; missing: string[] } {
  const paths = new Set(snapshot.files.map((f) => f.path));
  const found: string[] = [];
  const missing: string[] = [];

  for (const expected of expectedFiles) {
    if (paths.has(expected)) {
      found.push(expected);
    } else {
      missing.push(expected);
    }
  }

  return { found, missing };
}

/**
 * Save a snapshot to a file.
 */
export async function saveSnapshot(
  snapshot: Snapshot,
  filePath: string
): Promise<void> {
  const dir = filePath.substring(0, filePath.lastIndexOf('/'));
  if (dir) {
    await mkdir(dir, { recursive: true });
  }
  await writeFile(filePath, JSON.stringify(snapshot, null, 2), 'utf-8');
}

/**
 * Load a snapshot from a file.
 */
export async function loadSnapshot(filePath: string): Promise<Snapshot> {
  const content = await readFile(filePath, 'utf-8');
  return JSON.parse(content) as Snapshot;
}

/**
 * Format a snapshot diff for display.
 */
export function formatDiff(diff: SnapshotDiff): string {
  const lines: string[] = [];

  if (diff.added.length > 0) {
    lines.push('Added files:');
    for (const file of diff.added) {
      lines.push(`  + ${file.path}`);
    }
  }

  if (diff.removed.length > 0) {
    lines.push('Removed files:');
    for (const file of diff.removed) {
      lines.push(`  - ${file.path}`);
    }
  }

  if (diff.modified.length > 0) {
    lines.push('Modified files:');
    for (const file of diff.modified) {
      lines.push(`  ~ ${file.path}`);
    }
  }

  if (lines.length === 0) {
    lines.push('No changes');
  }

  return lines.join('\n');
}

/**
 * Get a summary of a snapshot.
 */
export function getSnapshotSummary(snapshot: Snapshot): {
  totalFiles: number;
  totalDirectories: number;
  totalSize: number;
  filesByExtension: Record<string, number>;
} {
  let totalFiles = 0;
  let totalDirectories = 0;
  let totalSize = 0;
  const filesByExtension: Record<string, number> = {};

  for (const file of snapshot.files) {
    if (file.type === 'directory') {
      totalDirectories++;
    } else {
      totalFiles++;
      totalSize += file.size ?? 0;

      const ext = file.path.includes('.')
        ? '.' + file.path.split('.').pop()
        : '(no extension)';
      filesByExtension[ext] = (filesByExtension[ext] ?? 0) + 1;
    }
  }

  return { totalFiles, totalDirectories, totalSize, filesByExtension };
}

/**
 * Snapshot manager for test context.
 * Helps manage multiple snapshots during a test run.
 */
export class SnapshotManager {
  private snapshots: Map<string, Snapshot> = new Map();
  private rootPath: string;
  private centyOnly: boolean;
  private outputDir?: string;

  constructor(options: {
    rootPath: string;
    centyOnly?: boolean;
    outputDir?: string;
  }) {
    this.rootPath = options.rootPath;
    this.centyOnly = options.centyOnly ?? true;
    this.outputDir = options.outputDir ?? process.env.E2E_SNAPSHOT_DIR;
  }

  /**
   * Take a named snapshot.
   */
  async take(label: string, includeContents = true): Promise<Snapshot> {
    const snapshot = await takeSnapshot({
      rootPath: this.rootPath,
      label,
      centyOnly: this.centyOnly,
      includeContents,
    });

    this.snapshots.set(label, snapshot);

    // Optionally save to disk
    if (this.outputDir) {
      const filename = `${label.replace(/[^a-z0-9]/gi, '-')}.json`;
      await saveSnapshot(snapshot, join(this.outputDir, filename));
    }

    return snapshot;
  }

  /**
   * Get a previously taken snapshot.
   */
  get(label: string): Snapshot | undefined {
    return this.snapshots.get(label);
  }

  /**
   * Compare two snapshots by label.
   */
  compare(oldLabel: string, newLabel: string): SnapshotDiff {
    const oldSnapshot = this.snapshots.get(oldLabel);
    const newSnapshot = this.snapshots.get(newLabel);

    if (!oldSnapshot) {
      throw new Error(`Snapshot not found: ${oldLabel}`);
    }
    if (!newSnapshot) {
      throw new Error(`Snapshot not found: ${newLabel}`);
    }

    return compareSnapshots(oldSnapshot, newSnapshot);
  }

  /**
   * Assert that a file exists in the latest snapshot.
   */
  assertFileExists(label: string, filePath: string): boolean {
    const snapshot = this.snapshots.get(label);
    if (!snapshot) {
      throw new Error(`Snapshot not found: ${label}`);
    }
    return snapshot.files.some((f) => f.path === filePath);
  }

  /**
   * Get file content from a snapshot.
   */
  getFileContent(label: string, filePath: string): string | undefined {
    const snapshot = this.snapshots.get(label);
    if (!snapshot) {
      throw new Error(`Snapshot not found: ${label}`);
    }
    const file = snapshot.files.find((f) => f.path === filePath);
    return file?.content;
  }

  /**
   * Clear all snapshots.
   */
  clear(): void {
    this.snapshots.clear();
  }

  /**
   * Get file tree structure for Vitest snapshot matching.
   * Returns a stable, serializable structure suitable for toMatchSnapshot().
   */
  getFileTree(label: string): FileTree {
    const snapshot = this.snapshots.get(label);
    if (!snapshot) {
      throw new Error(`Snapshot not found: ${label}`);
    }
    return snapshotToFileTree(snapshot);
  }

  /**
   * Get diff summary for Vitest snapshot matching.
   * Returns a stable, serializable structure suitable for toMatchSnapshot().
   */
  getDiffSummary(oldLabel: string, newLabel: string): DiffSummary {
    const diff = this.compare(oldLabel, newLabel);
    return diffToSummary(diff);
  }
}

/**
 * Stable file tree structure for Vitest snapshots.
 */
export interface FileTree {
  /** Sorted list of all file paths */
  files: string[];
  /** Sorted list of all directory paths */
  directories: string[];
  /** File count by extension */
  byExtension: Record<string, string[]>;
}

/**
 * Stable diff summary for Vitest snapshots.
 */
export interface DiffSummary {
  /** Sorted list of added file paths */
  added: string[];
  /** Sorted list of removed file paths */
  removed: string[];
  /** Sorted list of modified file paths */
  modified: string[];
}

/**
 * Convert a snapshot to a stable file tree for Vitest matching.
 */
export function snapshotToFileTree(snapshot: Snapshot): FileTree {
  const files: string[] = [];
  const directories: string[] = [];
  const byExtension: Record<string, string[]> = {};

  for (const file of snapshot.files) {
    if (file.type === 'directory') {
      directories.push(file.path);
    } else {
      files.push(file.path);
      const ext = file.path.includes('.')
        ? '.' + file.path.split('.').pop()
        : '(no extension)';
      if (!byExtension[ext]) {
        byExtension[ext] = [];
      }
      byExtension[ext].push(file.path);
    }
  }

  // Sort all arrays for stable snapshots
  files.sort();
  directories.sort();
  for (const ext of Object.keys(byExtension)) {
    byExtension[ext].sort();
  }

  return { files, directories, byExtension };
}

/**
 * Convert a diff to a stable summary for Vitest matching.
 */
export function diffToSummary(diff: SnapshotDiff): DiffSummary {
  return {
    added: diff.added.map((f) => f.path).sort(),
    removed: diff.removed.map((f) => f.path).sort(),
    modified: diff.modified.map((f) => f.path).sort(),
  };
}
