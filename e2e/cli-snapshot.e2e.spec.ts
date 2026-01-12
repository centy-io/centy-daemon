import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { mkdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { randomUUID } from 'node:crypto';
import { CLIWrapper, createCLI } from './fixtures/cli-wrapper.js';
import { SnapshotManager } from './fixtures/snapshot.js';

/**
 * CLI and Snapshot E2E Tests
 *
 * Tests the CLI wrapper with filesystem snapshots after each command.
 * Uses Vitest's toMatchSnapshot() for stable assertions.
 *
 * This test demonstrates the Docker e2e testing pattern:
 * 1. Create a project via CLI
 * 2. Take filesystem snapshots after each command
 * 3. Assert on snapshot structure using toMatchSnapshot()
 */
describe('CLI with Filesystem Snapshots', () => {
  let projectPath: string;
  let cli: CLIWrapper;
  let snapshots: SnapshotManager;

  beforeEach(async () => {
    // Create isolated temp project directory
    projectPath = join(tmpdir(), `centy-cli-snapshot-${randomUUID()}`);
    await mkdir(projectPath, { recursive: true });

    cli = createCLI({ cwd: projectPath });
    snapshots = new SnapshotManager({
      rootPath: projectPath,
      centyOnly: false, // Capture all files for debugging
    });
  });

  afterEach(async () => {
    cli.close();
    await rm(projectPath, { recursive: true, force: true });
  });

  describe('Project Initialization Flow', () => {
    // TODO: This test uses toMatchSnapshot with file paths containing UUIDs
    // which change on each run. Need to normalize UUIDs before snapshotting.
    it.skip('should create .centy directory structure on init', async () => {
      // Take initial snapshot (empty project)
      await snapshots.take('before-init');
      const beforeTree = snapshots.getFileTree('before-init');
      expect(beforeTree.files).toHaveLength(0);

      // Run CLI init command
      const result = await cli.init({ force: true });
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('Initialized centy project');

      // Take snapshot after init and match structure
      await snapshots.take('after-init');
      const afterTree = snapshots.getFileTree('after-init');
      expect(afterTree).toMatchSnapshot('init-file-structure');

      // Verify diff
      const diff = snapshots.getDiffSummary('before-init', 'after-init');
      expect(diff).toMatchSnapshot('init-diff');
    });

    it('should report status correctly', async () => {
      // Before init - should report not initialized
      let result = await cli.status();
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('Not a centy project');

      // After init - should report initialized
      await cli.init({ force: true });
      result = await cli.status();
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('Centy project initialized');
    });
  });

  describe('Issue Workflow', () => {
    beforeEach(async () => {
      await cli.init({ force: true });
      await snapshots.take('initialized');
    });

    it('should create an issue and track file changes', async () => {
      // Create first issue
      const result = await cli.issueCreate('Fix login bug', {
        description: 'Users cannot login with email',
        priority: 1,
        status: 'open',
      });

      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('Created issue #1');

      // Take snapshot after issue creation
      await snapshots.take('after-issue');

      // Match the diff structure
      const diff = snapshots.getDiffSummary('initialized', 'after-issue');
      expect(diff.added.length).toBeGreaterThan(0);
      expect(diff.removed).toHaveLength(0);

      // Verify issue files were created
      expect(diff.added.some((p) => p.startsWith('.centy/issues/'))).toBe(true);
      expect(diff.added.some((p) => p.endsWith('issue.md'))).toBe(true);
    });

    it('should list issues', async () => {
      // Create multiple issues
      await cli.issueCreate('Issue 1', { status: 'open' });
      await cli.issueCreate('Issue 2', { status: 'in_progress' });
      await cli.issueCreate('Issue 3', { status: 'done' });

      // List all issues
      const result = await cli.issueList();
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('#1');
      expect(result.stdout).toContain('#2');
      expect(result.stdout).toContain('#3');

      // Filter by status
      const openResult = await cli.issueList({ status: 'open' });
      expect(openResult.exitCode).toBe(0);
      expect(openResult.stdout).toContain('#1');
    });

    it('should show issue details', async () => {
      await cli.issueCreate('Test Issue', {
        description: 'This is a test description',
      });

      const result = await cli.issueShow(1);
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('Issue #1');
      expect(result.stdout).toContain('Test Issue');
      expect(result.stdout).toContain('This is a test description');
    });

    it('should update an issue and track changes', async () => {
      await cli.issueCreate('Original Title', { status: 'open' });
      await snapshots.take('before-update');

      // Update the issue
      const result = await cli.issueUpdate(1, {
        title: 'Updated Title',
        status: 'in_progress',
      });

      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('Updated issue #1');

      // Take snapshot and compare
      await snapshots.take('after-update');
      const diff = snapshots.getDiffSummary('before-update', 'after-update');

      // Should have modified files (not added or removed)
      expect(diff.modified.length).toBeGreaterThan(0);
      expect(diff.added).toHaveLength(0);
      expect(diff.removed).toHaveLength(0);

      // Verify issue file was modified
      expect(
        diff.modified.some((p) => p.includes('issue.md') || p.includes('metadata.json'))
      ).toBe(true);
    });

    it('should delete an issue and remove files', async () => {
      await cli.issueCreate('To Be Deleted', {});
      await snapshots.take('before-delete');

      // Verify issue exists
      const beforeTree = snapshots.getFileTree('before-delete');
      expect(beforeTree.files.some((f) => f.includes('.centy/issues/'))).toBe(true);

      // Delete the issue
      const result = await cli.issueDelete(1);
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('Deleted issue #1');

      // Take snapshot and compare
      await snapshots.take('after-delete');
      const diff = snapshots.getDiffSummary('before-delete', 'after-delete');

      // Should have removed issue files
      expect(diff.removed.length).toBeGreaterThan(0);
      expect(diff.removed.some((p) => p.includes('.centy/issues/'))).toBe(true);
    });
  });

  describe('Document Workflow', () => {
    beforeEach(async () => {
      await cli.init({ force: true });
      await snapshots.take('initialized');
    });

    it('should create a document and track file changes', async () => {
      const result = await cli.docCreate('Getting Started', {
        content: '# Getting Started\n\nWelcome to the project!',
        slug: 'getting-started',
      });

      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('Created doc: getting-started');

      // Take snapshot
      await snapshots.take('after-doc');
      const diff = snapshots.getDiffSummary('initialized', 'after-doc');

      // Should have created doc file
      expect(diff.added.length).toBeGreaterThan(0);
      expect(diff.added.some((p) => p.includes('.centy/docs/'))).toBe(true);
    });

    it('should list documents', async () => {
      await cli.docCreate('Doc 1', { slug: 'doc-1' });
      await cli.docCreate('Doc 2', { slug: 'doc-2' });

      const result = await cli.docList();
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('doc-1');
      expect(result.stdout).toContain('doc-2');
    });
  });

  describe('Snapshot Structure Tests', () => {
    // TODO: These tests use toMatchSnapshot with file paths containing UUIDs
    // which change on each run. Need to normalize UUIDs before snapshotting.
    it.skip('should match full project structure', async () => {
      await cli.init({ force: true });
      await cli.issueCreate('Test Issue 1', {});
      await cli.issueCreate('Test Issue 2', {});
      await cli.docCreate('Test Doc', { slug: 'test-doc' });

      await snapshots.take('full-project');
      const tree = snapshots.getFileTree('full-project');

      // Match the full structure
      expect(tree).toMatchSnapshot('full-project-structure');
    });

    it.skip('should match file extensions breakdown', async () => {
      await cli.init({ force: true });
      await cli.issueCreate('Test Issue', { description: 'Test' });
      await cli.docCreate('Test Doc', { slug: 'test-doc' });

      await snapshots.take('with-content');
      const tree = snapshots.getFileTree('with-content');

      // Match extensions breakdown
      expect(tree.byExtension).toMatchSnapshot('file-extensions');
    });

    it('should get file content from snapshots', async () => {
      await cli.init({ force: true });
      await cli.issueCreate('Content Test', {
        description: 'Test description content',
      });

      const snapshot = await snapshots.take('with-issue', true);

      // Find the issue.md file and check its content
      const issueFile = snapshot.files.find((f) => f.path.endsWith('issue.md'));
      expect(issueFile).toBeDefined();
      expect(issueFile?.content).toBeDefined();
      expect(issueFile?.content).toContain('Content Test');
    });
  });

  describe('Error Handling', () => {
    it('should handle errors gracefully', async () => {
      // Try to show non-existent issue without init
      const result = await cli.issueShow(999);
      expect(result.exitCode).toBe(1);
      expect(result.stderr).toBeTruthy();
    });
  });

  describe('Daemon Info', () => {
    it('should return daemon info', async () => {
      const result = await cli.info();
      expect(result.exitCode).toBe(0);
      expect(result.stdout).toContain('Centy Daemon');
      expect(result.stdout).toContain('Version:');
    });
  });
});
