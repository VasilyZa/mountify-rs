#!/system/bin/sh
PATH=/data/adb/ap/bin:/data/adb/ksu/bin:/data/adb/magisk:$PATH
MODDIR="/data/adb/modules/mountify-rs"
MODULE_UPDATES_DIR="/data/adb/modules_update/mountify_whiteouts"

echo "[+] mountify's whiteout generator"

ARCH=$(uname -m)
case "$ARCH" in
	aarch64|arm64) BIN="$MODDIR/mountify-arm64" ;;
	armv7*|arm*)   BIN="$MODDIR/mountify-arm" ;;
	*)             BIN="$MODDIR/mountify-arm64" ;;
esac

exec $BIN whiteout-gen "${1:-/data/adb/mountify/whiteouts.txt}" --output "$MODULE_UPDATES_DIR"
