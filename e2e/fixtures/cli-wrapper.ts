/**
 * CLI Wrapper for E2E Testing
 *
 * Simulates CLI commands by invoking gRPC calls to the daemon.
 * This provides a CLI-like interface for testing without requiring
 * the actual CLI binary.
 *
 * Each method returns a CLIResult that includes:
 * - stdout: simulated CLI output
 * - stderr: error messages
 * - exitCode: 0 for success, non-zero for errors
 */

import { createGrpcClient, promisifyClient, type PromisifiedCentyClient } from './grpc-client.js';

export interface CLIResult {
  stdout: string;
  stderr: string;
  exitCode: number;
  /** Raw response data from gRPC (for assertions) */
  data?: unknown;
}

export interface CLIWrapperOptions {
  /** Working directory for CLI commands */
  cwd: string;
  /** Daemon address */
  daemonAddress?: string;
  /** Enable verbose output */
  verbose?: boolean;
}

/**
 * CLI Wrapper class that simulates centy CLI commands via gRPC.
 */
export class CLIWrapper {
  private client: PromisifiedCentyClient;
  private cwd: string;
  private verbose: boolean;

  constructor(options: CLIWrapperOptions) {
    this.cwd = options.cwd;
    this.verbose = options.verbose ?? false;
    const rawClient = createGrpcClient(options.daemonAddress ?? '127.0.0.1:50051');
    this.client = promisifyClient(rawClient);
  }

  /**
   * Close the gRPC connection.
   */
  close(): void {
    this.client.close();
  }

