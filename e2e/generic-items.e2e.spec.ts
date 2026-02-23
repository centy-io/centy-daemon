import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createTempProject, type TempProject } from './fixtures/temp-project.js';

/**
 * gRPC Plain Text Tests for Generic CRUD RPCs
 *
 * Tests the CreateItem, GetItem, ListItems, UpdateItem, DeleteItem,
 * SoftDeleteItem, and RestoreItem RPCs which operate on any item type.
 */
describe('gRPC: Generic CRUD RPCs', () => {
  let project: TempProject;

  beforeEach(async () => {
    project = await createTempProject({ initialize: true });
  });

  afterEach(async () => {
    await project.cleanup();
  });

  // ─── CreateItem ────────────────────────────────────────────────────────────

  describe('CreateItem', () => {
    it('should create an issue via generic RPC', async () => {
      const result = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Test Issue',
        body: 'Issue body text',
        status: 'open',
        priority: 2,
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.item).toBeDefined();
      expect(result.item!.title).toBe('Test Issue');
      expect(result.item!.body).toBe('Issue body text');
      expect(result.item!.itemType).toBe('issues');
      expect(result.item!.id).toBeTruthy();
      expect(result.item!.metadata.displayNumber).toBe(1);
      expect(result.item!.metadata.status).toBe('open');
      expect(result.item!.metadata.priority).toBe(2);
      expect(result.item!.metadata.createdAt).toBeTruthy();
      expect(result.item!.metadata.deletedAt).toBe('');
    });

    it('should create a doc via generic RPC', async () => {
      const result = await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Getting Started',
        body: '# Getting Started\n\nWelcome!',
      });

      expect(result.success).toBe(true);
      expect(result.item).toBeDefined();
      expect(result.item!.id).toBe('getting-started');
      expect(result.item!.itemType).toBe('docs');
      expect(result.item!.title).toBe('Getting Started');
      // Docs have no display number, status, or priority
      expect(result.item!.metadata.displayNumber).toBe(0);
      expect(result.item!.metadata.status).toBe('');
      expect(result.item!.metadata.priority).toBe(0);
    });

    it('should accept singular item type name', async () => {
      const result = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issue',
        title: 'Singular Type Test',
        status: 'open',
      });

      expect(result.success).toBe(true);
      expect(result.item!.itemType).toBe('issues');
    });

    it('should fail for unknown item type', async () => {
      const result = await project.client.createItem({
        projectPath: project.path,
        itemType: 'nonexistent',
        title: 'Test',
      });

      expect(result.success).toBe(false);
      expect(result.error).toContain('ITEM_TYPE_NOT_FOUND');
    });

    it('should store custom fields', async () => {
      const result = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Issue with Custom Fields',
        status: 'open',
        priority: 1,
        customFields: {
          env: '"production"',
          count: '42',
          tags: '["bug","urgent"]',
        },
      });

      expect(result.success).toBe(true);
      expect(result.item!.metadata.customFields['env']).toBe('"production"');
      expect(result.item!.metadata.customFields['count']).toBe('42');
      expect(result.item!.metadata.customFields['tags']).toBe('["bug","urgent"]');
    });

    it('should increment display numbers for issues', async () => {
      const first = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'First',
        status: 'open',
      });
      const second = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Second',
        status: 'open',
      });

      expect(first.item!.metadata.displayNumber).toBe(1);
      expect(second.item!.metadata.displayNumber).toBe(2);
    });
  });

  // ─── GetItem ───────────────────────────────────────────────────────────────

  describe('GetItem', () => {
    it('should get an issue by id', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Test Issue',
        status: 'open',
        priority: 1,
      });
      expect(created.success).toBe(true);
      const id = created.item!.id;

      const result = await project.client.getItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: id,
      });

      expect(result.success).toBe(true);
      expect(result.item!.id).toBe(id);
      expect(result.item!.title).toBe('Test Issue');
      expect(result.item!.metadata.status).toBe('open');
      expect(result.item!.metadata.priority).toBe(1);
    });

    it('should get an issue by display number', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Display Number Test',
        status: 'open',
      });
      expect(created.success).toBe(true);

      const result = await project.client.getItem({
        projectPath: project.path,
        itemType: 'issues',
        displayNumber: 1,
      });

      expect(result.success).toBe(true);
      expect(result.item!.title).toBe('Display Number Test');
    });

    it('should get a doc by slug', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'API Reference',
        body: 'API docs content',
      });

      const result = await project.client.getItem({
        projectPath: project.path,
        itemType: 'docs',
        itemId: 'api-reference',
      });

      expect(result.success).toBe(true);
      expect(result.item!.title).toBe('API Reference');
      expect(result.item!.body).toBe('API docs content');
    });

    it('should fail for non-existent item', async () => {
      const result = await project.client.getItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: '00000000-0000-0000-0000-000000000000',
      });

      expect(result.success).toBe(false);
      expect(result.error).toContain('ITEM_NOT_FOUND');
    });
  });

  // ─── ListItems ─────────────────────────────────────────────────────────────

  describe('ListItems', () => {
    it('should list all issues', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Issue 1',
        status: 'open',
      });
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Issue 2',
        status: 'closed',
      });

      const result = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
      });

      expect(result.success).toBe(true);
      expect(result.totalCount).toBe(2);
      expect(result.items.length).toBe(2);
    });

    it('should filter by status using MQL', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Open Issue',
        status: 'open',
      });
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Closed Issue',
        status: 'closed',
      });

      const result = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
        filter: '{"status":"open"}',
      });

      expect(result.success).toBe(true);
      expect(result.totalCount).toBe(1);
      expect(result.items[0].title).toBe('Open Issue');
    });

    it('should support pagination with limit and offset', async () => {
      for (let i = 1; i <= 5; i++) {
        await project.client.createItem({
          projectPath: project.path,
          itemType: 'issues',
          title: `Issue ${i}`,
          status: 'open',
        });
      }

      const page1 = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
        limit: 2,
        offset: 0,
      });
      expect(page1.totalCount).toBe(2);

      const page2 = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
        limit: 2,
        offset: 2,
      });
      expect(page2.totalCount).toBe(2);

      // Page IDs should be different
      const page1Ids = new Set(page1.items.map((i) => i.id));
      const page2Ids = new Set(page2.items.map((i) => i.id));
      for (const id of page2Ids) {
        expect(page1Ids.has(id)).toBe(false);
      }
    });

    it('should return empty list for project with no items', async () => {
      const result = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
      });

      expect(result.success).toBe(true);
      expect(result.totalCount).toBe(0);
      expect(result.items).toEqual([]);
    });

    it('should list docs', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Doc One',
        body: 'Content',
      });
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Doc Two',
        body: 'More content',
      });

      const result = await project.client.listItems({
        projectPath: project.path,
        itemType: 'docs',
      });

      expect(result.success).toBe(true);
      expect(result.totalCount).toBe(2);
    });
  });

  // ─── UpdateItem ────────────────────────────────────────────────────────────

  describe('UpdateItem', () => {
    it('should update an issue title, body, status, and priority', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Original Title',
        body: 'Original body',
        status: 'open',
        priority: 3,
      });
      expect(created.success).toBe(true);
      const id = created.item!.id;

      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: id,
        title: 'Updated Title',
        body: 'Updated body',
        status: 'in-progress',
        priority: 1,
      });

      expect(result.success).toBe(true);
      expect(result.item!.title).toBe('Updated Title');
      expect(result.item!.body).toBe('Updated body');
      expect(result.item!.metadata.status).toBe('in-progress');
      expect(result.item!.metadata.priority).toBe(1);
    });

    it('should preserve unchanged fields when doing partial update', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Original',
        body: 'Keep this body',
        status: 'open',
        priority: 2,
      });
      const id = created.item!.id;

      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: id,
        title: 'New Title',
        // body, status, priority not set - should be preserved
      });

      expect(result.success).toBe(true);
      expect(result.item!.title).toBe('New Title');
      expect(result.item!.body).toBe('Keep this body');
      expect(result.item!.metadata.status).toBe('open');
      expect(result.item!.metadata.priority).toBe(2);
    });

    it('should update a doc title and body', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Original Doc',
        body: 'Original content',
      });

      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'docs',
        itemId: 'original-doc',
        title: 'Updated Doc',
        body: 'Updated content',
      });

      expect(result.success).toBe(true);
      expect(result.item!.title).toBe('Updated Doc');
      expect(result.item!.body).toBe('Updated content');
    });

    it('should fail for non-existent item', async () => {
      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: '00000000-0000-0000-0000-000000000000',
        title: 'Updated',
      });

      expect(result.success).toBe(false);
      expect(result.error).toBeTruthy();
    });
  });

  // ─── DeleteItem ────────────────────────────────────────────────────────────

  describe('DeleteItem', () => {
    it('should hard delete an issue when force=true', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'To Delete',
        status: 'open',
      });
      const id = created.item!.id;

      const deleted = await project.client.deleteItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: id,
        force: true,
      });

      expect(deleted.success).toBe(true);

      // Should not be findable after delete
      const get = await project.client.getItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: id,
      });
      expect(get.success).toBe(false);
      expect(get.error).toContain('ITEM_NOT_FOUND');
    });

    it('should delete a doc', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Doc to Delete',
        body: 'Content',
      });

      const deleted = await project.client.deleteItem({
        projectPath: project.path,
        itemType: 'docs',
        itemId: 'doc-to-delete',
        force: true,
      });

      expect(deleted.success).toBe(true);
    });

    it('should fail to hard delete non-existent item', async () => {
      const result = await project.client.deleteItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: '00000000-0000-0000-0000-000000000000',
        force: true,
      });

      expect(result.success).toBe(false);
    });
  });

  // ─── SoftDeleteItem + RestoreItem ─────────────────────────────────────────

  describe('SoftDeleteItem + RestoreItem', () => {
    it('should soft delete and restore an issue', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Soft Delete Test',
        status: 'open',
      });
      const id = created.item!.id;

      // Soft delete
      const softDeleted = await project.client.softDeleteItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: id,
      });

      expect(softDeleted.success).toBe(true);
      expect(softDeleted.item).toBeDefined();
      expect(softDeleted.item!.metadata.deletedAt).not.toBe('');

      // Should not appear in regular list
      const listAfterDelete = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
      });
      expect(listAfterDelete.totalCount).toBe(0);

      // Should appear with deletedAt filter
      const listDeleted = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
        filter: '{"deletedAt":{"$exists":true}}',
      });
      expect(listDeleted.totalCount).toBe(1);

      // Restore
      const restored = await project.client.restoreItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: id,
      });

      expect(restored.success).toBe(true);
      expect(restored.item!.metadata.deletedAt).toBe('');

      // Should appear in regular list again
      const listAfterRestore = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
      });
      expect(listAfterRestore.totalCount).toBe(1);
    });

    it('should soft delete a doc', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Doc to Soft Delete',
        body: 'Content',
      });

      const result = await project.client.softDeleteItem({
        projectPath: project.path,
        itemType: 'docs',
        itemId: 'doc-to-soft-delete',
      });

      expect(result.success).toBe(true);
      expect(result.item!.metadata.deletedAt).not.toBe('');
    });

    it('should fail to restore an item that was not soft deleted', async () => {
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Not Deleted',
        status: 'open',
      });

      const result = await project.client.restoreItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: created.item!.id,
      });

      expect(result.success).toBe(false);
    });
  });

  // ─── Full CRUD lifecycle ───────────────────────────────────────────────────

  describe('Full CRUD lifecycle', () => {
    it('should support a complete create → read → update → delete cycle', async () => {
      // Create
      const created = await project.client.createItem({
        projectPath: project.path,
        itemType: 'issues',
        title: 'Lifecycle Issue',
        body: 'Initial body',
        status: 'open',
        priority: 2,
      });
      expect(created.success).toBe(true);
      const id = created.item!.id;

      // Read
      const fetched = await project.client.getItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: id,
      });
      expect(fetched.success).toBe(true);
      expect(fetched.item!.title).toBe('Lifecycle Issue');

      // Update
      const updated = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: id,
        status: 'closed',
      });
      expect(updated.success).toBe(true);
      expect(updated.item!.metadata.status).toBe('closed');

      // List — should appear
      const list = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
        filter: '{"status":"closed"}',
      });
      expect(list.totalCount).toBe(1);

      // Hard delete
      const deleted = await project.client.deleteItem({
        projectPath: project.path,
        itemType: 'issues',
        itemId: id,
        force: true,
      });
      expect(deleted.success).toBe(true);

      // List — should be gone
      const listAfter = await project.client.listItems({
        projectPath: project.path,
        itemType: 'issues',
      });
      expect(listAfter.totalCount).toBe(0);
    });
  });
});
