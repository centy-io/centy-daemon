import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import {
  createTempProject,
  type TempProject,
  projectFileExists,
  readProjectFile,
  writeProjectFile,
} from './fixtures/temp-project.js';

/**
 * gRPC Plain Text Tests for Init Operations
 *
 * Tests direct gRPC calls for project initialization.
 */
describe('gRPC: Init Operations', () => {
  let project: TempProject;

  beforeEach(async () => {
    // Create temp project without initialization
    project = await createTempProject({ initialize: false });
  });

  afterEach(async () => {
    await project.cleanup();
  });

  describe('Init', () => {
    it('should initialize a new project', async () => {
      const result = await project.client.init({
        projectPath: project.path,
        force: true,
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.created.length).toBeGreaterThan(0);
    });

    it('should create manifest file', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      expect(projectFileExists(project, '.centy/.centy-manifest.json')).toBe(true);
    });

    it('should create README file', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      expect(projectFileExists(project, '.centy/README.md')).toBe(true);
    });

    it('should create required directories', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      expect(projectFileExists(project, '.centy/issues')).toBe(true);
      expect(projectFileExists(project, '.centy/docs')).toBe(true);
      expect(projectFileExists(project, '.centy/assets')).toBe(true);
      expect(projectFileExists(project, '.centy/templates')).toBe(true);
    });

    it('should return manifest in response', async () => {
      const result = await project.client.init({
        projectPath: project.path,
        force: true,
      });

      expect(result.manifest).toBeDefined();
      expect(result.manifest?.schemaVersion).toBeGreaterThan(0);
      expect(result.manifest?.centyVersion).toBeDefined();
    });

    it('should create issues/config.yaml', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      expect(projectFileExists(project, '.centy/issues/config.yaml')).toBe(true);
    });

    it('should create docs/config.yaml', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      expect(projectFileExists(project, '.centy/docs/config.yaml')).toBe(true);
    });

    it('should create issues/config.yaml with correct defaults', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      const content = await readProjectFile(project, '.centy/issues/config.yaml');
      expect(content).toContain('name: Issue');
      expect(content).toContain('identifier: uuid');
      expect(content).toContain('displayNumber: true');
      expect(content).toContain('status: true');
      expect(content).toContain('priority: true');
      expect(content).toContain('defaultStatus: open');
    });

    it('should create docs/config.yaml with minimal defaults', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      const content = await readProjectFile(project, '.centy/docs/config.yaml');
      expect(content).toContain('name: Doc');
      expect(content).toContain('identifier: slug');
      expect(content).toContain('displayNumber: false');
      expect(content).toContain('status: false');
      expect(content).toContain('priority: false');
      // Docs should not have statuses, defaultStatus, or priorityLevels
      expect(content).not.toContain('statuses:');
      expect(content).not.toContain('defaultStatus:');
      expect(content).not.toContain('priorityLevels:');
    });

    it('should not overwrite existing issues/config.yaml on re-init', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      // Overwrite issues/config.yaml with custom content
      await writeProjectFile(project, '.centy/issues/config.yaml', 'name: CustomIssue\n');

      // Re-init should not overwrite
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      const content = await readProjectFile(project, '.centy/issues/config.yaml');
      expect(content).toBe('name: CustomIssue\n');
    });

    it('should not overwrite existing docs/config.yaml on re-init', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      // Overwrite docs/config.yaml with custom content
      await writeProjectFile(project, '.centy/docs/config.yaml', 'name: CustomDoc\n');

      // Re-init should not overwrite
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      const content = await readProjectFile(project, '.centy/docs/config.yaml');
      expect(content).toBe('name: CustomDoc\n');
    });

    it('should report issues/config.yaml and docs/config.yaml as created', async () => {
      const result = await project.client.init({
        projectPath: project.path,
        force: true,
      });

      expect(result.created).toContain('issues/config.yaml');
      expect(result.created).toContain('docs/config.yaml');
    });
  });

  describe('Init with config options', () => {
    // Full default config shape returned by the daemon after a plain init.
    // version is dynamic (daemon version), so we match it with expect.any(String).
    const defaultConfig = {
      customFields: [],
      defaults: {},
      priorityLevels: 3,
      version: expect.any(String),
      stateColors: {},
      priorityColors: {},
      customLinkTypes: [],
      defaultEditor: '',
      hooks: [],
      workspace: {},
    };

    it('should apply priorityLevels from initConfig', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
        initConfig: { priorityLevels: 5 },
      });

      const config = await project.client.getConfig({ projectPath: project.path });
      expect(config).toEqual({ ...defaultConfig, priorityLevels: 5 });
    });

    it('should apply defaultEditor from initConfig', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
        initConfig: { defaultEditor: 'vscode' },
      });

      const config = await project.client.getConfig({ projectPath: project.path });
      expect(config).toEqual({ ...defaultConfig, defaultEditor: 'vscode' });
    });

    it('should apply workspace.updateStatusOnOpen from initConfig', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
        initConfig: { workspace: { updateStatusOnOpen: true } },
      });

      const config = await project.client.getConfig({ projectPath: project.path });
      expect(config).toEqual({ ...defaultConfig, workspace: { updateStatusOnOpen: true } });
    });

    it('should apply title during initialization', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
        title: 'My Awesome Project',
      });

      const { project: info } = await project.client.getProjectInfo({
        projectPath: project.path,
      });
      expect(info?.projectTitle).toBe('My Awesome Project');
    });

    it('should preserve all other config fields as defaults when initConfig is partial', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
        initConfig: { priorityLevels: 4 },
      });

      const config = await project.client.getConfig({ projectPath: project.path });
      expect(config).toEqual({ ...defaultConfig, priorityLevels: 4 });
    });
  });

  describe('IsInitialized', () => {
    it('should return false for uninitialized directory', async () => {
      const result = await project.client.isInitialized({
        projectPath: project.path,
      });

      expect(result.initialized).toBe(false);
    });

    it('should return true for initialized directory', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      const result = await project.client.isInitialized({
        projectPath: project.path,
      });

      expect(result.initialized).toBe(true);
      expect(result.centyPath).toContain('.centy');
    });
  });

  describe('GetReconciliationPlan', () => {
    it('should return plan for uninitialized project', async () => {
      const plan = await project.client.getReconciliationPlan({
        projectPath: project.path,
      });

      expect(plan).toBeDefined();
      expect(plan.toCreate.length).toBeGreaterThan(0);
    });

    it('should return empty plan for fully initialized project', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
      });

      const plan = await project.client.getReconciliationPlan({
        projectPath: project.path,
      });

      expect(plan.toCreate.length).toBe(0);
      expect(plan.toRestore.length).toBe(0);
      expect(plan.needsDecisions).toBe(false);
    });
  });
});
