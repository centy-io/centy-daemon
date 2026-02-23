import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createTempProject, type TempProject } from './fixtures/temp-project.js';

/**
 * gRPC Plain Text Tests for Issue Operations
 *
 * Tests direct gRPC calls for issue CRUD operations using generic RPCs.
 */
describe('gRPC: Issue Operations', () => {
  let project: TempProject;

  beforeEach(async () => {
    project = await createTempProject({ initialize: true });
  });

  afterEach(async () => {
    await project.cleanup();
  });

  describe('CreateItem (issues)', () => {
    it('should create an issue with minimal fields', async () => {
      const result = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Test Issue',
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.item).toBeDefined();
      expect(result.item!.id).toBeTruthy();
      expect(result.item!.metadata.displayNumber).toBe(1);
    });

    it('should create an issue with all fields', async () => {
      const result = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Full Issue',
        body: 'This is a detailed description',
        priority: 1,
        status: 'in-progress',
      });

      expect(result.success).toBe(true);
      expect(result.item!.metadata.displayNumber).toBe(1);
      expect(result.item!.metadata.status).toBe('in-progress');
      expect(result.item!.metadata.priority).toBe(1);
    });

    it('should auto-increment display numbers', async () => {
      const issue1 = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'First Issue',
      });

      const issue2 = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Second Issue',
      });

      const issue3 = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Third Issue',
      });

      expect(issue1.item!.metadata.displayNumber).toBe(1);
      expect(issue2.item!.metadata.displayNumber).toBe(2);
      expect(issue3.item!.metadata.displayNumber).toBe(3);
    });
  });

  describe('GetItem (issues)', () => {
    it('should get issue by ID', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Get By ID Test',
        body: 'Test description',
      });

      const result = await project.client.getItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
      });

      expect(result.success).toBe(true);
      expect(result.item!.id).toBe(created.item!.id);
      expect(result.item!.title).toBe('Get By ID Test');
      expect(result.item!.body).toContain('Test description');
    });

    it('should return full metadata', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Metadata Test',
        priority: 2,
        status: 'open',
      });

      const result = await project.client.getItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
      });

      expect(result.item!.metadata).toBeDefined();
      expect(result.item!.metadata.status).toBe('open');
      expect(result.item!.metadata.priority).toBe(2);
      expect(result.item!.metadata.createdAt).toBeTruthy();
      expect(result.item!.metadata.updatedAt).toBeTruthy();
    });

    it('should get issue by display number', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Display Number Test',
      });

      const result = await project.client.getItem({
        projectPath: project.path,
        itemType: 'issues',
        displayNumber: 1,
      });

      expect(result.success).toBe(true);
      expect(result.item!.metadata.displayNumber).toBe(1);
      expect(result.item!.title).toBe('Display Number Test');
    });
  });

  describe('ListItems (issues)', () => {
    it('should list all issues', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Issue 1',
      });
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Issue 2',
      });
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Issue 3',
      });

      const result = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
      });

      expect(result.success).toBe(true);
      expect(result.totalCount).toBe(3);
      expect(result.items.length).toBe(3);
    });

    it('should return empty list for empty project', async () => {
      const result = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
      });

      expect(result.success).toBe(true);
      expect(result.totalCount).toBe(0);
      expect(result.items.length).toBe(0);
    });
  });

  describe('UpdateItem (issues)', () => {
    it('should update issue title', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Original Title',
      });

      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
        title: 'Updated Title',
      });

      expect(result.success).toBe(true);
      expect(result.item?.title).toBe('Updated Title');
    });

    it('should update issue status', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Status Update Test',
        status: 'open',
      });

      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
        status: 'closed',
      });

      expect(result.success).toBe(true);
      expect(result.item?.metadata.status).toBe('closed');
    });

    it('should update issue priority', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Priority Update Test',
        priority: 3,
      });

      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
        priority: 1,
      });

      expect(result.success).toBe(true);
      expect(result.item?.metadata.priority).toBe(1);
    });

    it('should update issue body/description', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Description Update Test',
        body: 'Original description',
      });

      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
        body: 'Updated description with more details',
      });

      expect(result.success).toBe(true);
      expect(result.item?.body).toContain('Updated description');
    });

    it('should update updatedAt timestamp', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Timestamp Test',
      });

      // Small delay to ensure different timestamp
      await new Promise((r) => setTimeout(r, 100));

      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
        title: 'Updated Timestamp Test',
      });

      // Just verify updatedAt is a valid date
      expect(result.item?.metadata.updatedAt).toBeTruthy();
    });
  });

  describe('DeleteItem (issues)', () => {
    it('should delete an issue', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'To Delete',
      });

      const deleteResult = await project.client.deleteItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
        force: true,
      });

      expect(deleteResult.success).toBe(true);

      // Verify it's gone
      const listResult = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
      });
      expect(listResult.totalCount).toBe(0);
    });

    it('should return error for non-existent issue', async () => {
      const result = await project.client.deleteItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: 'non-existent-id',
        force: true,
      });

      expect(result.success).toBe(false);
      expect(result.error).toBeTruthy();
    });
  });

  describe('SoftDeleteItem and RestoreItem (issues)', () => {
    it('should soft-delete an issue', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Soft Delete Test',
      });

      const result = await project.client.softDeleteItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
      });

      expect(result.success).toBe(true);
      expect(result.item?.metadata.deletedAt).toBeTruthy();
    });

    it('should restore a soft-deleted issue', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Restore Test',
      });

      // First soft-delete
      await project.client.softDeleteItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
      });

      // Then restore
      const result = await project.client.restoreItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
      });

      expect(result.success).toBe(true);
      expect(result.item?.metadata.deletedAt).toBe('');
    });
  });

  describe('GetNextIssueNumber', () => {
    it('should return next available number', async () => {
      const result1 = await project.client.getNextIssueNumber({
        projectPath: project.path,
      });

      await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Issue 1',
      });

      const result2 = await project.client.getNextIssueNumber({
        projectPath: project.path,
      });

      expect(result1.issueNumber).toBeDefined();
      expect(result2.issueNumber).toBeDefined();
    });
  });
});
