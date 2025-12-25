---
title: "Agent Manager Architecture"
createdAt: "2025-12-24T23:42:48.329401+00:00"
updatedAt: "2025-12-25T07:39:18.102005+00:00"
---

# Agent Manager Architecture

# Agent Manager Architecture

A TUI window manager for AI code agents (Claude Code, Codex CLI, Aider, etc.) integrated into the centy daemon.

## Overview

The Agent Manager provides:
- **Spawn** TUI-based code agents in managed PTYs
- **Grid View** with live previews of all running agents
- **Zoom** into any agent for full interactive mode
- **Background persistence** - agents survive TUI disconnect (daemon owns PTYs)
- **Cross-platform** support (Unix PTY + Windows ConPTY)
- **Notifications** - desktop alerts when agents finish or need input

## High-Level Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                         Daemon (centy-daemon)                     │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │              Existing Services (Issues, Docs, PRs...)       │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │              NEW: TUI Agent Manager Service                 │  │
│  │                                                             │  │
│  │  Uses cockpit library internally:                          │  │
│  │  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────┐   │  │
│  │  │ Agent: claude   │ │ Agent: claude   │ │ Agent: codex│   │  │
│  │  │ project: /app   │ │ project: /lib   │ │ project: /x │   │  │
│  │  │ PTY (owned)     │ │ PTY (owned)     │ │ PTY (owned) │   │  │
│  │  │ vt100 state     │ │ vt100 state     │ │ vt100 state │   │  │
│  │  └─────────────────┘ └─────────────────┘ └─────────────┘   │  │
│  │                                                             │  │
│  │  Notification Manager (desktop alerts)                      │  │
│  └────────────────────────────────────────────────────────────┘  │
│                              │                                    │
│                     gRPC API (with streaming)                     │
└──────────────────────────────────────────────────────────────────┘
                               │
            ┌──────────────────┼──────────────────┐
            │                  │                  │
       ┌────┴────┐        ┌────┴────┐        ┌────┴────┐
       │  TUI    │        │   CLI   │        │  TUI    │
       │ (panel) │        │ (spawn) │        │ (panel) │
       └─────────┘        └─────────┘        └─────────┘
```

## Component Responsibilities

### Daemon (centy-daemon)

**Owns and manages everything:**
- Spawns agent processes (claude, codex, etc.)
- Owns PTY file descriptors (processes stay alive when TUI disconnects)
- Maintains vt100 screen state per agent
- Sends desktop notifications
- Exposes gRPC API for TUI clients

**Uses cockpit library** for PTY and terminal emulation:
- `portable-pty` for cross-platform PTY
- `vt100` for terminal emulation / screen state

### TUI (cockpit-based panel in centy-tui)

**Stateless view layer:**
- Connects to daemon via gRPC
- Sends commands: spawn, kill, resize, input
- Receives screen updates via streaming
- Renders grid view / zoomed view
- Multiple instances can connect simultaneously

### cockpit (library)

**Reusable terminal multiplexer library:**
- Already has PTY management, vt100, layout, widgets
- Used by daemon internally
- Could be used for standalone TUI apps too

## TUI Views

### Grid View (default)

```
┌─────────────────────────────────────────────────────────────────┐
│  Centy Agent Manager                                   [?]Help  │
├─────────────────────────────────────────────────────────────────┤
│ ┌───────────────────────┐ ┌───────────────────────┐            │
│ │ ● claude: fixing auth │ │ ● claude: adding tests│            │
│ ├───────────────────────┤ ├───────────────────────┤            │
│ │ > Analyzing files...  │ │ > Running pytest...   │            │
│ │   Reading auth.rs     │ │   ✓ 12 tests passed   │            │
│ │   Found the bug in... │ │   Adding test for...  │            │
│ │   ▊                   │ │   ▊                   │            │
│ └───────────────────────┘ └───────────────────────┘            │
│ ┌───────────────────────┐                                       │
│ │ ○ aider: idle         │     ╭─────────────────╮              │
│ ├───────────────────────┤     │   + New Agent   │              │
│ │ Waiting for input...  │     ╰─────────────────╯              │
│ └───────────────────────┘                                       │
├─────────────────────────────────────────────────────────────────┤
│ [n]ew  [Enter]zoom  [k]ill  [Esc]back  [q]uit                  │
└─────────────────────────────────────────────────────────────────┘
```

### Zoomed View (full interactive)

```
┌─────────────────────────────────────────────────────────────────┐
│  claude: fixing auth                              [Esc] → grid  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ╭─ Claude Code ─────────────────────────────────────────────╮  │
│  │                                                           │  │
│  │  I found the authentication bug. The session token...    │  │
│  │                                                           │  │
│  │  Let me fix this by updating src/auth.rs:                 │  │
│  │  ... (full interactive agent TUI) ...                     │  │
│  │                                                           │  │
│  ╰───────────────────────────────────────────────────────────╯  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow

