import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createTempProject, type TempProject, projectFileExists } from './fixtures/temp-project.js';

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
  });

  describe('Init with config options', () => {
    it('should apply priorityLevels from initConfig', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
        initConfig: { priorityLevels: 5 },
      });

      const config = await project.client.getConfig({ projectPath: project.path });
      expect(config.priorityLevels).toBe(5);
    });

    it('should apply defaultEditor from initConfig', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
        initConfig: { defaultEditor: 'vscode' },
      });

      const config = await project.client.getConfig({ projectPath: project.path });
      expect(config.defaultEditor).toBe('vscode');
    });

    it('should apply workspace.updateStatusOnOpen from initConfig', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
        initConfig: { workspace: { updateStatusOnOpen: true } },
      });

      const config = await project.client.getConfig({ projectPath: project.path });
      expect(config.workspace?.updateStatusOnOpen).toBe(true);
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

    it('should preserve unset config fields as defaults when initConfig is partial', async () => {
      await project.client.init({
        projectPath: project.path,
        force: true,
        initConfig: { priorityLevels: 4 },
      });

      const config = await project.client.getConfig({ projectPath: project.path });
      expect(config.priorityLevels).toBe(4);
      // Other fields should remain at defaults
      expect(config.customFields).toEqual([]);
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
