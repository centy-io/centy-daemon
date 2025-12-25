---
title: "Agent Manager Architecture"
createdAt: "2025-12-24T23:42:48.329401+00:00"
updatedAt: "2025-12-25T07:39:18.102005+00:00"
---

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
│                    Daemon (centy-daemon)                          │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │           NEW: TUI Agent Manager Module                     │  │
│  │                                                             │  │
│  │  Uses directly (NOT cockpit):                              │  │
│  │  • portable-pty (cross-platform PTY)                       │  │
│  │  • vt100 (terminal emulation / screen state)               │  │
│  │                                                             │  │
│  │  Daemon OWNS the PTYs → agents persist on TUI disconnect!  │  │
│  │                                                             │  │
│  │  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────┐   │  │
│  │  │ Agent: claude   │ │ Agent: claude   │ │ Agent: codex│   │  │
│  │  │ project: /app   │ │ project: /lib   │ │ project: /x │   │  │
│  │  │ PTY (owned)     │ │ PTY (owned)     │ │ PTY (owned) │   │  │
│  │  │ vt100 state     │ │ vt100 state     │ │ vt100 state │   │  │
│  │  └─────────────────┘ └─────────────────┘ └─────────────┘   │  │
│  └────────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │           Notification Service                              │  │
│  │  • Desktop notifications (notify-rust)                     │  │
│  │  • Network notifications (ntfy.sh)                         │  │
│  │  • Fan-out to all enabled channels                         │  │
│  └────────────────────────────────────────────────────────────┘  │
│                              │                                    │
│                     gRPC API (with streaming)                     │
└──────────────────────────────────────────────────────────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          │                    │                    │
     ┌────┴────┐          ┌────┴────┐          ┌────┴────┐
     │ TUI Mgr │          │Standalone│          │   CLI   │
     │(cockpit)│          │   TUI    │          │ (spawn) │
     │ panels  │          │(terminal)│          │         │
     └────┬────┘          └────┬─────┘          └─────────┘
          │                    │
          │ Check socket       │ Check socket
          ▼                    ▼
    If in manager:        If standalone:
    → open as panel       → attach to terminal
    via cockpit           directly
```

**Key insight**: cockpit is ONLY for TUI panel layout, NOT for agent PTY management.

## Component Responsibilities

### Daemon (centy-daemon)

**Owns and manages everything:**
- Spawns agent processes (claude, codex, etc.)
- Owns PTY file descriptors (processes stay alive when TUI disconnects)
- Maintains vt100 screen state per agent
- Sends notifications (desktop + network)
- Exposes gRPC API for TUI clients

**Uses directly** (NOT cockpit):
- `portable-pty` for cross-platform PTY
- `vt100` for terminal emulation / screen state

### TUI (centy-tui)

**Stateless view layer:**
- Connects to daemon via gRPC
- Sends commands: spawn, kill, resize, input
- Receives screen updates via streaming
- Renders grid view / zoomed view
- Multiple instances can connect simultaneously

**Context Detection** (via socket check):
- Check if tui-manager socket exists at known path
- If socket exists → running inside manager → open agent as cockpit panel
- If no socket → standalone → attach agent directly to terminal

### cockpit (library)

**Used ONLY for TUI panel layout:**
- Manages cockpit panels in tui-manager
- NOT used for agent PTY management
- The daemon owns PTYs directly

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
    agent.rs                    # ManagedTuiAgent struct
    pty.rs                      # PTY spawning and management
    config.rs                   # Agent type configuration
    screen.rs                   # Screen state and diffing

  notifications/                # NEW
    mod.rs                      # NotificationService public API
    router.rs                   # NotificationRouter (fan-out)
    channel.rs                  # NotificationChannel trait
    channels/
      mod.rs
      desktop.rs                # Desktop notifications (notify-rust)
      ntfy.rs                   # ntfy.sh push notifications
```

### Using portable-pty Directly

