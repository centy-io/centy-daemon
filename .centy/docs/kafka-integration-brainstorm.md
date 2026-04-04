---
# This file is managed by Centy. Use the Centy CLI to modify it.
createdAt: 2026-04-04T21:40:14.243894+00:00
updatedAt: 2026-04-04T21:40:14.243894+00:00
---

# Kafka Integration Brainstorm

## Goal

Add an optional Kafka sink so that centy-daemon can fire events to a Kafka broker on item mutations (create, update, delete, etc.). This is an opt-in integration — projects that don't configure Kafka are unaffected.

---

## Where to Hook In

The natural injection point is **alongside the existing post-hooks system**. After \`maybe_run_post_hooks()\` in each handler, a Kafka producer would fire as a peer-level sink — not a subprocess, but a direct async call.

The producer should be a **shared singleton** initialized at daemon startup (from global config) so there's no per-request connection overhead.

---

## Config Shape

### Global (\`~/.centy/config.yaml\`) — broker connection config

\`\`\`yaml
kafka:
  brokers:
    - localhost:9092
  auth:
    mechanism: plain  # sasl_ssl | none
    username: ...
    password: ...
  default_topic: centy.events
  topic_routing:
    issues: centy.issues
    docs: centy.docs
\`\`\`

### Per-project (\`.centy/config.yaml\`) — opt-in and topic override

\`\`\`yaml
kafka:
  enabled: true
  topic: centy.my-project.events
  events:
    - item.created
    - item.updated
    - item.deleted
\`\`\`

---

## Event Schema

### Option A — CloudEvents (industry standard, good for polyglot consumers)

\`\`\`json
{
  "specversion": "1.0",
  "type": "io.centy.item.created",
  "source": "/projects/my-project",
  "id": "item-abc123",
  "time": "2026-04-05T10:00:00Z",
  "datacontenttype": "application/json",
  "data": { "...item fields..." }
}
\`\`\`

### Option B — Centy-native flat schema (simpler)

\`\`\`json
{
  "event": "item.created",
  "project": "/path/to/project",
  "item_type": "issues",
  "item_id": "abc123",
  "timestamp": "2026-04-05T10:00:00Z",
  "data": { "...item fields..." },
  "diff": { "...changed fields only, on updates..." }
}
\`\`\`

CloudEvents is worth adopting if downstream consumers will be external/polyglot. The \`diff\` field on updates is very useful for consumers but adds serialization complexity.

---

## Topic Structure Options

| Model | Example | Tradeoff |
|-------|---------|----------|
| Single topic per project | \`centy.my-project\` | Simple, consumers filter by \`event\` field |
| Event-type topics | \`centy.items.created\` | Easy consumer filtering, more partitions |
| Hybrid (per-project, per-domain) | \`centy.my-project.items\` | Project-scoped, consolidated |

Leaning toward **per-project topics** with event type as a message header/field.

---

## Rust Client

**\`rdkafka\`** (\`rust-rdkafka\` crate) — wraps librdkafka, battle-tested, has \`FutureProducer\` for tokio async.

Alternative: **\`rskafka\`** — pure Rust, no C dependency, simpler but less mature.

Given tokio is already in use, \`rdkafka\` with \`FutureProducer\` is the safe production pick.

---

## Delivery Semantics

| Mode | Description | When to use |
|------|-------------|-------------|
| Fire-and-forget | Spawn task, log failures, never block gRPC response | Good starting point |
| In-memory buffer | Channel between handler and producer, absorbs bursts | If throughput is a concern |
| Persistent queue (WAL) | Write to local file before sending, retry on failure | If guaranteed delivery is required |

Recommended starting point: fire-and-forget with structured error logging. The file store is always authoritative.

---

## Open Design Questions

1. **Global vs per-project config** — or both with per-project overrides?
2. **CloudEvents envelope or Centy-native schema?**
3. **Include \`diff\` on updates?** (very useful downstream, adds serialization work)
4. **Failure mode** — is fire-and-forget acceptable, or do we need a local buffer?
5. **Should Kafka be a "hook type"** (unified with existing hooks system) or a separate sink?
6. **Authentication** — plain, SASL/SSL, mTLS — what auth mechanisms to support at launch?
7. **Producer lifecycle** — singleton at startup vs lazy-initialized on first use?
