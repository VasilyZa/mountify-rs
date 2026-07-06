#!/system/bin/sh
PATH=/data/adb/ap/bin:/data/adb/ksu/bin:/data/adb/magisk:$PATH
MODDIR="/data/adb/modules/mountify-rs"

ARCH=$(uname -m)
case "$ARCH" in
	aarch64|arm64) BIN="$MODDIR/mountify-arm64" ;;
	armv7*|arm*)   BIN="$MODDIR/mountify-arm" ;;
	*)             BIN="$MODDIR/mountify-arm64" ;;
esac

case "$(basename "$0" 2>/dev/null)" in
	metamount.sh) FLAGS="--metamodule" ;;
	*)            FLAGS="" ;;
esac

exec $BIN mount --moddir "$MODDIR" $FLAGS