```
User types in TUI          TUI closes/disconnects
       │                       │
       ▼                       ▼
  ┌─────────┐            Daemon keeps agents
  │   TUI   │───────────►running (persistent!)
  └────┬────┘            
       │                 New TUI can reconnect
       │ gRPC            and see current state
       ▼
  ┌─────────┐     PTY     ┌───────────┐
  │ Daemon  │◄───────────►│  claude   │
  │         │             │  (child)  │
  │ vt100   │◄─ output ───┤           │
  │ parser  │             └───────────┘
  └─────────┘
       │
       │ gRPC stream (ScreenDiff)
       ▼
  ┌─────────┐
  │   TUI   │ renders preview/zoomed
  └─────────┘
```

## gRPC API (New RPCs for daemon)

### Service Definition

```protobuf
service CentyDaemon {
  // ... existing RPCs ...
  
  // ============ TUI Agent Manager RPCs ============
  
  // Spawn a new TUI agent (claude, codex, etc.)
  rpc SpawnTuiAgent(SpawnTuiAgentRequest) returns (SpawnTuiAgentResponse);
  
  // List all running TUI agents
  rpc ListTuiAgents(ListTuiAgentsRequest) returns (ListTuiAgentsResponse);
  
  // Kill a TUI agent
  rpc KillTuiAgent(KillTuiAgentRequest) returns (KillTuiAgentResponse);
  
  // Subscribe to screen updates (streaming)
  rpc SubscribeAgentScreen(SubscribeAgentScreenRequest) 
      returns (stream AgentScreenUpdate);
  
  // Send keyboard input to an agent
  rpc SendAgentInput(SendAgentInputRequest) returns (SendAgentInputResponse);
  
  // Resize agent PTY
  rpc ResizeAgent(ResizeAgentRequest) returns (ResizeAgentResponse);
  
  // Get agent info (status, metadata)
  rpc GetTuiAgent(GetTuiAgentRequest) returns (TuiAgent);
}
```

### Message Definitions

```protobuf
// TUI Agent types (different from LLM agent types)
enum TuiAgentType {
  TUI_AGENT_TYPE_UNSPECIFIED = 0;
  TUI_AGENT_TYPE_CLAUDE = 1;      // claude CLI
  TUI_AGENT_TYPE_CODEX = 2;       // codex CLI
  TUI_AGENT_TYPE_AIDER = 3;       // aider
  TUI_AGENT_TYPE_CUSTOM = 4;      // custom command
}

enum TuiAgentStatus {
  TUI_AGENT_STATUS_UNSPECIFIED = 0;
  TUI_AGENT_STATUS_STARTING = 1;
  TUI_AGENT_STATUS_RUNNING = 2;
  TUI_AGENT_STATUS_IDLE = 3;
  TUI_AGENT_STATUS_WAITING_INPUT = 4;
  TUI_AGENT_STATUS_FINISHED = 5;
  TUI_AGENT_STATUS_ERROR = 6;
}

message TuiAgent {
  string id = 1;                    // UUID
  TuiAgentType agent_type = 2;
  string project_path = 3;          // Working directory
  TuiAgentStatus status = 4;
  string started_at = 5;            // ISO timestamp
  uint32 rows = 6;                  // Current PTY rows
  uint32 cols = 7;                  // Current PTY cols
}

message SpawnTuiAgentRequest {
  string project_path = 1;          // Directory to run in
  TuiAgentType agent_type = 2;      // Default: CLAUDE
  repeated string extra_args = 3;   // Additional CLI args
}

message SpawnTuiAgentResponse {
  bool success = 1;
  string error = 2;
  TuiAgent agent = 3;
}

message ListTuiAgentsRequest {
  string project_path = 1;          // Filter by project (empty = all)
}

message ListTuiAgentsResponse {
  repeated TuiAgent agents = 1;
  uint32 total_count = 2;
}

message SubscribeAgentScreenRequest {
  string agent_id = 1;
  bool full_screen = 2;             // Request full screen first, then diffs
}

message AgentScreenUpdate {
  string agent_id = 1;
  oneof update {
    FullScreen full_screen = 2;
    ScreenDiff diff = 3;
  }
  TuiAgentStatus status = 4;        // Include status in each update
}

message FullScreen {
  repeated ScreenRow rows = 1;
  uint32 cursor_row = 2;
  uint32 cursor_col = 3;
}

message ScreenRow {
  repeated ScreenCell cells = 1;
}

message ScreenCell {
  string char = 1;
  uint32 fg_color = 2;              // RGB packed
  uint32 bg_color = 3;              // RGB packed
  bool bold = 4;
  bool italic = 5;
  bool underline = 6;
}

message ScreenDiff {
  repeated CellChange changes = 1;
  optional uint32 cursor_row = 2;
  optional uint32 cursor_col = 3;
}

message CellChange {
  uint32 row = 1;
  uint32 col = 2;
  ScreenCell cell = 3;
}

message SendAgentInputRequest {
  string agent_id = 1;
  bytes data = 2;                   // Raw keyboard input
}

message ResizeAgentRequest {
  string agent_id = 1;
  uint32 rows = 2;
  uint32 cols = 3;
}
```