```rust
use portable_pty::{native_pty_system, PtyPair, PtySize, CommandBuilder};
use vt100::Parser as Vt100Parser;

pub struct TuiAgentManager {
    agents: HashMap<Uuid, ManagedTuiAgent>,
    config: TuiAgentConfig,
    notification_service: NotificationService,
}

pub struct ManagedTuiAgent {
    id: Uuid,
    agent_type: TuiAgentType,
    project_path: PathBuf,

    // PTY management (daemon owns these directly)
    pty_master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send>,

    // Terminal state
    parser: Arc<RwLock<Vt100Parser>>,

    // Streaming subscribers
    subscribers: Vec<mpsc::Sender<AgentScreenUpdate>>,

    // Metadata
    status: TuiAgentStatus,
    started_at: Instant,
}

impl TuiAgentManager {
    pub fn spawn(&mut self, project_path: PathBuf, agent_type: TuiAgentType) -> Result<Uuid> {
        let config = self.config.get_agent_config(agent_type)?;

        // Spawn PTY directly using portable-pty
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(&config.command);
        cmd.args(&config.args);
        cmd.cwd(&project_path);

        let child = pair.slave.spawn_command(cmd)?;

        let id = Uuid::new_v4();
        let agent = ManagedTuiAgent {
            id,
            agent_type,
            project_path,
            pty_master: pair.master,
            child,
            parser: Arc::new(RwLock::new(Vt100Parser::new(24, 80, 1000))),
            subscribers: vec![],
            status: TuiAgentStatus::Starting,
            started_at: Instant::now(),
        };

        self.agents.insert(id, agent);
        Ok(id)
    }
}
```

### Background Polling Loop

```rust
async fn run_poll_loop(manager: Arc<RwLock<TuiAgentManager>>) {
    let mut interval = tokio::time::interval(Duration::from_millis(16)); // ~60fps

    loop {
        interval.tick().await;
        let mut mgr = manager.write().await;

        for agent in mgr.agents.values_mut() {
            // Read PTY output
            if let Some(data) = agent.poll_pty() {
                agent.parser.write().process(&data);

                // Compute diff and send to subscribers
                let diff = agent.compute_screen_diff();
                for tx in &agent.subscribers {
                    let _ = tx.send(AgentScreenUpdate { diff, .. }).await;
                }
            }

            // Check for status changes → notifications
            if agent.status_changed() && agent.status == Finished {
                mgr.notification_service.agent_finished(
                    &agent.name(),
                    &agent.project_path,
                    agent.started_at.elapsed()
                ).await;
            }
        }
    }
}
```

## Notification Service

An extensible, multi-channel notification service that supports both desktop and network notifications.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Notification Service                          │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                  NotificationRouter                        │  │
│  │  • Receives notifications via mpsc channel                │  │
│  │  • Sends to ALL enabled channels (fan-out)                │  │
│  │  • Logs failures, doesn't block on slow channels          │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│       ┌──────────────────────┼──────────────────────┐           │
│       │                      │                      │           │
│  ┌────┴────┐           ┌────┴────┐           ┌────┴────┐       │
│  │ Desktop │           │ ntfy.sh │           │ Future: │       │
│  │ Channel │           │ Channel │           │ Webhook │       │
│  │notify-rs│           │  HTTP   │           │ Gotify  │       │
│  └─────────┘           └─────────┘           └─────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

### NotificationChannel Trait

```rust
// src/notifications/channel.rs
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub category: NotificationCategory,
    pub priority: NotificationPriority,
    pub icon: Option<String>,
    pub actions: Vec<NotificationAction>,
}

#[derive(Clone, Debug)]
pub enum NotificationCategory {
    AgentFinished,
    AgentNeedsInput,
    AgentError,
    SystemInfo,
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Channel identifier (e.g., "desktop", "ntfy")
    fn name(&self) -> &str;

    /// Check if channel is configured and available
    fn is_enabled(&self) -> bool;

    /// Send notification (non-blocking, fire-and-forget)
    async fn send(&self, notification: &Notification) -> Result<(), ChannelError>;
}
```

