#!/system/bin/sh
PATH=/data/adb/ap/bin:/data/adb/ksu/bin:/data/adb/magisk:$PATH
MODDIR="/data/adb/modules/mountify-rs"

ARCH=$(uname -m)
case "$ARCH" in
	aarch64|arm64) BIN="$MODDIR/mountify-arm64" ;;
	armv7*|arm*)   BIN="$MODDIR/mountify-arm" ;;
	*)             BIN="$MODDIR/mountify-arm64" ;;
esac

echo "[+] mountify"
echo "[+] extended status"
echo ""
$BIN status

[ -z "$MMRL" ] && [ -z "$KSU_NEXT" ] && { [ "$KSU" = "true" ] || [ "$APATCH" = "true" ]; } && sleep 20
