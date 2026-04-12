import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createTempProject, type TempProject } from './fixtures/temp-project.js';

/**
 * gRPC Plain Text Tests for Documentation Operations
 *
 * Tests direct gRPC calls for doc CRUD operations via the generic item RPCs.
 * Docs use slug-based identifiers, so itemId is the slug.
 */
describe('gRPC: Doc Operations', () => {
  let project: TempProject;

  beforeEach(async () => {
    project = await createTempProject({ initialize: true });
  });

  afterEach(async () => {
    await project.cleanup();
  });

  describe('CreateItem (docs)', () => {
    it('should create a doc with title only', async () => {
      const result = await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Getting Started',
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.item.id).toBe('getting-started');
    });

    it('should create a doc with content', async () => {
      const result = await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'API Reference',
        body: '# API\n\nThis is the API documentation.',
      });

      expect(result.success).toBe(true);
      expect(result.item.id).toBe('api-reference');
    });

    it('should return doc metadata', async () => {
      const result = await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Metadata Test',
      });

      expect(result.success).toBe(true);
      expect(result.item.metadata.createdAt).toBeDefined();
      expect(result.item.metadata.updatedAt).toBeDefined();
    });
  });

  describe('GetItem (docs)', () => {
    it('should get doc by slug', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Test Doc',
        body: '# Test\n\nThis is test content.',
      });

      const result = await project.client.getItem({
        projectPath: project.path,
        itemType: 'docs',
        itemId: 'test-doc',
      });

      expect(result.item.id).toBe('test-doc');
      expect(result.item.title).toBe('Test Doc');
      expect(result.item.body).toContain('test content');
    });

    it('should return doc metadata', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Metadata Test',
      });

      const result = await project.client.getItem({
        projectPath: project.path,
        itemType: 'docs',
        itemId: 'metadata-test',
      });

      expect(result.item.metadata).toBeDefined();
      expect(result.item.metadata.createdAt).toBeDefined();
      expect(result.item.metadata.updatedAt).toBeDefined();
    });
  });

  describe('ListItems (docs)', () => {
    it('should list all docs', async () => {
      await project.client.createItem({ projectPath: project.path, itemType: 'docs', title: 'Doc One' });
      await project.client.createItem({ projectPath: project.path, itemType: 'docs', title: 'Doc Two' });
      await project.client.createItem({ projectPath: project.path, itemType: 'docs', title: 'Doc Three' });

      const result = await project.client.listItems({
        projectPath: project.path,
        itemType: 'docs',
      });

      expect(result.totalCount).toBe(3);
      expect(result.items.length).toBe(3);
    });

    it('should return empty list for empty project', async () => {
      const result = await project.client.listItems({
        projectPath: project.path,
        itemType: 'docs',
      });

      expect(result.totalCount).toBe(0);
      expect(result.items.length).toBe(0);
    });

    it('should return docs with all fields populated', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Full Doc',
        body: '# Content here',
      });

      const result = await project.client.listItems({
        projectPath: project.path,
        itemType: 'docs',
      });

      const doc = result.items[0];
      expect(doc.id).toBeDefined();
      expect(doc.title).toBeDefined();
      expect(doc.metadata).toBeDefined();
    });
  });

  describe('UpdateItem (docs)', () => {
    it('should update doc title', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Original Title',
      });

      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'docs',
        itemId: 'original-title',
        title: 'Updated Title',
      });

      expect(result.success).toBe(true);
      expect(result.item.title).toBe('Updated Title');
    });

    it('should update doc content', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Content Test',
        body: 'Original content',
      });

      const result = await project.client.updateItem({
        projectPath: project.path,
        itemType: 'docs',
        itemId: 'content-test',
        body: 'Updated content with more information',
      });

      expect(result.success).toBe(true);
      expect(result.item.body).toContain('Updated content');
    });
  });

  describe('MoveItem (docs - rename)', () => {
    it('should rename doc (change slug)', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'Old Slug',
      });

      const result = await project.client.moveItem({
        sourceProjectPath: project.path,
        targetProjectPath: project.path,
        itemType: 'docs',
        itemId: 'old-slug',
        newId: 'new-slug',
      });

      expect(result.success).toBe(true);
      expect(result.item.id).toBe('new-slug');

      // Verify old slug no longer exists
      try {
        await project.client.getItem({
          projectPath: project.path,
          itemType: 'docs',
          itemId: 'old-slug',
        });
        expect.fail('Should have thrown an error');
      } catch {
        // Expected
      }
    });
  });

  describe('DeleteItem (docs)', () => {
    it('should delete a doc', async () => {
      await project.client.createItem({
        projectPath: project.path,
        itemType: 'docs',
        title: 'To Delete',
      });

      const deleteResult = await project.client.deleteItem({
        projectPath: project.path,
        itemType: 'docs',
        itemId: 'to-delete',
        force: true,
      });

      expect(deleteResult.success).toBe(true);

      // Verify it's gone
      const listResult = await project.client.listItems({
        projectPath: project.path,
        itemType: 'docs',
      });
      expect(listResult.totalCount).toBe(0);
    });

    it('should return error for non-existent doc', async () => {
      try {
        await project.client.deleteItem({
          projectPath: project.path,
          itemType: 'docs',
          itemId: 'non-existent',
          force: true,
        });
        expect.fail('Should have thrown an error');
      } catch (error: unknown) {
        expect(error).toBeDefined();
      }
    });
  });
});
