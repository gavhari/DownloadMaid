# Docker Deployment for FolderMaid

**Date:** 2026-06-17
**Status:** Approved Design

## Overview

Containerized FolderMaid that runs on a schedule to auto-organize a mounted Downloads folder. Single-image, cron-based daemon using multistage Rust build.

## Architecture

```
 downloadmaid:latest
 ┌───────────────────────────────────────┐
 │  crond (busybox)                      │
 │  │  cron: schedule → /usr/local/bin/  │
 │  │        run-downloadmaid.sh         │
 │  └──────────┬─────────────────────────┘
 │             ▼                          │
 │  ┌──────────────────────────────────┐  │
 │  │  downloadmaid --path /data       │  │
 │  │  (reads /etc/downloadmaid/       │  │
 │  │   config.toml for schedule)      │  │
 │  └──────────┬───────────────────────┘  │
 └─────────────┼──────────────────────────┘
               ▼
      Volume: /data (Downloads)
```

## Build

Multistage `Dockerfile`:

1. **Builder stage** (`rust:alpine`) — compile binary
2. **Runtime stage** (`alpine:latest`) — copy binary + entrypoint + config

## Configuration

### config.toml (new fields)

```toml
path = "/data"
schedule = "0 * * * *"   # default: every hour
dry_run = false
```

### Environment Variables

| Env | Override |
|-----|----------|
| `DOWNLOADS_PATH` | Overrides config `path` |
| `CRON_SCHEDULE` | Overrides config `schedule` |
| `DRY_RUN` | Overrides config `dry_run` |
| `RUN_ON_START` | Run `downloadmaid` immediately on container start (`true`/`false`, default `false`) |

## Container Filesystem

```
/etc/downloadmaid/
  └── config.toml        # bundled config
/usr/local/bin/
  ├── downloadmaid       # Rust binary
  └── entrypoint.sh      # startup script
/data/                    # mounted volume (Downloads)
```

## Entrypoint Behavior

1. Parse env vars, override config values if set
2. Write crontab with `CRON_SCHEDULE` + command
3. Run `downloadmaid` once at startup (optional, controlled by `RUN_ON_START`)
4. Start `crond` in foreground (logs to stdout → docker logs)

## Usage

```bash
# Build
docker build -t downloadmaid .

# Run daemon (organize every hour)
docker run -d \
  -v /home/user/Downloads:/data \
  --name downloadmaid \
  downloadmaid

# Custom schedule (daily at 2am)
docker run -d \
  -v /home/user/Downloads:/data \
  -e CRON_SCHEDULE="0 2 * * *" \
  --name downloadmaid \
  downloadmaid

# One-shot dry-run
docker run --rm \
  -v /home/user/Downloads:/data \
  -e DRY_RUN=true \
  -e RUN_ON_START=true \
  downloadmaid

# View logs
docker logs -f downloadmaid
```

## Files to Create/Modify

### Create
- `Dockerfile` — multistage build
- `.dockerignore` — exclude target/, .git/
- `docker/entrypoint.sh` — startup script
- `docker/config.toml` — default config packaged in image

### Modify
- `config.toml` (project root) — add `schedule` field to existing config
- `src/config.rs` — parse `schedule` field from TOML

## Image Size Target

Under 15 MB compressed (static Rust binary on Alpine).

## Error Handling

- Config parse failure → fallback to defaults, log warning
- Volume not mounted → crond runs, downloadmaid errors, log to stderr, next cycle retries
- Binary panics → crond logs failure, next cycle retries

## Testing

- `docker build` passes
- `docker run --rm downloadmaid` with dry-run flag exits 0
- Config override via env vars works
- Cron schedule change via env var reflected in crond