### Desktop Channel

```rust
// src/notifications/channels/desktop.rs
use notify_rust::Notification as DesktopNotif;

pub struct DesktopChannel {
    enabled: bool,
}

#[async_trait]
impl NotificationChannel for DesktopChannel {
    fn name(&self) -> &str { "desktop" }
    fn is_enabled(&self) -> bool { self.enabled }

    async fn send(&self, notif: &Notification) -> Result<(), ChannelError> {
        DesktopNotif::new()
            .summary(&notif.title)
            .body(&notif.body)
            .icon(notif.icon.as_deref().unwrap_or("terminal"))
            .urgency(match notif.priority {
                NotificationPriority::Low => notify_rust::Urgency::Low,
                NotificationPriority::Normal => notify_rust::Urgency::Normal,
                NotificationPriority::High | NotificationPriority::Urgent =>
                    notify_rust::Urgency::Critical,
            })
            .show()?;
        Ok(())
    }
}
```

### ntfy.sh Channel

```rust
// src/notifications/channels/ntfy.rs
use reqwest::Client;

pub struct NtfyChannel {
    client: Client,
    server_url: String,  // e.g., "https://ntfy.sh" or self-hosted
    topic: String,       // e.g., "centy-notifications"
    enabled: bool,
}

#[async_trait]
impl NotificationChannel for NtfyChannel {
    fn name(&self) -> &str { "ntfy" }
    fn is_enabled(&self) -> bool { self.enabled }

    async fn send(&self, notif: &Notification) -> Result<(), ChannelError> {
        let url = format!("{}/{}", self.server_url, self.topic);

        self.client.post(&url)
            .header("Title", &notif.title)
            .header("Priority", match notif.priority {
                NotificationPriority::Low => "2",
                NotificationPriority::Normal => "3",
                NotificationPriority::High => "4",
                NotificationPriority::Urgent => "5",
            })
            .header("Tags", match notif.category {
                NotificationCategory::AgentFinished => "white_check_mark",
                NotificationCategory::AgentNeedsInput => "bell",
                NotificationCategory::AgentError => "x",
                NotificationCategory::SystemInfo => "information_source",
            })
            .body(notif.body.clone())
            .send()
            .await?;
        Ok(())
    }
}
```

### NotificationRouter (Fan-out)

```rust
// src/notifications/router.rs
pub struct NotificationRouter {
    channels: Vec<Box<dyn NotificationChannel>>,
    rx: mpsc::Receiver<Notification>,
}

impl NotificationRouter {
    pub async fn run(mut self) {
        while let Some(notif) = self.rx.recv().await {
            // Fan-out: send to all enabled channels concurrently
            let futures: Vec<_> = self.channels
                .iter()
                .filter(|ch| ch.is_enabled())
                .map(|ch| {
                    let notif = notif.clone();
                    async move {
                        if let Err(e) = ch.send(&notif).await {
                            tracing::warn!("Notification channel {} failed: {}", ch.name(), e);
                        }
                    }
                })
                .collect();

            // Send to all channels concurrently, don't wait
            tokio::spawn(async move {
                futures::future::join_all(futures).await;
            });
        }
    }
}
```

### NotificationService Public API

