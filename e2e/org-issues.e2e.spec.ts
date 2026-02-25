import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { randomUUID } from 'node:crypto';
import {
  createTempProject,
  type TempProject,
  testData,
} from './fixtures/temp-project.js';

// Skip until org issues backend is implemented
describe.skip('Org Issues E2E Tests', () => {
  let project: TempProject;
  let orgSlug: string;

  beforeEach(async () => {
    project = await createTempProject({ initialize: true });

    // Create a unique organization for each test
    orgSlug = `test-org-${randomUUID().slice(0, 8)}`;
    const orgResult = await project.client.createOrganization({
      slug: orgSlug,
      name: testData.randomOrgName(),
      description: 'Test organization for E2E tests',
    });

    if (!orgResult.success) {
      throw new Error(`Failed to create organization: ${orgResult.error}`);
    }
  });

  afterEach(async () => {
    // Clean up organization
    try {
      await project.client.deleteOrganization({ slug: orgSlug });
    } catch {
      // Ignore cleanup errors
    }
    await project.cleanup();
  });

  describe('CreateOrgIssue', () => {
    it('should create an org issue with title and description', async () => {
      const result = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Test Org Issue',
        description: 'This is a test org issue',
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.id).toBeDefined();
      expect(result.displayNumber).toBe(1);
    });

    it('should assign sequential display numbers', async () => {
      const first = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'First Issue',
      });
      expect(first.displayNumber).toBe(1);

      const second = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Second Issue',
      });
      expect(second.displayNumber).toBe(2);

      const third = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Third Issue',
      });
      expect(third.displayNumber).toBe(3);
    });

    it('should create issue with custom priority and status', async () => {
      const result = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'High Priority Issue',
        priority: 1,
        status: 'in-progress',
      });

      expect(result.success).toBe(true);

      const issue = await project.client.getOrgIssue({
        organizationSlug: orgSlug,
        issueId: result.id,
      });

      expect(issue.metadata.priority).toBe(1);
      expect(issue.metadata.status).toBe('in-progress');
    });

    it('should create issue with referenced projects', async () => {
      const result = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Cross-Project Issue',
        referencedProjects: [project.path],
      });

      expect(result.success).toBe(true);

      const issue = await project.client.getOrgIssue({
        organizationSlug: orgSlug,
        issueId: result.id,
      });

      expect(issue.metadata.referencedProjects).toContain(project.path);
    });

    it('should create issue with custom fields', async () => {
      const result = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Issue with Custom Fields',
        customFields: {
          team: 'backend',
          sprint: '2024-Q1',
        },
      });

      expect(result.success).toBe(true);

      const issue = await project.client.getOrgIssue({
        organizationSlug: orgSlug,
        issueId: result.id,
      });

      expect(issue.metadata.customFields['team']).toBe('backend');
      expect(issue.metadata.customFields['sprint']).toBe('2024-Q1');
    });
  });

  describe('GetOrgIssue', () => {
    it('should get org issue by UUID', async () => {
      const created = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Get By ID Test',
        description: 'Test description',
      });

      const issue = await project.client.getOrgIssue({
        organizationSlug: orgSlug,
        issueId: created.id,
      });

      expect(issue.id).toBe(created.id);
      expect(issue.title).toBe('Get By ID Test');
      expect(issue.description).toContain('Test description');
      expect(issue.displayNumber).toBe(1);
    });

    it('should throw error for non-existent issue', async () => {
      try {
        await project.client.getOrgIssue({
          organizationSlug: orgSlug,
          issueId: 'non-existent-uuid',
        });
        expect.fail('Should have thrown an error');
      } catch (error: unknown) {
        expect(error).toBeDefined();
      }
    });
  });

  describe('GetOrgIssueByDisplayNumber', () => {
    it('should get org issue by display number', async () => {
      const created = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Display Number Test',
      });

      const issue = await project.client.getOrgIssueByDisplayNumber({
        organizationSlug: orgSlug,
        displayNumber: created.displayNumber,
      });

      expect(issue.id).toBe(created.id);
      expect(issue.displayNumber).toBe(created.displayNumber);
      expect(issue.title).toBe('Display Number Test');
    });

    it('should throw error for non-existent display number', async () => {
      try {
        await project.client.getOrgIssueByDisplayNumber({
          organizationSlug: orgSlug,
          displayNumber: 999,
        });
        expect.fail('Should have thrown an error');
      } catch (error: unknown) {
        expect(error).toBeDefined();
      }
    });
  });

  describe('ListOrgIssues', () => {
    beforeEach(async () => {
      // Create multiple issues with different statuses
      await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Open Issue',
        status: 'open',
        priority: 1,
      });

      await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'In Progress Issue',
        status: 'in-progress',
        priority: 2,
      });

      await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Closed Issue',
        status: 'closed',
        priority: 3,
      });
    });

    it('should list all org issues', async () => {
      const result = await project.client.listOrgIssues({
        organizationSlug: orgSlug,
      });

      expect(result.issues.length).toBe(3);
      expect(result.totalCount).toBe(3);
    });

    it('should filter issues by status', async () => {
      const result = await project.client.listOrgIssues({
        organizationSlug: orgSlug,
        status: 'open',
      });

      expect(result.issues.length).toBe(1);
      expect(result.issues[0].title).toBe('Open Issue');
    });

    it('should filter issues by priority', async () => {
      const result = await project.client.listOrgIssues({
        organizationSlug: orgSlug,
        priority: 1,
      });

      expect(result.issues.length).toBe(1);
      expect(result.issues[0].title).toBe('Open Issue');
    });

    it('should return empty list when no matches', async () => {
      const result = await project.client.listOrgIssues({
        organizationSlug: orgSlug,
        status: 'non-existent-status',
      });

      expect(result.issues.length).toBe(0);
      expect(result.totalCount).toBe(0);
    });
  });

  describe('ListOrgIssues with Referenced Projects', () => {
    it('should filter by referenced project', async () => {
      // Create issue without project reference
      await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'No Reference',
      });

      // Create issue with project reference
      await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'With Reference',
        referencedProjects: [project.path],
      });

      const result = await project.client.listOrgIssues({
        organizationSlug: orgSlug,
        referencedProject: project.path,
      });

      expect(result.issues.length).toBe(1);
      expect(result.issues[0].title).toBe('With Reference');
    });
  });

  describe('UpdateOrgIssue', () => {
    let issueId: string;

    beforeEach(async () => {
      const created = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Original Title',
        description: 'Original description',
        status: 'open',
      });
      issueId = created.id;
    });

    it('should update issue title', async () => {
      const result = await project.client.updateOrgIssue({
        organizationSlug: orgSlug,
        issueId,
        title: 'Updated Title',
      });

      expect(result.success).toBe(true);
      expect(result.issue?.title).toBe('Updated Title');
    });

    it('should update issue description', async () => {
      const result = await project.client.updateOrgIssue({
        organizationSlug: orgSlug,
        issueId,
        description: 'Updated description',
      });

      expect(result.success).toBe(true);
      expect(result.issue?.description).toContain('Updated description');
    });

    it('should update issue status', async () => {
      const result = await project.client.updateOrgIssue({
        organizationSlug: orgSlug,
        issueId,
        status: 'in-progress',
      });

      expect(result.success).toBe(true);
      expect(result.issue?.metadata.status).toBe('in-progress');
    });

    it('should update issue priority', async () => {
      const result = await project.client.updateOrgIssue({
        organizationSlug: orgSlug,
        issueId,
        priority: 1,
      });

      expect(result.success).toBe(true);
      expect(result.issue?.metadata.priority).toBe(1);
    });

    it('should add referenced projects', async () => {
      const result = await project.client.updateOrgIssue({
        organizationSlug: orgSlug,
        issueId,
        addReferencedProjects: [project.path],
      });

      expect(result.success).toBe(true);
      expect(result.issue?.metadata.referencedProjects).toContain(project.path);
    });

    it('should remove referenced projects', async () => {
      // First add a reference
      await project.client.updateOrgIssue({
        organizationSlug: orgSlug,
        issueId,
        addReferencedProjects: [project.path],
      });

      // Then remove it
      const result = await project.client.updateOrgIssue({
        organizationSlug: orgSlug,
        issueId,
        removeReferencedProjects: [project.path],
      });

      expect(result.success).toBe(true);
      expect(result.issue?.metadata.referencedProjects).not.toContain(project.path);
    });

    it('should update multiple fields at once', async () => {
      const result = await project.client.updateOrgIssue({
        organizationSlug: orgSlug,
        issueId,
        title: 'New Title',
        description: 'New description',
        status: 'closed',
        priority: 1,
      });

      expect(result.success).toBe(true);
      expect(result.issue?.title).toBe('New Title');
      expect(result.issue?.description).toContain('New description');
      expect(result.issue?.metadata.status).toBe('closed');
      expect(result.issue?.metadata.priority).toBe(1);
    });
  });

  describe('DeleteOrgIssue', () => {
    it('should delete an org issue', async () => {
      const created = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'To Be Deleted',
      });

      const deleteResult = await project.client.deleteOrgIssue({
        organizationSlug: orgSlug,
        issueId: created.id,
      });

      expect(deleteResult.success).toBe(true);

      // Verify issue is deleted
      try {
        await project.client.getOrgIssue({
          organizationSlug: orgSlug,
          issueId: created.id,
        });
        expect.fail('Should have thrown an error');
      } catch (error: unknown) {
        expect(error).toBeDefined();
      }
    });

    it('should fail when deleting non-existent issue', async () => {
      try {
        await project.client.deleteOrgIssue({
          organizationSlug: orgSlug,
          issueId: 'non-existent-uuid',
        });
        expect.fail('Should have thrown an error');
      } catch (error: unknown) {
        expect(error).toBeDefined();
      }
    });
  });

  describe('OrgConfig', () => {
    it('should get org config', async () => {
      const config = await project.client.getOrgConfig({
        organizationSlug: orgSlug,
      });

      expect(config.priorityLevels).toBeGreaterThan(0);
    });

    it('should update org config priority levels', async () => {
      const result = await project.client.updateOrgConfig({
        organizationSlug: orgSlug,
        config: {
          priorityLevels: 5,
        },
      });

      expect(result.success).toBe(true);
      expect(result.config?.priorityLevels).toBe(5);
    });
  });

  describe('Org Issue Metadata Validation', () => {
    it('should have valid timestamps', async () => {
      const created = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Timestamp Test',
      });

      const issue = await project.client.getOrgIssue({
        organizationSlug: orgSlug,
        issueId: created.id,
      });

      expect(() => new Date(issue.metadata.createdAt)).not.toThrow();
      expect(() => new Date(issue.metadata.updatedAt)).not.toThrow();
    });

    it('should have priority label', async () => {
      const created = await project.client.createOrgIssue({
        organizationSlug: orgSlug,
        title: 'Priority Label Test',
        priority: 1,
      });

      const issue = await project.client.getOrgIssue({
        organizationSlug: orgSlug,
        issueId: created.id,
      });

      expect(issue.metadata.priorityLabel).toBeDefined();
      expect(issue.metadata.priorityLabel.length).toBeGreaterThan(0);
    });
  });
});
