import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import {
  createTempGitProject,
  createGitBranch,
  switchGitBranch,
  getCurrentGitBranch,
  type TempProject,
  testData,
} from './fixtures/temp-project.js';

describe('PR E2E Tests', () => {
  let project: TempProject;

  beforeEach(async () => {
    project = await createTempGitProject({ initialize: true });
  });

  afterEach(async () => {
    await project.cleanup();
  });

  describe('CreatePr', () => {
    it('should create a PR with title and description', async () => {
      // Create a feature branch
      createGitBranch(project.path, 'feature/test');

      const result = await project.client.createPr({
        projectPath: project.path,
        title: 'Add new feature',
        description: 'This PR adds a new feature',
        sourceBranch: 'feature/test',
        targetBranch: 'main',
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.id).toBeDefined();
      expect(result.displayNumber).toBe(1);
      expect(result.detectedSourceBranch).toBe('feature/test');
    });

    it('should auto-detect current branch as source', async () => {
      // Create and switch to feature branch
      createGitBranch(project.path, 'feature/auto-detect');

      const result = await project.client.createPr({
        projectPath: project.path,
        title: 'Auto-detect branch PR',
      });

      expect(result.success).toBe(true);
      expect(result.detectedSourceBranch).toBe('feature/auto-detect');
    });

    it('should assign sequential display numbers', async () => {
      createGitBranch(project.path, 'feature/first');
      const first = await project.client.createPr({
        projectPath: project.path,
        title: 'First PR',
        sourceBranch: 'feature/first',
      });
      expect(first.displayNumber).toBe(1);

      switchGitBranch(project.path, 'main');
      createGitBranch(project.path, 'feature/second');
      const second = await project.client.createPr({
        projectPath: project.path,
        title: 'Second PR',
        sourceBranch: 'feature/second',
      });
      expect(second.displayNumber).toBe(2);

      switchGitBranch(project.path, 'main');
      createGitBranch(project.path, 'feature/third');
      const third = await project.client.createPr({
        projectPath: project.path,
        title: 'Third PR',
        sourceBranch: 'feature/third',
      });
      expect(third.displayNumber).toBe(3);
    });

    it('should create PR with custom priority and status', async () => {
      createGitBranch(project.path, 'feature/priority');

      const result = await project.client.createPr({
        projectPath: project.path,
        title: 'High priority PR',
        priority: 1,
        status: 'open',
        sourceBranch: 'feature/priority',
      });

      expect(result.success).toBe(true);

      // Verify the PR was created with correct metadata
      const pr = await project.client.getPr({
        projectPath: project.path,
        prId: result.id,
      });

      expect(pr.metadata.priority).toBe(1);
      expect(pr.metadata.status).toBe('open');
    });

    it('should create PR with reviewers', async () => {
      createGitBranch(project.path, 'feature/reviewers');

      const result = await project.client.createPr({
        projectPath: project.path,
        title: 'PR with reviewers',
        sourceBranch: 'feature/reviewers',
        reviewers: ['alice', 'bob'],
      });

      expect(result.success).toBe(true);

      const pr = await project.client.getPr({
        projectPath: project.path,
        prId: result.id,
      });

      expect(pr.metadata.reviewers).toContain('alice');
      expect(pr.metadata.reviewers).toContain('bob');
    });
  });

  describe('GetPr', () => {
    it('should get PR by UUID', async () => {
      createGitBranch(project.path, 'feature/get-test');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'Get By ID Test',
        description: 'Test description',
        sourceBranch: 'feature/get-test',
      });

      const pr = await project.client.getPr({
        projectPath: project.path,
        prId: created.id,
      });

      expect(pr.id).toBe(created.id);
      expect(pr.title).toBe('Get By ID Test');
      expect(pr.description).toContain('Test description');
      expect(pr.displayNumber).toBe(1);
    });

    it('should throw error for non-existent PR', async () => {
      try {
        await project.client.getPr({
          projectPath: project.path,
          prId: 'non-existent-uuid',
        });
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error).toBeDefined();
      }
    });
  });

  describe('GetPrByDisplayNumber', () => {
    it('should get PR by display number', async () => {
      createGitBranch(project.path, 'feature/display-num');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'Display Number Test',
        sourceBranch: 'feature/display-num',
      });

      const pr = await project.client.getPrByDisplayNumber({
        projectPath: project.path,
        displayNumber: created.displayNumber,
      });

      expect(pr.id).toBe(created.id);
      expect(pr.displayNumber).toBe(created.displayNumber);
      expect(pr.title).toBe('Display Number Test');
    });

    it('should throw error for non-existent display number', async () => {
      try {
        await project.client.getPrByDisplayNumber({
          projectPath: project.path,
          displayNumber: 999,
        });
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error).toBeDefined();
      }
    });
  });

  describe('ListPrs', () => {
    beforeEach(async () => {
      // Create multiple PRs with different statuses
      createGitBranch(project.path, 'feature/draft');
      await project.client.createPr({
        projectPath: project.path,
        title: 'Draft PR',
        status: 'draft',
        sourceBranch: 'feature/draft',
      });

      switchGitBranch(project.path, 'main');
      createGitBranch(project.path, 'feature/open');
      await project.client.createPr({
        projectPath: project.path,
        title: 'Open PR',
        status: 'open',
        sourceBranch: 'feature/open',
      });

      switchGitBranch(project.path, 'main');
      createGitBranch(project.path, 'feature/merged');
      await project.client.createPr({
        projectPath: project.path,
        title: 'Merged PR',
        status: 'merged',
        sourceBranch: 'feature/merged',
      });
    });

    it('should list all PRs', async () => {
      const result = await project.client.listPrs({
        projectPath: project.path,
      });

      expect(result.prs.length).toBe(3);
      expect(result.totalCount).toBe(3);
    });

    it('should filter PRs by status', async () => {
      const result = await project.client.listPrs({
        projectPath: project.path,
        status: 'open',
      });

      expect(result.prs.length).toBe(1);
      expect(result.prs[0].title).toBe('Open PR');
    });

    it('should filter PRs by source branch', async () => {
      const result = await project.client.listPrs({
        projectPath: project.path,
        sourceBranch: 'feature/draft',
      });

      expect(result.prs.length).toBe(1);
      expect(result.prs[0].title).toBe('Draft PR');
    });

    it('should return empty list when no matches', async () => {
      const result = await project.client.listPrs({
        projectPath: project.path,
        status: 'closed',
      });

      expect(result.prs.length).toBe(0);
      expect(result.totalCount).toBe(0);
    });
  });

  describe('UpdatePr', () => {
    let prId: string;

    beforeEach(async () => {
      createGitBranch(project.path, 'feature/update-test');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'Original Title',
        description: 'Original description',
        status: 'draft',
        sourceBranch: 'feature/update-test',
      });
      prId = created.id;
    });

    it('should update PR title', async () => {
      const result = await project.client.updatePr({
        projectPath: project.path,
        prId,
        title: 'Updated Title',
      });

      expect(result.success).toBe(true);
      expect(result.pr?.title).toBe('Updated Title');
    });

    it('should update PR description', async () => {
      const result = await project.client.updatePr({
        projectPath: project.path,
        prId,
        description: 'Updated description',
      });

      expect(result.success).toBe(true);
      expect(result.pr?.description).toContain('Updated description');
    });

    it('should update PR status', async () => {
      const result = await project.client.updatePr({
        projectPath: project.path,
        prId,
        status: 'open',
      });

      expect(result.success).toBe(true);
      expect(result.pr?.metadata.status).toBe('open');
    });

    it('should set merged_at when status changes to merged', async () => {
      const result = await project.client.updatePr({
        projectPath: project.path,
        prId,
        status: 'merged',
      });

      expect(result.success).toBe(true);
      expect(result.pr?.metadata.status).toBe('merged');
      expect(result.pr?.metadata.mergedAt).not.toBe('');
      expect(() => new Date(result.pr!.metadata.mergedAt)).not.toThrow();
    });

    it('should set closed_at when status changes to closed', async () => {
      const result = await project.client.updatePr({
        projectPath: project.path,
        prId,
        status: 'closed',
      });

      expect(result.success).toBe(true);
      expect(result.pr?.metadata.status).toBe('closed');
      expect(result.pr?.metadata.closedAt).not.toBe('');
      expect(() => new Date(result.pr!.metadata.closedAt)).not.toThrow();
    });

    it('should update multiple fields at once', async () => {
      const result = await project.client.updatePr({
        projectPath: project.path,
        prId,
        title: 'New Title',
        description: 'New description',
        status: 'open',
        priority: 1,
      });

      expect(result.success).toBe(true);
      expect(result.pr?.title).toBe('New Title');
      expect(result.pr?.description).toContain('New description');
      expect(result.pr?.metadata.status).toBe('open');
      expect(result.pr?.metadata.priority).toBe(1);
    });
  });

  describe('DeletePr', () => {
    it('should delete a PR', async () => {
      createGitBranch(project.path, 'feature/delete-test');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'To Be Deleted',
        sourceBranch: 'feature/delete-test',
      });

      const deleteResult = await project.client.deletePr({
        projectPath: project.path,
        prId: created.id,
      });

      expect(deleteResult.success).toBe(true);

      // Verify PR is deleted
      try {
        await project.client.getPr({
          projectPath: project.path,
          prId: created.id,
        });
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error).toBeDefined();
      }
    });

    it('should fail when deleting non-existent PR', async () => {
      try {
        await project.client.deletePr({
          projectPath: project.path,
          prId: 'non-existent-uuid',
        });
        expect.fail('Should have thrown an error');
      } catch (error: any) {
        expect(error).toBeDefined();
      }
    });
  });

  describe('Display number resolution', () => {
    it('should get PR by display number string via getPr', async () => {
      createGitBranch(project.path, 'feature/dn-get');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'Display Num Get Test',
        sourceBranch: 'feature/dn-get',
      });

      const pr = await project.client.getPr({
        projectPath: project.path,
        prId: String(created.displayNumber),
      });

      expect(pr.id).toBe(created.id);
      expect(pr.title).toBe('Display Num Get Test');
    });

    it('should update PR by display number string', async () => {
      createGitBranch(project.path, 'feature/dn-update');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'Display Num Update Test',
        sourceBranch: 'feature/dn-update',
      });

      const result = await project.client.updatePr({
        projectPath: project.path,
        prId: String(created.displayNumber),
        title: 'Updated via Display Number',
      });

      expect(result.success).toBe(true);
      expect(result.pr?.title).toBe('Updated via Display Number');
    });

    it('should soft-delete PR by display number string', async () => {
      createGitBranch(project.path, 'feature/dn-soft-del');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'Display Num Soft Delete Test',
        sourceBranch: 'feature/dn-soft-del',
      });

      const result = await project.client.softDeletePr({
        projectPath: project.path,
        prId: String(created.displayNumber),
      });

      expect(result.success).toBe(true);
      expect(result.pr?.metadata.deletedAt).toBeDefined();
      expect(result.pr?.metadata.deletedAt).not.toBe('');
    });

    it('should restore PR by display number string', async () => {
      createGitBranch(project.path, 'feature/dn-restore');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'Display Num Restore Test',
        sourceBranch: 'feature/dn-restore',
      });

      // First soft-delete
      await project.client.softDeletePr({
        projectPath: project.path,
        prId: created.id,
      });

      // Then restore using display number
      const result = await project.client.restorePr({
        projectPath: project.path,
        prId: String(created.displayNumber),
      });

      expect(result.success).toBe(true);
      expect(result.pr?.metadata.deletedAt).toBe('');
    });

    it('should delete PR by display number string', async () => {
      createGitBranch(project.path, 'feature/dn-delete');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'Display Num Delete Test',
        sourceBranch: 'feature/dn-delete',
      });

      const deleteResult = await project.client.deletePr({
        projectPath: project.path,
        prId: String(created.displayNumber),
      });

      expect(deleteResult.success).toBe(true);

      // Verify it's gone
      const listResult = await project.client.listPrs({
        projectPath: project.path,
      });
      expect(listResult.totalCount).toBe(0);
    });
  });

  describe('GetNextPrNumber', () => {
    it('should return 1 for empty project', async () => {
      const result = await project.client.getNextPrNumber({
        projectPath: project.path,
      });

      expect(result.nextNumber).toBe(1);
    });

    it('should return next sequential number', async () => {
      createGitBranch(project.path, 'feature/first');
      await project.client.createPr({
        projectPath: project.path,
        title: 'First PR',
        sourceBranch: 'feature/first',
      });

      switchGitBranch(project.path, 'main');
      createGitBranch(project.path, 'feature/second');
      await project.client.createPr({
        projectPath: project.path,
        title: 'Second PR',
        sourceBranch: 'feature/second',
      });

      const result = await project.client.getNextPrNumber({
        projectPath: project.path,
      });

      expect(result.nextNumber).toBe(3);
    });
  });

  describe('PR Metadata Validation', () => {
    it('should have valid timestamps', async () => {
      createGitBranch(project.path, 'feature/timestamps');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'Timestamp Test',
        sourceBranch: 'feature/timestamps',
      });

      const pr = await project.client.getPr({
        projectPath: project.path,
        prId: created.id,
      });

      expect(() => new Date(pr.metadata.createdAt)).not.toThrow();
      expect(() => new Date(pr.metadata.updatedAt)).not.toThrow();
    });

    it('should have priority label', async () => {
      createGitBranch(project.path, 'feature/priority-label');

      const created = await project.client.createPr({
        projectPath: project.path,
        title: 'Priority Label Test',
        priority: 1,
        sourceBranch: 'feature/priority-label',
      });

      const pr = await project.client.getPr({
        projectPath: project.path,
        prId: created.id,
      });

      expect(pr.metadata.priorityLabel).toBeDefined();
      expect(pr.metadata.priorityLabel.length).toBeGreaterThan(0);
    });
  });
});
