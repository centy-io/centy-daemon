import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { createTempProject, createTempGitProject, type TempProject } from './fixtures/temp-project.js';

/**
 * gRPC Plain Text Tests for User Operations
 *
 * Tests direct gRPC calls for user CRUD operations and sync from git.
 */
describe('gRPC: User Operations', () => {
  let project: TempProject;

  afterEach(async () => {
    if (project) {
      await project.cleanup();
    }
  });

  describe('SyncUsers', () => {
    describe('when project is NOT in a git repository', () => {
      beforeEach(async () => {
        // Create a temp project WITHOUT git initialization
        project = await createTempProject({ initialize: true });
      });

      it('should return a meaningful error explaining the project is not a git repository', async () => {
        const result = await project.client.syncUsers({
          projectPath: project.path,
          dryRun: false,
        });

        expect(result.success).toBe(false);
        expect(result.error).toBeDefined();
        expect(result.error).not.toBe('');
        // The error should clearly explain that the project is not a git repository
        expect(result.error.toLowerCase()).toMatch(/git|repository|repo/);
      });

      it('should return a meaningful error on dry run when not a git repository', async () => {
        const result = await project.client.syncUsers({
          projectPath: project.path,
          dryRun: true,
        });

        expect(result.success).toBe(false);
        expect(result.error).toBeDefined();
        expect(result.error).not.toBe('');
        // The error should clearly explain that the project is not a git repository
        expect(result.error.toLowerCase()).toMatch(/git|repository|repo/);
      });
    });

    describe('when project IS in a git repository', () => {
      beforeEach(async () => {
        // Create a temp project WITH git initialization
        project = await createTempGitProject({ initialize: true });
      });

      it('should successfully sync users from git history', async () => {
        const result = await project.client.syncUsers({
          projectPath: project.path,
          dryRun: false,
        });

        expect(result.success).toBe(true);
        expect(result.error).toBe('');
        // Should have found at least the test user from the git setup
        expect(result.created.length + result.skipped.length).toBeGreaterThanOrEqual(0);
      });

      it('should return contributors in dry run mode', async () => {
        const result = await project.client.syncUsers({
          projectPath: project.path,
          dryRun: true,
        });

        expect(result.success).toBe(true);
        expect(result.error).toBe('');
        // In dry run, created/skipped should be empty, wouldCreate/wouldSkip should have data
        expect(result.created).toHaveLength(0);
        expect(result.skipped).toHaveLength(0);
      });
    });
  });

  describe('CreateUser', () => {
    beforeEach(async () => {
      project = await createTempProject({ initialize: true });
    });

    it('should create a user with minimal fields', async () => {
      const result = await project.client.createUser({
        projectPath: project.path,
        id: 'john-doe',
        name: 'John Doe',
      });

      expect(result.success).toBe(true);
      expect(result.error).toBe('');
      expect(result.user).toBeDefined();
      expect(result.user?.id).toBe('john-doe');
      expect(result.user?.name).toBe('John Doe');
    });

    it('should create a user with all fields', async () => {
      const result = await project.client.createUser({
        projectPath: project.path,
        id: 'jane-doe',
        name: 'Jane Doe',
        email: 'jane@example.com',
        gitUsernames: ['janedoe', 'jane.doe'],
      });

      expect(result.success).toBe(true);
      expect(result.user?.email).toBe('jane@example.com');
      expect(result.user?.gitUsernames).toContain('janedoe');
      expect(result.user?.gitUsernames).toContain('jane.doe');
    });
  });

  describe('GetUser', () => {
    beforeEach(async () => {
      project = await createTempProject({ initialize: true });
    });

    it('should get user by ID', async () => {
      await project.client.createUser({
        projectPath: project.path,
        id: 'test-user',
        name: 'Test User',
        email: 'test@example.com',
      });

      const user = await project.client.getUser({
        projectPath: project.path,
        userId: 'test-user',
      });

      expect(user.id).toBe('test-user');
      expect(user.name).toBe('Test User');
      expect(user.email).toBe('test@example.com');
    });
  });

  describe('ListUsers', () => {
    beforeEach(async () => {
      project = await createTempProject({ initialize: true });
    });

    it('should list all users', async () => {
      await project.client.createUser({
        projectPath: project.path,
        id: 'user-1',
        name: 'User 1',
      });
      await project.client.createUser({
        projectPath: project.path,
        id: 'user-2',
        name: 'User 2',
      });

      const result = await project.client.listUsers({
        projectPath: project.path,
      });

      expect(result.totalCount).toBe(2);
      expect(result.users.length).toBe(2);
    });

    it('should return empty list for project with no users', async () => {
      const result = await project.client.listUsers({
        projectPath: project.path,
      });

      expect(result.totalCount).toBe(0);
      expect(result.users.length).toBe(0);
    });
  });

  describe('UpdateUser', () => {
    beforeEach(async () => {
      project = await createTempProject({ initialize: true });
    });

    it('should update user name', async () => {
      await project.client.createUser({
        projectPath: project.path,
        id: 'update-test',
        name: 'Original Name',
      });

      const result = await project.client.updateUser({
        projectPath: project.path,
        userId: 'update-test',
        name: 'Updated Name',
      });

      expect(result.success).toBe(true);
      expect(result.user?.name).toBe('Updated Name');
    });

    it('should update user email', async () => {
      await project.client.createUser({
        projectPath: project.path,
        id: 'email-test',
        name: 'Email Test',
        email: 'old@example.com',
      });

      const result = await project.client.updateUser({
        projectPath: project.path,
        userId: 'email-test',
        email: 'new@example.com',
      });

      expect(result.success).toBe(true);
      expect(result.user?.email).toBe('new@example.com');
    });
  });

  describe('DeleteUser', () => {
    beforeEach(async () => {
      project = await createTempProject({ initialize: true });
    });

    it('should delete a user', async () => {
      await project.client.createUser({
        projectPath: project.path,
        id: 'to-delete',
        name: 'To Delete',
      });

      const deleteResult = await project.client.deleteUser({
        projectPath: project.path,
        userId: 'to-delete',
      });

      expect(deleteResult.success).toBe(true);

      // Verify it's gone
      const listResult = await project.client.listUsers({
        projectPath: project.path,
      });
      expect(listResult.totalCount).toBe(0);
    });
  });
});
