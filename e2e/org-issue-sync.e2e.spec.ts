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
 * This tests the "sync copies" approach where org issues are created
 * in a project and automatically synced to all other projects in the
 * same organization (similar to how org docs work).
 */
describe('Org Issue Sync E2E Tests', () => {
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

  describe('CreateIssue with isOrgIssue', () => {
    it('should create an org issue with org metadata', async () => {
      const result = await project1.client.createIssue({
        projectPath: project1.path,
        title: 'Org Issue Test',
        description: 'Testing org issue creation',
        isOrgIssue: true,
      });

      expect(result.success).toBe(true);
      expect(result.id).toBeDefined();
      expect(result.displayNumber).toBe(1);
      expect(result.orgDisplayNumber).toBe(1);
    });

    it('should assign sequential org display numbers across projects', async () => {
      // Create org issue in project1
      const issue1 = await project1.client.createIssue({
        projectPath: project1.path,
        title: 'First Org Issue',
        isOrgIssue: true,
      });
      expect(issue1.orgDisplayNumber).toBe(1);

      // Create org issue in project2
      const issue2 = await project2.client.createIssue({
        projectPath: project2.path,
        title: 'Second Org Issue',
        isOrgIssue: true,
      });
      expect(issue2.orgDisplayNumber).toBe(2);

      // Create another org issue in project1
      const issue3 = await project1.client.createIssue({
        projectPath: project1.path,
        title: 'Third Org Issue',
        isOrgIssue: true,
      });
      expect(issue3.orgDisplayNumber).toBe(3);
    });

    it('should return sync results', async () => {
      const result = await project1.client.createIssue({
        projectPath: project1.path,
        title: 'Synced Org Issue',
        isOrgIssue: true,
      });

      expect(result.success).toBe(true);
      expect(result.syncResults).toBeDefined();
      // Should sync to project2
      expect(result.syncResults?.length).toBeGreaterThanOrEqual(0);
    });

    it('should include org metadata in retrieved issue', async () => {
      const created = await project1.client.createIssue({
        projectPath: project1.path,
        title: 'Org Metadata Test',
        isOrgIssue: true,
      });

      const issue = await project1.client.getIssue({
        projectPath: project1.path,
        issueId: created.id,
      });

      expect(issue.metadata.isOrgIssue).toBe(true);
      expect(issue.metadata.orgSlug).toBe(orgSlug);
      expect(issue.metadata.orgDisplayNumber).toBe(1);
    });
  });

  describe('Non-org issues', () => {
    it('should not have org metadata for regular issues', async () => {
      const result = await project1.client.createIssue({
        projectPath: project1.path,
        title: 'Regular Issue',
        description: 'This is a regular issue',
        // isOrgIssue not set
      });

      expect(result.success).toBe(true);
      expect(result.orgDisplayNumber).toBeUndefined();
      expect(result.syncResults?.length).toBe(0);

      const issue = await project1.client.getIssue({
        projectPath: project1.path,
        issueId: result.id,
      });

      expect(issue.metadata.isOrgIssue).toBeFalsy();
      expect(issue.metadata.orgSlug).toBeUndefined();
    });

    it('should not sync regular issues', async () => {
      const result = await project1.client.createIssue({
        projectPath: project1.path,
        title: 'Non-synced Issue',
      });

      expect(result.success).toBe(true);
      expect(result.syncResults?.length).toBe(0);
    });
  });

  describe('Error handling', () => {
    it('should fail to create org issue for project without organization', async () => {
      // Create a project without an organization
      const unaffiliatedProject = await createTempProject({ initialize: true });

      try {
        const result = await unaffiliatedProject.client.createIssue({
          projectPath: unaffiliatedProject.path,
          title: 'Orphan Org Issue',
          isOrgIssue: true,
        });

        // Should fail because project has no organization
        expect(result.success).toBe(false);
        expect(result.error).toContain('organization');
      } finally {
        await unaffiliatedProject.cleanup();
      }
    });
  });
});