  /**
   * Execute: centy init [--force]
   */
  async init(options: { force?: boolean } = {}): Promise<CLIResult> {
    try {
      const result = await this.client.init({
        projectPath: this.cwd,
        force: options.force ?? false,
      });

      if (!result.success) {
        return {
          stdout: '',
          stderr: `Error: ${result.error}`,
          exitCode: 1,
          data: result,
        };
      }

      const lines: string[] = [
        `Initialized centy project at ${this.cwd}`,
        '',
        'Created files:',
        ...result.created.map((f) => `  - ${f}`),
      ];

      if (result.restored.length > 0) {
        lines.push('', 'Restored files:', ...result.restored.map((f) => `  - ${f}`));
      }

      return {
        stdout: lines.join('\n'),
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy status
   */
  async status(): Promise<CLIResult> {
    try {
      const result = await this.client.isInitialized({
        projectPath: this.cwd,
      });

      if (!result.initialized) {
        return {
          stdout: 'Not a centy project. Run `centy init` to initialize.',
          stderr: '',
          exitCode: 0,
          data: result,
        };
      }

      return {
        stdout: `Centy project initialized at: ${result.centyPath}`,
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy issue create <title> [--description] [--priority] [--status]
   */
  async issueCreate(
    title: string,
    options: {
      description?: string;
      priority?: number;
      status?: string;
      template?: string;
    } = {}
  ): Promise<CLIResult> {
    try {
      const result = await this.client.createIssue({
        projectPath: this.cwd,
        title,
        description: options.description,
        priority: options.priority,
        status: options.status,
        template: options.template,
      });

      if (!result.success) {
        return {
          stdout: '',
          stderr: `Error: ${result.error}`,
          exitCode: 1,
          data: result,
        };
      }

      return {
        stdout: `Created issue #${result.displayNumber}: ${title}`,
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy issue list [--status] [--priority]
   */
  async issueList(
    options: {
      status?: string;
      priority?: number;
    } = {}
  ): Promise<CLIResult> {
    try {
      const result = await this.client.listIssues({
        projectPath: this.cwd,
        status: options.status,
        priority: options.priority,
      });

      if (result.issues.length === 0) {
        return {
          stdout: 'No issues found.',
          stderr: '',
          exitCode: 0,
          data: result,
        };
      }

      const lines = result.issues.map(
        (issue) =>
          `#${issue.displayNumber} [${issue.metadata.status}] ${issue.title}`
      );

      return {
        stdout: lines.join('\n'),
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy issue show <number>
   */
  async issueShow(displayNumber: number): Promise<CLIResult> {
    try {
      const result = await this.client.getIssueByDisplayNumber({
        projectPath: this.cwd,
        displayNumber,
      });

      const lines = [
        `Issue #${result.displayNumber}`,
        `Title: ${result.title}`,
        `Status: ${result.metadata.status}`,
        `Priority: ${result.metadata.priorityLabel}`,
        `Created: ${result.metadata.createdAt}`,
        '',
        result.description || '(No description)',
      ];

      return {
        stdout: lines.join('\n'),
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy issue update <number> [--title] [--status] [--priority]
   */
  async issueUpdate(
    displayNumber: number,
    options: {
      title?: string;
      description?: string;
      status?: string;
      priority?: number;
    }
  ): Promise<CLIResult> {
    try {
      // First get the issue to get its ID
      const issue = await this.client.getIssueByDisplayNumber({
        projectPath: this.cwd,
        displayNumber,
      });

      const result = await this.client.updateIssue({
        projectPath: this.cwd,
        issueId: issue.id,
        ...options,
      });

      if (!result.success) {
        return {
          stdout: '',
          stderr: `Error: ${result.error}`,
          exitCode: 1,
          data: result,
        };
      }

      return {
        stdout: `Updated issue #${displayNumber}`,
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy issue delete <number>
   */
  async issueDelete(displayNumber: number): Promise<CLIResult> {
    try {
      const issue = await this.client.getIssueByDisplayNumber({
        projectPath: this.cwd,
        displayNumber,
      });

      const result = await this.client.deleteIssue({
        projectPath: this.cwd,
        issueId: issue.id,
      });

      if (!result.success) {
        return {
          stdout: '',
          stderr: `Error: ${result.error}`,
          exitCode: 1,
          data: result,
        };
      }

      return {
        stdout: `Deleted issue #${displayNumber}`,
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy doc create <title> [--content] [--slug]
   */
  async docCreate(
    title: string,
    options: {
      content?: string;
      slug?: string;
      template?: string;
    } = {}
  ): Promise<CLIResult> {
    try {
      const result = await this.client.createDoc({
        projectPath: this.cwd,
        title,
        content: options.content,
        slug: options.slug,
        template: options.template,
      });

      if (!result.success) {
        return {
          stdout: '',
          stderr: `Error: ${result.error}`,
          exitCode: 1,
          data: result,
        };
      }

      return {
        stdout: `Created doc: ${result.slug}`,
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy doc list
   */
  async docList(): Promise<CLIResult> {
    try {
      const result = await this.client.listDocs({
        projectPath: this.cwd,
      });

      if (result.docs.length === 0) {
        return {
          stdout: 'No documents found.',
          stderr: '',
          exitCode: 0,
          data: result,
        };
      }

      const lines = result.docs.map((doc) => `${doc.slug}: ${doc.title}`);

      return {
        stdout: lines.join('\n'),
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy config get
   */
  async configGet(): Promise<CLIResult> {
    try {
      const result = await this.client.getConfig({
        projectPath: this.cwd,
      });

      return {
        stdout: JSON.stringify(result, null, 2),
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy user create <id> <name> [--email] [--git-usernames]
   */
  async userCreate(
    id: string,
    name: string,
    options: {
      email?: string;
      gitUsernames?: string[];
    } = {}
  ): Promise<CLIResult> {
    try {
      const result = await this.client.createUser({
        projectPath: this.cwd,
        id,
        name,
        email: options.email,
        gitUsernames: options.gitUsernames,
      });

      if (!result.success) {
        return {
          stdout: '',
          stderr: `Error: ${result.error}`,
          exitCode: 1,
          data: result,
        };
      }

      return {
        stdout: `Created user: ${name} (${id})`,
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy user list
   */
  async userList(): Promise<CLIResult> {
    try {
      const result = await this.client.listUsers({
        projectPath: this.cwd,
      });

      if (result.users.length === 0) {
        return {
          stdout: 'No users found.',
          stderr: '',
          exitCode: 0,
          data: result,
        };
      }

      const lines = result.users.map(
        (user) => `${user.id}: ${user.name}${user.email ? ` <${user.email}>` : ''}`
      );

      return {
        stdout: lines.join('\n'),
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy pr create <title> [--description] [--source-branch] [--target-branch]
   */
  async prCreate(
    title: string,
    options: {
      description?: string;
      sourceBranch?: string;
      targetBranch?: string;
      reviewers?: string[];
      priority?: number;
    } = {}
  ): Promise<CLIResult> {
    try {
      const result = await this.client.createPr({
        projectPath: this.cwd,
        title,
        description: options.description,
        sourceBranch: options.sourceBranch,
        targetBranch: options.targetBranch,
        reviewers: options.reviewers,
        priority: options.priority,
      });

      if (!result.success) {
        return {
          stdout: '',
          stderr: `Error: ${result.error}`,
          exitCode: 1,
          data: result,
        };
      }

      return {
        stdout: `Created PR #${result.displayNumber}: ${title}`,
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy pr list [--status]
   */
  async prList(
    options: {
      status?: string;
      sourceBranch?: string;
      targetBranch?: string;
    } = {}
  ): Promise<CLIResult> {
    try {
      const result = await this.client.listPrs({
        projectPath: this.cwd,
        status: options.status,
        sourceBranch: options.sourceBranch,
        targetBranch: options.targetBranch,
      });

      if (result.prs.length === 0) {
        return {
          stdout: 'No pull requests found.',
          stderr: '',
          exitCode: 0,
          data: result,
        };
      }

      const lines = result.prs.map(
        (pr) =>
          `#${pr.displayNumber} [${pr.metadata.status}] ${pr.title} (${pr.metadata.sourceBranch} -> ${pr.metadata.targetBranch})`
      );

      return {
        stdout: lines.join('\n'),
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Execute: centy info
   */
  async info(): Promise<CLIResult> {
    try {
      const result = await this.client.getDaemonInfo({});

      const lines = [
        `Centy Daemon`,
        `Version: ${result.version}`,
      ];

      return {
        stdout: lines.join('\n'),
        stderr: '',
        exitCode: 0,
        data: result,
      };
    } catch (error) {
      return this.handleError(error);
    }
  }

  /**
   * Handle gRPC errors and convert to CLI result.
   */
  private handleError(error: unknown): CLIResult {
    const errorMessage =
      error instanceof Error ? error.message : String(error);

    if (this.verbose) {
      console.error('CLI Error:', error);
    }

    return {
      stdout: '',
      stderr: `Error: ${errorMessage}`,
      exitCode: 1,
      data: error,
    };
  }
}

/**
 * Create a CLI wrapper for a project directory.
 */
export function createCLI(options: CLIWrapperOptions): CLIWrapper {
  return new CLIWrapper(options);
}
