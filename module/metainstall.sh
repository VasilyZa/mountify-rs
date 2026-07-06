#!/system/bin/sh
[ "$KSU" = true ] && { export KSU_HAS_METAMODULE="true"; export KSU_METAMODULE="mountify"; }
[ "$APATCH" = true ] && { export APATCH_HAS_METAMODULE="true"; export APATCH_METAMODULE="mountify"; }
export MOUNTIFY="true"
export MOUNTIFY_HAS_HOT_INSTALL="true"

mark_replace() { mkdir -p "$1" 2>/dev/null; setfattr -n trusted.overlay.opaque -v y "$1"; chmod 644 "$1"; }
handle_partition() { true; }

mountify_handle_partition() {
	p="$1"; [ ! -d "$MODPATH/system/$p" ] && return
	[ -L "/system/$p" ] && [ -d "/$p" ] && ln -sf "./system/$p" "$MODPATH/$p"
}

install_module
mountify_handle_partition system_ext
mountify_handle_partition vendor
mountify_handle_partition product
mountify_handle_partition odm

ARCH=$(uname -m)
case "$ARCH" in
	aarch64|arm64) BIN="/data/adb/modules/mountify-rs/mountify-arm64" ;;
	armv7*|arm*)   BIN="/data/adb/modules/mountify-rs/mountify-arm" ;;
	*)             BIN="/data/adb/modules/mountify-rs/mountify-arm64" ;;
esac
chmod +x "$BIN" 2>/dev/null
exec $BIN metainstall --modid "$MODID" --modpath "$MODPATH"
