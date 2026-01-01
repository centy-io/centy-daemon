# Check VS Code availability on each request instead of at startup

Currently, VS Code availability is checked once at daemon startup and cached. This means if a user installs VS Code or adds the 'code' command to PATH after the daemon starts, the UI will still show 'VS Code not found' until the daemon is restarted.

The check should happen on each relevant request (e.g., GetDaemonInfo, CreateWorkspace) so that users don't need to restart the daemon after installing VS Code.

This affects is_vscode_available() in centy-daemon/src/workspace/vscode.rs which is called from the server.
