import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import {
  createTempProject,
  type TempProject,
  testData,
} from './fixtures/temp-project.js';
import { LinkTargetType } from './fixtures/grpc-client.js';

describe('Links E2E Tests', () => {
  let project: TempProject;
  let issueId1: string;
  let issueId2: string;

  beforeEach(async () => {
    project = await createTempProject({ initialize: true });

    // Create two issues for linking tests
    const issue1 = await project.client.createIssue({
      projectPath: project.path,
      title: 'Issue 1',
      description: 'First test issue',
    });
    issueId1 = issue1.id;

    const issue2 = await project.client.createIssue({
      projectPath: project.path,
      title: 'Issue 2',
      description: 'Second test issue',
    });
    issueId2 = issue2.id;
  });

  afterEach(async () => {
    await project.cleanup();
  });

  describe('CreateLink', () => {
    it('should create a link between two issues', async () => {
      const result = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.createdLink).toBeDefined();
      expect(result.createdLink?.targetId).toBe(issueId2);
      expect(result.createdLink?.linkType).toBe('blocks');
    });

    it('should create inverse link automatically', async () => {
      const result = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      expect(result.success).toBe(true);
      expect(result.inverseLink).toBeDefined();
      expect(result.inverseLink?.targetId).toBe(issueId1);
      expect(result.inverseLink?.linkType).toBe('blocked-by');
    });

    it('should create parent-child links', async () => {
      const result = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'parent-of',
      });

      expect(result.success).toBe(true);
      expect(result.createdLink?.linkType).toBe('parent-of');
      expect(result.inverseLink?.linkType).toBe('child-of');
    });

    it('should create relates-to links', async () => {
      const result = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'relates-to',
      });

      expect(result.success).toBe(true);
      expect(result.createdLink?.linkType).toBe('relates-to');
      expect(result.inverseLink?.linkType).toBe('related-from');
    });

    it('should create duplicates links', async () => {
      const result = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'duplicates',
      });

      expect(result.success).toBe(true);
      expect(result.createdLink?.linkType).toBe('duplicates');
      expect(result.inverseLink?.linkType).toBe('duplicated-by');
    });

    it('should fail when linking entity to itself', async () => {
      const result = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId1,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      expect(result.success).toBe(false);
      expect(result.error).not.toBe('');
    });

    it('should fail when creating duplicate link', async () => {
      // Create first link
      await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      // Try to create duplicate
      const result = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      expect(result.success).toBe(false);
      expect(result.error).not.toBe('');
    });

    it('should fail for invalid link type', async () => {
      const result = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'invalid-link-type',
      });

      expect(result.success).toBe(false);
      expect(result.error).not.toBe('');
    });

    it('should have valid timestamp on created link', async () => {
      const result = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      expect(result.success).toBe(true);
      expect(result.createdLink?.createdAt).toBeDefined();
      expect(() => new Date(result.createdLink!.createdAt)).not.toThrow();
    });
  });

  describe('CreateLink - Cross Entity Types', () => {
    let docSlug: string;

    beforeEach(async () => {
      // Create a doc for cross-entity linking
      const docResult = await project.client.createDoc({
        projectPath: project.path,
        title: 'Test Documentation',
        content: 'This is test documentation content.',
      });
      docSlug = docResult.slug;
    });

    it('should link issue to doc', async () => {
      const result = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: docSlug,
        targetType: LinkTargetType.DOC,
        linkType: 'relates-to',
      });

      expect(result.success).toBe(true);
      expect(result.createdLink?.targetId).toBe(docSlug);
      expect(result.createdLink?.targetType).toBe('LINK_TARGET_TYPE_DOC');
    });
  });

  describe('ListLinks', () => {
    beforeEach(async () => {
      // Create multiple links from issue1
      await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      // Create a third issue and link it
      const issue3 = await project.client.createIssue({
        projectPath: project.path,
        title: 'Issue 3',
      });

      await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issue3.id,
        targetType: LinkTargetType.ISSUE,
        linkType: 'parent-of',
      });
    });

    it('should list all links for an entity', async () => {
      const result = await project.client.listLinks({
        projectPath: project.path,
        entityId: issueId1,
        entityType: LinkTargetType.ISSUE,
      });

      expect(result.links.length).toBe(2);
      expect(result.totalCount).toBe(2);
    });

    it('should include inverse links in list', async () => {
      // Issue2 should have a "blocked-by" link to Issue1
      const result = await project.client.listLinks({
        projectPath: project.path,
        entityId: issueId2,
        entityType: LinkTargetType.ISSUE,
      });

      expect(result.links.length).toBe(1);
      expect(result.links[0].linkType).toBe('blocked-by');
      expect(result.links[0].targetId).toBe(issueId1);
    });

    it('should return empty list when no links exist', async () => {
      // Create an issue with no links
      const newIssue = await project.client.createIssue({
        projectPath: project.path,
        title: 'No Links Issue',
      });

      const result = await project.client.listLinks({
        projectPath: project.path,
        entityId: newIssue.id,
        entityType: LinkTargetType.ISSUE,
      });

      expect(result.links.length).toBe(0);
      expect(result.totalCount).toBe(0);
    });
  });

  describe('DeleteLink', () => {
    beforeEach(async () => {
      // Create a link to delete
      await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });
    });

    it('should delete a specific link type', async () => {
      const result = await project.client.deleteLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      expect(result.success).toBe(true);
      expect(result.deletedCount).toBeGreaterThan(0);

      // Verify link is deleted
      const links = await project.client.listLinks({
        projectPath: project.path,
        entityId: issueId1,
        entityType: LinkTargetType.ISSUE,
      });

      expect(links.links.length).toBe(0);
    });

    it('should delete inverse link when forward link is deleted', async () => {
      await project.client.deleteLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      // Verify inverse link is also deleted
      const links = await project.client.listLinks({
        projectPath: project.path,
        entityId: issueId2,
        entityType: LinkTargetType.ISSUE,
      });

      expect(links.links.length).toBe(0);
    });

    it('should delete all links between entities when no link type specified', async () => {
      // Add another link type
      await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'relates-to',
      });

      // Delete all links without specifying type
      const result = await project.client.deleteLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
      });

      expect(result.success).toBe(true);
      expect(result.deletedCount).toBeGreaterThanOrEqual(2);

      // Verify all links are deleted
      const links = await project.client.listLinks({
        projectPath: project.path,
        entityId: issueId1,
        entityType: LinkTargetType.ISSUE,
      });

      const linksToIssue2 = links.links.filter(l => l.targetId === issueId2);
      expect(linksToIssue2.length).toBe(0);
    });

    it('should return 0 deleted count when link does not exist', async () => {
      // Delete the existing link first
      await project.client.deleteLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      // Try to delete again
      const result = await project.client.deleteLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      expect(result.success).toBe(true);
      expect(result.deletedCount).toBe(0);
    });
  });

  describe('GetAvailableLinkTypes', () => {
    it('should return built-in link types', async () => {
      const result = await project.client.getAvailableLinkTypes({
        projectPath: project.path,
      });

      expect(result.linkTypes.length).toBeGreaterThan(0);

      // Check for built-in types
      const linkTypeNames = result.linkTypes.map(lt => lt.name);
      expect(linkTypeNames).toContain('blocks');
      expect(linkTypeNames).toContain('parent-of');
      expect(linkTypeNames).toContain('relates-to');
      expect(linkTypeNames).toContain('duplicates');
    });

    it('should include inverse names for each link type', async () => {
      const result = await project.client.getAvailableLinkTypes({
        projectPath: project.path,
      });

      const blocksType = result.linkTypes.find(lt => lt.name === 'blocks');
      expect(blocksType).toBeDefined();
      expect(blocksType?.inverse).toBe('blocked-by');

      const parentOfType = result.linkTypes.find(lt => lt.name === 'parent-of');
      expect(parentOfType).toBeDefined();
      expect(parentOfType?.inverse).toBe('child-of');
    });

    it('should mark built-in types correctly', async () => {
      const result = await project.client.getAvailableLinkTypes({
        projectPath: project.path,
      });

      const builtinTypes = result.linkTypes.filter(lt => lt.isBuiltin);
      expect(builtinTypes.length).toBeGreaterThan(0);

      // All standard types should be marked as builtin
      const blocksType = result.linkTypes.find(lt => lt.name === 'blocks');
      expect(blocksType?.isBuiltin).toBe(true);
    });
  });

  describe('Link Persistence', () => {
    it('should persist links across reads', async () => {
      // Create a link
      await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });

      // Read links multiple times
      const result1 = await project.client.listLinks({
        projectPath: project.path,
        entityId: issueId1,
        entityType: LinkTargetType.ISSUE,
      });

      const result2 = await project.client.listLinks({
        projectPath: project.path,
        entityId: issueId1,
        entityType: LinkTargetType.ISSUE,
      });

      expect(result1.links.length).toBe(result2.links.length);
      expect(result1.links[0].targetId).toBe(result2.links[0].targetId);
    });
  });

  describe('Multiple Link Types Between Same Entities', () => {
    it('should allow multiple different link types between same entities', async () => {
      // Create first link
      const result1 = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'blocks',
      });
      expect(result1.success).toBe(true);

      // Create second link with different type
      const result2 = await project.client.createLink({
        projectPath: project.path,
        sourceId: issueId1,
        sourceType: LinkTargetType.ISSUE,
        targetId: issueId2,
        targetType: LinkTargetType.ISSUE,
        linkType: 'relates-to',
      });
      expect(result2.success).toBe(true);

      // Verify both links exist
      const links = await project.client.listLinks({
        projectPath: project.path,
        entityId: issueId1,
        entityType: LinkTargetType.ISSUE,
      });

      const linkTypes = links.links.map(l => l.linkType);
      expect(linkTypes).toContain('blocks');
      expect(linkTypes).toContain('relates-to');
    });
  });
});