## Agent Configuration

```yaml
# ~/.config/centy/agents.yaml or in project config.local.json
tui_agents:
  types:
    claude:
      command: "claude"
      args: []
      default: true
    codex:
      command: "codex"
      args: []
    aider:
      command: "aider"
      args: ["--model", "claude-3-5-sonnet"]
    custom:
      command: "/path/to/my-agent"
      args: ["--some-flag"]
```

## Implementation in Daemon

### Module Structure

```
centy-daemon/src/
  tui_agents/                   # NEW
    mod.rs                      # TuiAgentManager
    managed_agent.rs            # ManagedTuiAgent struct
    screen.rs                   # Screen diffing logic
    notifications.rs            # Desktop notification triggers
    grpc.rs                     # gRPC handlers
```

### Using cockpit Library

```rust
// In daemon, use cockpit for PTY management
use cockpit::{PaneManager, SpawnConfig, PaneHandle};

pub struct TuiAgentManager {
    agents: HashMap<Uuid, ManagedTuiAgent>,
    config: TuiAgentConfig,
    notification_tx: Sender<Notification>,
}

struct ManagedTuiAgent {
    id: Uuid,
    agent_type: TuiAgentType,
    project_path: PathBuf,
    
    // cockpit pane handle
    pane: PaneHandle,
    
    // For screen streaming
    last_screen: vt100::Screen,
    subscribers: Vec<Sender<AgentScreenUpdate>>,
    
    status: TuiAgentStatus,
    started_at: Instant,
}

impl TuiAgentManager {
    pub fn spawn(&mut self, project_path: PathBuf, agent_type: TuiAgentType) -> Result<Uuid> {
        let config = self.config.get_agent_config(agent_type)?;
        
        // Use cockpit to spawn PTY
        let mut pane_manager = PaneManager::new();
        let spawn_config = SpawnConfig::new(&config.command)
            .args(&config.args)
            .cwd(&project_path);
        
        let pane = pane_manager.spawn(spawn_config)?;
        
        let id = Uuid::new_v4();
        let agent = ManagedTuiAgent {
            id,
            agent_type,
            project_path,
            pane,
            last_screen: vt100::Screen::new(24, 80),
            subscribers: vec![],
            status: TuiAgentStatus::Starting,
            started_at: Instant::now(),
        };
        
        self.agents.insert(id, agent);
        Ok(id)
    }
    
    pub async fn poll_all(&mut self) {
        for agent in self.agents.values_mut() {
            // Poll pane for updates
            let screen = agent.pane.screen();
            let diff = compute_diff(&agent.last_screen, screen);
            
            if !diff.changes.is_empty() {
                // Send to all subscribers
                for tx in &agent.subscribers {
                    let _ = tx.send(AgentScreenUpdate {
                        agent_id: agent.id.to_string(),
                        update: diff.clone(),
                        status: agent.status,
                    }).await;
                }
                agent.last_screen = screen.clone();
            }
            
            // Check for status changes → notifications
            let new_status = detect_status(screen);
            if new_status != agent.status {
                if new_status == TuiAgentStatus::Finished {
                    self.notification_tx.send(Notification {
                        title: "Agent Finished".to_string(),
                        body: format!("{} completed in {:?}", agent.name(), agent.started_at.elapsed()),
                    }).await;
                }
                agent.status = new_status;
            }
        }
    }
}
```

## CLI Commands

```bash
# Open TUI panel with agents grid
centy agents

# CLI operations (talk to daemon)
centy agents list
centy agents spawn              # spawn claude in current project
centy agents spawn --type codex
centy agents kill <id>
centy agents attach <id>        # zoom directly into one agent
```

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| PTY ownership | Daemon | Agents persist when TUI disconnects |
| Communication | gRPC with streaming | Real-time screen updates, cross-platform |
| Screen updates | Diffs | Minimize bandwidth, faster rendering |
| Notifications | Daemon-managed | Works even when TUI is closed |
| cockpit usage | As library in daemon | Reuse PTY/vt100 logic |
| Agent types | Configurable, Claude default | Extensible but works out of box |
| Spawn location | Project directory | Always spawn in project root |

## Future Considerations

1. **Multi-attach**: Multiple TUIs viewing same agent (one writer, many readers)
2. **Scrollback sync**: How much history to keep and stream on reconnect
3. **Agent templates**: Pre-configured agent setups for common tasks
4. **Resource monitoring**: CPU/memory per agent
5. **Agent groups**: Group agents by project/task