```rust
// src/notifications/mod.rs
pub struct NotificationService {
    tx: mpsc::Sender<Notification>,
}

impl NotificationService {
    pub fn new(config: &NotificationConfig) -> (Self, NotificationRouter) {
        let (tx, rx) = mpsc::channel(256);

        let mut channels: Vec<Box<dyn NotificationChannel>> = vec![];

        // Always add desktop if on supported platform
        if config.desktop.enabled {
            channels.push(Box::new(DesktopChannel::new()));
        }

        // Add ntfy if configured
        if let Some(ntfy) = &config.ntfy {
            channels.push(Box::new(NtfyChannel::new(
                &ntfy.server_url,
                &ntfy.topic,
            )));
        }

        let router = NotificationRouter { channels, rx };
        (Self { tx }, router)
    }

    pub async fn notify(&self, notification: Notification) {
        let _ = self.tx.send(notification).await;
    }

    // Convenience methods
    pub async fn agent_finished(&self, agent_name: &str, project: &str, duration: Duration) {
        self.notify(Notification {
            title: "Agent Finished".to_string(),
            body: format!("{} completed in {:?} ({})", agent_name, duration, project),
            category: NotificationCategory::AgentFinished,
            priority: NotificationPriority::Normal,
            icon: Some("checkmark".to_string()),
            actions: vec![],
        }).await;
    }

    pub async fn agent_needs_input(&self, agent_name: &str, project: &str) {
        self.notify(Notification {
            title: "Agent Needs Input".to_string(),
            body: format!("{} is waiting for input ({})", agent_name, project),
            category: NotificationCategory::AgentNeedsInput,
            priority: NotificationPriority::High,
            icon: Some("bell".to_string()),
            actions: vec![],
        }).await;
    }
}
```

### Notification Configuration

```rust
// Part of daemon config or separate notifications.json
#[derive(Deserialize, Clone)]
pub struct NotificationConfig {
    pub desktop: DesktopConfig,
    pub ntfy: Option<NtfyConfig>,
}

#[derive(Deserialize, Clone)]
pub struct DesktopConfig {
    pub enabled: bool,
}

#[derive(Deserialize, Clone)]
pub struct NtfyConfig {
    pub server_url: String,  // "https://ntfy.sh" or self-hosted
    pub topic: String,
}
```

### Usage in Daemon

```rust
// In main.rs or service initialization
let (notification_service, router) = NotificationService::new(&config.notifications);

// Spawn router background task
tokio::spawn(router.run());

// In agent manager, use notification_service
if agent.status == Finished {
    notification_service.agent_finished(
        &agent.name,
        &agent.project_path,
        agent.duration()
    ).await;
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
| PTY ownership | Daemon directly (portable-pty) | Agents persist when TUI disconnects |
| PTY library | portable-pty + vt100 | Cross-platform (Unix PTY + Windows ConPTY), battle-tested |
| Communication | gRPC with streaming | Real-time screen updates, cross-platform |
| Screen updates | Diffs | Minimize bandwidth, faster rendering |
| Notifications | Multi-channel service | Extensible, works even when TUI is closed |
| Notification dispatch | Fan-out to all enabled | User gets notified via preferred channels |
| Network notifications | ntfy.sh | Self-hostable, simple HTTP API, mobile apps |
| cockpit usage | TUI panel layout only | NOT for agent PTY management |
| TUI context detection | Socket check | Multiple managers can run on same machine |
| Agent types | Configurable, Claude default | Extensible but works out of box |
| Spawn location | Project directory | Always spawn in project root |

## Dependencies

```toml
# Daemon Cargo.toml additions
portable-pty = "0.8"           # Cross-platform PTY
vt100 = "0.15"                 # Terminal emulation
tokio-stream = "0.1"           # For gRPC streaming
notify-rust = "4"              # Desktop notifications
reqwest = { version = "0.12", features = ["json"] }  # HTTP for ntfy.sh
async-trait = "0.1"            # Async trait support
```

## Future Considerations

1. **Multi-attach**: Multiple TUIs viewing same agent (one writer, many readers)
2. **Scrollback sync**: How much history to keep and stream on reconnect
3. **Agent templates**: Pre-configured agent setups for common tasks
4. **Resource monitoring**: CPU/memory per agent
5. **Agent groups**: Group agents by project/task
6. **Additional notification channels**: Webhook, Gotify, Slack, Discord
7. **Notification preferences per agent**: Different channels for different agent types
