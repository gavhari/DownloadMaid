#!/bin/sh
set -e

CONFIG_FILE="/etc/foldermaid/config.toml"
SCHEDULE="${CRON_SCHEDULE:-$(grep -oP "schedule\s*=\s*\"\K[^\"]+" "$CONFIG_FILE" 2>/dev/null || echo "0 * * * *")}"
DOWNLOADS_PATH="${DOWNLOADS_PATH:-$(grep -oP "path\s*=\s*\"\K[^\"]+" "$CONFIG_FILE" 2>/dev/null || echo "/data")}"
DRY_RUN="${DRY_RUN:-false}"

CMD="/usr/local/bin/foldermaid"
if [ "$DRY_RUN" = "true" ] || [ "$DRY_RUN" = "1" ]; then
    CMD="$CMD --dry-run"
fi
CMD="$CMD $DOWNLOADS_PATH"

echo "[foldermaid] Schedule: $SCHEDULE"
echo "[foldermaid] Path: $DOWNLOADS_PATH"
echo "[foldermaid] Dry-run: $DRY_RUN"

echo "$SCHEDULE $CMD" | crontab -

if [ "$RUN_ON_START" = "true" ] || [ "$RUN_ON_START" = "1" ]; then
    echo "[foldermaid] Running once at startup..."
    eval "$CMD"
fi

echo "[foldermaid] Starting crond..."
exec crond -f -d 8
