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
    } = {}
  ): Promise<CLIResult> {
    try {
      const result = await this.client.createItem({
        projectPath: this.cwd,
        itemType: 'issues',
        title,
        body: options.description,
        priority: options.priority,
        status: options.status,
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
        stdout: `Created issue #${result.item.metadata.displayNumber}: ${title}`,
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
      const filterParts: Record<string, unknown> = {};
      if (options.status) filterParts['status'] = { $eq: options.status };
      if (options.priority) filterParts['priority'] = { $eq: options.priority };

      const result = await this.client.listItems({
        projectPath: this.cwd,
        itemType: 'issues',
        filter: Object.keys(filterParts).length > 0 ? JSON.stringify(filterParts) : undefined,
      });

      if (result.items.length === 0) {
        return {
          stdout: 'No issues found.',
          stderr: '',
          exitCode: 0,
          data: result,
        };
      }

      const lines = result.items.map(
        (item) =>
          `#${item.metadata.displayNumber} [${item.metadata.status}] ${item.title}`
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
      const result = await this.client.getItem({
        projectPath: this.cwd,
        itemType: 'issues',
        displayNumber,
      });

      const item = result.item;
      const lines = [
        `Issue #${item.metadata.displayNumber}`,
        `Title: ${item.title}`,
        `Status: ${item.metadata.status}`,
        `Priority: ${item.metadata.priority}`,
        `Created: ${item.metadata.createdAt}`,
        '',
        item.body || '(No description)',
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
      const getResult = await this.client.getItem({
        projectPath: this.cwd,
        itemType: 'issues',
        displayNumber,
      });

      const result = await this.client.updateItem({
        projectPath: this.cwd,
        itemType: 'issues',
        itemId: getResult.item.id,
        title: options.title,
        body: options.description,
        status: options.status,
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
      const getResult = await this.client.getItem({
        projectPath: this.cwd,
        itemType: 'issues',
        displayNumber,
      });

      const result = await this.client.deleteItem({
        projectPath: this.cwd,
        itemType: 'issues',
        itemId: getResult.item.id,
        force: true,
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
   * Execute: centy doc create <title> [--content]
   */
  async docCreate(
    title: string,
    options: {
      content?: string;
    } = {}
  ): Promise<CLIResult> {
    try {
      const result = await this.client.createItem({
        projectPath: this.cwd,
        itemType: 'docs',
        title,
        body: options.content,
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
        stdout: `Created doc: ${result.item.id}`,
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
      const result = await this.client.listItems({
        projectPath: this.cwd,
        itemType: 'docs',
      });

      if (result.items.length === 0) {
        return {
          stdout: 'No documents found.',
          stderr: '',
          exitCode: 0,
          data: result,
        };
      }

      const lines = result.items.map((item) => `${item.id}: ${item.title}`);

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
