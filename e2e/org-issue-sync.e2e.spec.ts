import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { randomUUID } from 'node:crypto';
import {
  createTempProject,
  type TempProject,
  testData,
} from './fixtures/temp-project.js';

/**
 * Tests for Org Issue Sync functionality.
 *
 * TODO: These tests use org-specific fields (isOrgIssue, orgDisplayNumber,
 * syncResults) that were part of the per-entity CreateIssue RPC which has been
 * removed in favour of the generic CreateItem RPC.  They need to be
 * re-implemented once org-issue functionality is exposed through the generic
 * item API (e.g. via custom_fields or a dedicated org RPC).
 */
// Skip until org-issue functionality is added to the generic item API
describe.skip('Org Issue Sync E2E Tests', () => {
  let project1: TempProject;
  let project2: TempProject;
  let orgSlug: string;

  beforeEach(async () => {
    // Create unique organization for this test
    orgSlug = `test-org-${randomUUID().slice(0, 8)}`;

    // Create two projects
    project1 = await createTempProject({ initialize: true });
    project2 = await createTempProject({ initialize: true });

    // Create the organization
    const orgResult = await project1.client.createOrganization({
      slug: orgSlug,
      name: testData.randomOrgName(),
      description: 'Test organization for org issue sync tests',
    });

    if (!orgResult.success) {
      throw new Error(`Failed to create organization: ${orgResult.error}`);
    }

    // Associate both projects with the organization
    const setOrg1 = await project1.client.setProjectOrganization({
      projectPath: project1.path,
      organizationSlug: orgSlug,
    });

    if (!setOrg1.success) {
      throw new Error(`Failed to set project1 organization: ${setOrg1.error}`);
    }

    const setOrg2 = await project2.client.setProjectOrganization({
      projectPath: project2.path,
      organizationSlug: orgSlug,
    });

    if (!setOrg2.success) {
      throw new Error(`Failed to set project2 organization: ${setOrg2.error}`);
    }
  });

  afterEach(async () => {
    // Cleanup
    try {
      await project1.client.deleteOrganization({ slug: orgSlug });
    } catch {
      // Ignore cleanup errors
    }
    await project1.cleanup();
    await project2.cleanup();
  });

  describe('CreateItem with isOrgIssue (via custom_fields)', () => {
    it('should create an org issue with org metadata', async () => {
      // TODO: pass isOrgIssue via custom_fields once supported
      const result = await project1.client.createItem({
        projectPath: project1.path,
        itemType: 'issues',
        title: 'Org Issue Test',
        body: 'Testing org issue creation',
      });

      expect(result.success).toBe(true);
      expect(result.item.id).toBeDefined();
      expect(result.item.metadata.displayNumber).toBe(1);
      // TODO: expect orgDisplayNumber to be 1 once org fields are in generic API
    });

    it('should assign sequential org display numbers across projects', async () => {
      // TODO: implement once org-specific fields are in generic API
      const issue1 = await project1.client.createItem({
        projectPath: project1.path,
        itemType: 'issues',
        title: 'First Org Issue',
      });
      expect(issue1.item.metadata.displayNumber).toBe(1);

      const issue2 = await project2.client.createItem({
        projectPath: project2.path,
        itemType: 'issues',
        title: 'Second Org Issue',
      });
      expect(issue2.item.metadata.displayNumber).toBe(1);
      // TODO: expect issue2 orgDisplayNumber to be 2
    });

    it('should return sync results', async () => {
      // TODO: sync results not available in generic CreateItemResponse
      const result = await project1.client.createItem({
        projectPath: project1.path,
        itemType: 'issues',
        title: 'Synced Org Issue',
      });

      expect(result.success).toBe(true);
      // TODO: check result.syncResults once available
    });

    it('should include org metadata in retrieved issue', async () => {
      const created = await project1.client.createItem({
        projectPath: project1.path,
        itemType: 'issues',
        title: 'Org Metadata Test',
      });

      const result = await project1.client.getItem({
        projectPath: project1.path,
        itemType: 'issues',
        itemId: created.item.id,
      });

      expect(result.item.id).toBe(created.item.id);
      // TODO: check isOrgIssue, orgSlug, orgDisplayNumber once available
    });
  });

  describe('Non-org issues', () => {
    it('should not have org metadata for regular issues', async () => {
      const result = await project1.client.createItem({
        projectPath: project1.path,
        itemType: 'issues',
        title: 'Regular Issue',
        body: 'This is a regular issue',
      });

      expect(result.success).toBe(true);
      expect(result.item.metadata.displayNumber).toBe(1);
      // TODO: check orgDisplayNumber is 0 / unset once org fields are in API

      const getResult = await project1.client.getItem({
        projectPath: project1.path,
        itemType: 'issues',
        itemId: result.item.id,
      });

      expect(getResult.item.id).toBe(result.item.id);
      // TODO: check isOrgIssue is falsy once org fields are in generic API
    });

    it('should not sync regular issues', async () => {
      const result = await project1.client.createItem({
        projectPath: project1.path,
        itemType: 'issues',
        title: 'Non-synced Issue',
      });

      expect(result.success).toBe(true);
      // TODO: check syncResults is empty once available in generic API
    });
  });

  describe('Error handling', () => {
    it('should fail to create org issue for project without organization', async () => {
      // Create a project without an organization
      const unaffiliatedProject = await createTempProject({ initialize: true });

      try {
        // TODO: once isOrgIssue is supported in generic API, pass it here
        const result = await unaffiliatedProject.client.createItem({
          projectPath: unaffiliatedProject.path,
          itemType: 'issues',
          title: 'Orphan Org Issue',
        });

        // For now, regular createItem succeeds even without an org
        expect(result.success).toBe(true);
        // TODO: should fail with error about organization once isOrgIssue is supported
      } finally {
        await unaffiliatedProject.cleanup();
      }
    });
  });
});
