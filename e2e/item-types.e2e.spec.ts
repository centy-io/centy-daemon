import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createTempProject, type TempProject } from './fixtures/temp-project.js';

/**
 * gRPC Plain Text Tests for Item Type Operations
 *
 * Tests the ListItemTypes RPC which returns all registered item types
 * and their configs for a given project.
 */
describe('gRPC: Item Type Operations', () => {
  let project: TempProject;

  beforeEach(async () => {
    project = await createTempProject({ initialize: true });
  });

  afterEach(async () => {
    await project.cleanup();
  });

  describe('ListItemTypes', () => {
    it('should return default item types for an initialized project', async () => {
      const result = await project.client.listItemTypes({
        projectPath: project.path,
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.totalCount).toBeGreaterThan(0);
      expect(result.itemTypes.length).toBe(result.totalCount);
    });

    it('should include issues and docs by default', async () => {
      const result = await project.client.listItemTypes({
        projectPath: project.path,
      });

      expect(result.success).toBe(true);

      const plurals = result.itemTypes.map((t) => t.plural);
      expect(plurals).toContain('issues');
      expect(plurals).toContain('docs');
    });

    it('should return correct structure for each item type', async () => {
      const result = await project.client.listItemTypes({
        projectPath: project.path,
      });

      expect(result.success).toBe(true);

      for (const itemType of result.itemTypes) {
        expect(itemType.name).toBeTruthy();
        expect(itemType.plural).toBeTruthy();
        expect(itemType.identifier).toMatch(/^(uuid|slug)$/);
        expect(itemType.features).toBeDefined();
        expect(Array.isArray(itemType.statuses)).toBe(true);
        expect(Array.isArray(itemType.customFields)).toBe(true);
      }
    });

    it('should return issue type with display_number and priority features enabled', async () => {
      const result = await project.client.listItemTypes({
        projectPath: project.path,
      });

      expect(result.success).toBe(true);

      const issueType = result.itemTypes.find((t) => t.plural === 'issues');
      expect(issueType).toBeDefined();
      expect(issueType!.features.displayNumber).toBe(true);
      expect(issueType!.features.priority).toBe(true);
      expect(issueType!.features.status).toBe(true);
      expect(issueType!.identifier).toBe('uuid');
    });

    it('should return doc type with slug identifier', async () => {
      const result = await project.client.listItemTypes({
        projectPath: project.path,
      });

      expect(result.success).toBe(true);

      const docType = result.itemTypes.find((t) => t.plural === 'docs');
      expect(docType).toBeDefined();
      expect(docType!.identifier).toBe('slug');
      expect(docType!.features.displayNumber).toBe(false);
    });

    it('should include a custom item type after it is created', async () => {
      const created = await project.client.createItemType({
        projectPath: project.path,
        name: 'Bug',
        plural: 'bugs',
        identifier: 'uuid',
        features: {
          displayNumber: true,
          status: true,
          priority: true,
          assets: true,
          orgSync: false,
          move: true,
          duplicate: true,
        },
        statuses: ['open', 'in-progress', 'closed'],
        defaultStatus: 'open',
        priorityLevels: 3,
      });

      expect(created.success).toBe(true);

      const result = await project.client.listItemTypes({
        projectPath: project.path,
      });

      expect(result.success).toBe(true);

      const bugType = result.itemTypes.find((t) => t.plural === 'bugs');
      expect(bugType).toBeDefined();
      expect(bugType!.name).toBe('Bug');
      expect(bugType!.identifier).toBe('uuid');
      expect(bugType!.statuses).toEqual(['open', 'in-progress', 'closed']);
      expect(bugType!.defaultStatus).toBe('open');
      expect(bugType!.priorityLevels).toBe(3);
    });

    it('should return empty list for a project with no .centy directory', async () => {
      const result = await project.client.listItemTypes({
        projectPath: '/tmp/nonexistent-centy-project-12345',
      });

      // The registry builds successfully but finds no types in a missing directory
      expect(result.success).toBe(true);
      expect(result.itemTypes).toEqual([]);
      expect(result.totalCount).toBe(0);
    });
  });

  describe('CreateItemType', () => {
    it('should create a new item type with minimal fields', async () => {
      const result = await project.client.createItemType({
        projectPath: project.path,
        name: 'Task',
        plural: 'tasks',
        identifier: 'uuid',
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.config).toBeDefined();
      expect(result.config!.name).toBe('Task');
      expect(result.config!.plural).toBe('tasks');
      expect(result.config!.identifier).toBe('uuid');
    });

    it('should create a new item type with statuses', async () => {
      const result = await project.client.createItemType({
        projectPath: project.path,
        name: 'Epic',
        plural: 'epics',
        identifier: 'uuid',
        statuses: ['backlog', 'active', 'done'],
        defaultStatus: 'backlog',
      });

      expect(result.success).toBe(true);
      expect(result.config!.statuses).toEqual(['backlog', 'active', 'done']);
      expect(result.config!.defaultStatus).toBe('backlog');
    });

    it('should persist the created item type', async () => {
      await project.client.createItemType({
        projectPath: project.path,
        name: 'Spike',
        plural: 'spikes',
        identifier: 'uuid',
      });

      const list = await project.client.listItemTypes({
        projectPath: project.path,
      });

      expect(list.success).toBe(true);
      expect(list.itemTypes.find((t) => t.plural === 'spikes')).toBeDefined();
    });

    it('should reject duplicate plural names', async () => {
      await project.client.createItemType({
        projectPath: project.path,
        name: 'Feature',
        plural: 'features',
        identifier: 'uuid',
      });

      const duplicate = await project.client.createItemType({
        projectPath: project.path,
        name: 'Feature2',
        plural: 'features',
        identifier: 'uuid',
      });

      expect(duplicate.success).toBe(false);
      expect(duplicate.error).toBeTruthy();
    });
  });
});
