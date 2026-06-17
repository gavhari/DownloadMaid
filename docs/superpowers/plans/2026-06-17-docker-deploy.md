# Docker Deployment Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Package FolderMaid as Docker image with cron-based scheduling for auto-organizing mounted Downloads folder.

**Architecture:** Multistage Dockerfile (rust:alpine build → alpine runtime). Entrypoint script reads bundled config + env overrides, generates crontab, starts crond foreground. Binary unchanged in logic — only schedule field added to config struct.

**Tech Stack:** Docker, Alpine Linux, busybox crond, Rust (existing)

---

### Task 1: Add `schedule` field to AppConfig

**Files:**
- Modify: `src/config.rs:7-32`
- Test: (implicit via existing tests)

- [ ] **Step 1: Add `schedule` field to `AppConfig` struct**

```rust
// src/config.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub path: PathBuf,
    pub recursive: bool,
    pub dry_run: bool,
    pub blacklist: Vec<String>,
    pub schedule: String,    // cron expression, e.g. "0 * * * *"
}
```

- [ ] **Step 2: Add default for `schedule`**

```rust
// src/config.rs — before impl Default
fn default_schedule() -> String {
    "0 * * * *".to_string()
}
```

- [ ] **Step 3: Update `Default` impl to include `schedule`**

```rust
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            path: default_path(),
            recursive: default_recursive(),
            dry_run: false,
            blacklist: Vec::new(),
            schedule: default_schedule(),
        }
    }
}
```

- [ ] **Step 4: Update `parse_config_file()` to parse `schedule`**

Add after `blacklist` parsing block:

```rust
    let schedule_val = toml_val
        .get("schedule")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(default_schedule);
```

Add `schedule: schedule_val,` to the `Some(AppConfig { ... })` constructor.

- [ ] **Step 5: Run existing tests to confirm no regression**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add src/config.rs
git commit -m "feat: add schedule field to config for cron support

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2: Create Docker infrastructure

**Files:**
- Create: `Dockerfile`
- Create: `.dockerignore`
- Create: `docker/entrypoint.sh`
- Create: `docker/config.toml`

- [ ] **Step 1: Create `.dockerignore`**

```
target/
.git/
.gitignore
*.md
docs/
tests/
```

- [ ] **Step 2: Create `docker/config.toml`**

```toml
path = "/data"
schedule = "0 * * * *"
dry_run = false
```

- [ ] **Step 3: Create `docker/entrypoint.sh`**

```bash
#!/bin/sh
set -e

CONFIG_FILE="/etc/downloadmaid/config.toml"
SCHEDULE="${CRON_SCHEDULE:-$(grep -oP 'schedule\s*=\s*"\K[^"]+' "$CONFIG_FILE" 2>/dev/null || echo "0 * * * *")}"
DOWNLOADS_PATH="${DOWNLOADS_PATH:-$(grep -oP 'path\s*=\s*"\K[^"]+' "$CONFIG_FILE" 2>/dev/null || echo "/data")}"
DRY_RUN="${DRY_RUN:-false}"

CMD="/usr/local/bin/downloadmaid"
if [ "$DRY_RUN" = "true" ] || [ "$DRY_RUN" = "1" ]; then
    CMD="$CMD --dry-run"
fi
CMD="$CMD $DOWNLOADS_PATH"

# Print config on startup
echo "[downloadmaid] Schedule: $SCHEDULE"
echo "[downloadmaid] Path: $DOWNLOADS_PATH"
echo "[downloadmaid] Dry-run: $DRY_RUN"

# Setup crontab
echo "$SCHEDULE $CMD" | crontab -

# Run once at startup if requested
if [ "$RUN_ON_START" = "true" ] || [ "$RUN_ON_START" = "1" ]; then
    echo "[downloadmaid] Running once at startup..."
    eval "$CMD"
fi

# Start crond in foreground
echo "[downloadmaid] Starting crond..."
exec crond -f -d 8
```

- [ ] **Step 4: Make entrypoint executable**

```bash
chmod +x docker/entrypoint.sh
```

- [ ] **Step 5: Create `Dockerfile`**

```dockerfile
# ---- Builder Stage ----
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/

RUN cargo build --release

# ---- Runtime Stage ----
FROM alpine:latest

RUN apk add --no-cache dcron

COPY --from=builder /app/target/release/downloadmaid /usr/local/bin/downloadmaid
COPY docker/config.toml /etc/downloadmaid/config.toml
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh

RUN chmod +x /usr/local/bin/entrypoint.sh

WORKDIR /data

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
```

- [ ] **Step 6: Build image**

Run: `docker build -t downloadmaid .`
Expected: build succeeds, binary compiled in rust:alpine, runtime image small

- [ ] **Step 7: Quick smoke test (dry-run)**

```bash
docker run --rm \
  -v /tmp:/data \
  -e DRY_RUN=true \
  -e RUN_ON_START=true \
  downloadmaid
```

Expected: "FolderMaid — Organizing /data" printed, no errors, container exits (crond runs in background but dry-run test shows it works)

- [ ] **Step 8: Commit**

```bash
git add Dockerfile .dockerignore docker/ docker/config.toml
git commit -m "feat: add Docker deployment with cron scheduling

Multistage Dockerfile, Alpine runtime, entrypoint with env
override support for CRON_SCHEDULE / DOWNLOADS_PATH / DRY_RUN.

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3: Verify full workflow

**Files:**
- Run: final integration tests

- [ ] **Step 1: Run cargo test**

```bash
cargo test
```
Expected: all pass

- [ ] **Step 2: Daemon mode test (run in bg, check docker logs)**

```bash
mkdir -p /tmp/test-downloads
touch /tmp/test-downloads/test.pdf /tmp/test-downloads/photo.jpg

docker run -d \
  -v /tmp/test-downloads:/data \
  -e CRON_SCHEDULE="*/1 * * * *" \
  --name downloadmaid-test \
  downloadmaid

sleep 70
docker logs downloadmaid-test 2>&1 | tail -10
```

Expected: logs show "Organizing /data", files organized into pdf/, jpg/ folders

- [ ] **Step 3: Clean up test container**

```bash
docker stop downloadmaid-test && docker rm downloadmaid-test
rm -rf /tmp/test-downloads
```

- [ ] **Step 4: Check image size**

```bash
docker images downloadmaid
```
Expected: under 20 MB

- [ ] **Step 5: Final commit (if any fixes needed)**

```bash
git push origin main
```
