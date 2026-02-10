import { mkdir, rm, readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { execSync } from 'node:child_process';
import { createGrpcClient, promisifyClient, type PromisifiedCentyClient } from './grpc-client.js';

export interface TempProject {
  /** Absolute path to the temp project directory */
  path: string;
  /** Path to the .centy folder */
  centyPath: string;
  /** Promisified gRPC client */
  client: PromisifiedCentyClient;
  /** Clean up the temp project */
  cleanup: () => Promise<void>;
}

export interface TempProjectOptions {
  /** Daemon address to use */
  daemonAddress?: string;
  /** Whether to initialize the .centy folder */
  initialize?: boolean;
  /** Prefix for the temp directory name */
  prefix?: string;
}

const DEFAULT_OPTIONS: Required<TempProjectOptions> = {
  daemonAddress: '127.0.0.1:50051',
  initialize: true,
  prefix: 'centy-test',
};

/**
 * Create a temporary project directory for E2E testing.
 * Optionally initializes the .centy folder.
 */
export async function createTempProject(
  options: TempProjectOptions = {}
): Promise<TempProject> {
  const opts = { ...DEFAULT_OPTIONS, ...options };

  // Create unique temp directory
  const projectPath = join(tmpdir(), `${opts.prefix}-${randomUUID()}`);
  await mkdir(projectPath, { recursive: true });

  const centyPath = join(projectPath, '.centy');
  const { centyClient, configClient } = createGrpcClient(opts.daemonAddress);
  const client = promisifyClient(centyClient, configClient);

  // Initialize if requested
  if (opts.initialize) {
    const result = await client.init({
      projectPath,
      force: true,
    });

    if (!result.success) {
      await rm(projectPath, { recursive: true, force: true });
      throw new Error(`Failed to initialize temp project: ${result.error}`);
    }
  }

  return {
    path: projectPath,
    centyPath,
    client,
    cleanup: async () => {
      client.close();
      await rm(projectPath, { recursive: true, force: true });
    },
  };
}

/**
 * Helper to read a file from the temp project.
 */
export async function readProjectFile(
  project: TempProject,
  relativePath: string
): Promise<string> {
  return readFile(join(project.path, relativePath), 'utf-8');
}

/**
 * Helper to write a file to the temp project.
 */
export async function writeProjectFile(
  project: TempProject,
  relativePath: string,
  content: string
): Promise<void> {
  const fullPath = join(project.path, relativePath);
  const dir = fullPath.substring(0, fullPath.lastIndexOf('/'));
  await mkdir(dir, { recursive: true });
  await writeFile(fullPath, content, 'utf-8');
}

/**
 * Check if a file exists in the temp project.
 */
export function projectFileExists(
  project: TempProject,
  relativePath: string
): boolean {
  return existsSync(join(project.path, relativePath));
}

/**
 * Helper to create test data factories.
 */
export const testData = {
  /**
   * Create a minimal valid PNG (1x1 transparent pixel).
   */
  createTestPng(): Buffer {
    return Buffer.from([
      0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, // PNG signature
      0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
      0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
      0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4, 0x89, // RGBA
      0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
      0x08, 0xd7, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01,
      0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, // IEND
      0xae, 0x42, 0x60, 0x82,
    ]);
  },

  /**
   * Create a minimal valid JPEG.
   */
  createTestJpeg(): Buffer {
    return Buffer.from([
      0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01,
      0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43,
      0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09,
      0x09, 0x08, 0x0a, 0x0c, 0x14, 0x0d, 0x0c, 0x0b, 0x0b, 0x0c, 0x19, 0x12,
      0x13, 0x0f, 0x14, 0x1d, 0x1a, 0x1f, 0x1e, 0x1d, 0x1a, 0x1c, 0x1c, 0x20,
      0x24, 0x2e, 0x27, 0x20, 0x22, 0x2c, 0x23, 0x1c, 0x1c, 0x28, 0x37, 0x29,
      0x2c, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1f, 0x27, 0x39, 0x3d, 0x38, 0x32,
      0x3c, 0x2e, 0x33, 0x34, 0x32, 0xff, 0xc0, 0x00, 0x0b, 0x08, 0x00, 0x01,
      0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xff, 0xc4, 0x00, 0x1f, 0x00, 0x00,
      0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
      0x09, 0x0a, 0x0b, 0xff, 0xc4, 0x00, 0xb5, 0x10, 0x00, 0x02, 0x01, 0x03,
      0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7d,
      0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06,
      0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08,
      0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0, 0x24, 0x33, 0x62, 0x72,
      0x82, 0x09, 0x0a, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28,
      0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45,
      0x46, 0x47, 0x48, 0x49, 0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59,
      0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x73, 0x74, 0x75,
      0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
      0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3,
      0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6,
      0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9,
      0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2,
      0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4,
      0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xff, 0xda, 0x00, 0x08, 0x01, 0x01,
      0x00, 0x00, 0x3f, 0x00, 0xfb, 0xd5, 0xdb, 0x20, 0xa8, 0xf0, 0x00, 0x01,
      0xff, 0xd9,
    ]);
  },

  /**
   * Create test text content.
   */
  createTestText(content: string = 'Test content'): Buffer {
    return Buffer.from(content, 'utf-8');
  },

  /**
   * Generate a random issue title.
   */
  randomIssueTitle(): string {
    const prefixes = ['Fix', 'Add', 'Update', 'Remove', 'Refactor'];
    const subjects = ['bug', 'feature', 'documentation', 'test', 'configuration'];
    const prefix = prefixes[Math.floor(Math.random() * prefixes.length)];
    const subject = subjects[Math.floor(Math.random() * subjects.length)];
    return `${prefix} ${subject} - ${randomUUID().slice(0, 8)}`;
  },

  /**
   * Generate a random doc title.
   */
  randomDocTitle(): string {
    const prefixes = ['Getting Started', 'API Reference', 'Configuration', 'Guide'];
    const prefix = prefixes[Math.floor(Math.random() * prefixes.length)];
    return `${prefix} - ${randomUUID().slice(0, 8)}`;
  },

  /**
   * Generate a random PR title.
   */
  randomPrTitle(): string {
    const prefixes = ['feat', 'fix', 'refactor', 'chore', 'docs'];
    const subjects = ['authentication', 'api', 'database', 'ui', 'tests'];
    const prefix = prefixes[Math.floor(Math.random() * prefixes.length)];
    const subject = subjects[Math.floor(Math.random() * subjects.length)];
    return `${prefix}: update ${subject} - ${randomUUID().slice(0, 8)}`;
  },

  /**
   * Generate a random org issue title.
   */
  randomOrgIssueTitle(): string {
    const prefixes = ['Plan', 'Track', 'Coordinate', 'Review', 'Audit'];
    const subjects = ['Q1 roadmap', 'cross-team sync', 'security review', 'performance', 'deployment'];
    const prefix = prefixes[Math.floor(Math.random() * prefixes.length)];
    const subject = subjects[Math.floor(Math.random() * subjects.length)];
    return `${prefix} ${subject} - ${randomUUID().slice(0, 8)}`;
  },

  /**
   * Generate a random organization name.
   */
  randomOrgName(): string {
    const prefixes = ['Acme', 'Test', 'Dev', 'Demo', 'Sample'];
    const suffixes = ['Corp', 'Labs', 'Team', 'Org', 'Inc'];
    const prefix = prefixes[Math.floor(Math.random() * prefixes.length)];
    const suffix = suffixes[Math.floor(Math.random() * suffixes.length)];
    return `${prefix} ${suffix} ${randomUUID().slice(0, 4)}`;
  },
};

export interface TempGitProjectOptions extends TempProjectOptions {
  /** Default branch name (default: 'main') */
  defaultBranch?: string;
  /** Create an initial commit (default: true) */
  initialCommit?: boolean;
}

const DEFAULT_GIT_OPTIONS: Required<TempGitProjectOptions> = {
  ...DEFAULT_OPTIONS,
  defaultBranch: 'main',
  initialCommit: true,
};

/**
 * Create a temporary project directory with git initialized.
 * Required for PR tests since PRs need git branch information.
 */
export async function createTempGitProject(
  options: TempGitProjectOptions = {}
): Promise<TempProject> {
  const opts = { ...DEFAULT_GIT_OPTIONS, ...options };

  // Create unique temp directory
  const projectPath = join(tmpdir(), `${opts.prefix}-${randomUUID()}`);
  await mkdir(projectPath, { recursive: true });

  // Initialize git repository
  execSync(`git init -b ${opts.defaultBranch}`, { cwd: projectPath, stdio: 'pipe' });
  execSync('git config user.email "test@example.com"', { cwd: projectPath, stdio: 'pipe' });
  execSync('git config user.name "Test User"', { cwd: projectPath, stdio: 'pipe' });

  // Create initial commit if requested
  if (opts.initialCommit) {
    await writeFile(join(projectPath, 'README.md'), '# Test Project\n', 'utf-8');
    execSync('git add .', { cwd: projectPath, stdio: 'pipe' });
    execSync('git commit -m "Initial commit"', { cwd: projectPath, stdio: 'pipe' });
  }

  const centyPath = join(projectPath, '.centy');
  const { centyClient, configClient } = createGrpcClient(opts.daemonAddress);
  const client = promisifyClient(centyClient, configClient);

  // Initialize centy if requested
  if (opts.initialize) {
    const result = await client.init({
      projectPath,
      force: true,
    });

    if (!result.success) {
      await rm(projectPath, { recursive: true, force: true });
      throw new Error(`Failed to initialize temp git project: ${result.error}`);
    }
  }

  return {
    path: projectPath,
    centyPath,
    client,
    cleanup: async () => {
      client.close();
      await rm(projectPath, { recursive: true, force: true });
    },
  };
}

/**
 * Create a git branch in a temp project.
 */
export function createGitBranch(projectPath: string, branchName: string): void {
  execSync(`git checkout -b ${branchName}`, { cwd: projectPath, stdio: 'pipe' });
}

/**
 * Switch to a git branch in a temp project.
 */
export function switchGitBranch(projectPath: string, branchName: string): void {
  execSync(`git checkout ${branchName}`, { cwd: projectPath, stdio: 'pipe' });
}

/**
 * Get the current git branch name.
 */
export function getCurrentGitBranch(projectPath: string): string {
  return execSync('git rev-parse --abbrev-ref HEAD', { cwd: projectPath, encoding: 'utf-8' }).trim();
}
