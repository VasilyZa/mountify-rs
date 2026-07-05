#!/system/bin/sh
PATH=/data/adb/ap/bin:/data/adb/ksu/bin:/data/adb/magisk:$PATH
WARNING_STRING="WARNING: this file is part of mountify's autoconfiguration! DO NOT DELETE."

[ -w "/mnt" ] && MNT_FOLDER="/mnt"
[ -w "/mnt/vendor" ] && ! grep -q " /mnt/vendor " "/proc/mounts" && MNT_FOLDER="/mnt/vendor"

[ ! "$KSU" = true ] && [ ! "$APATCH" = true ] && {
	echo "allow kernel fs_type file { read write }" > "$MODPATH/sepolicy.rule"
	echo "allow kernel dev_type file { read write }" >> "$MODPATH/sepolicy.rule"
	echo "allow kernel file_type file { read write }" >> "$MODPATH/sepolicy.rule"
	magiskpolicy --apply "$MODPATH/sepolicy.rule" > /dev/null 2>&1
}

PERSISTENT_DIR="/data/adb/mountify"
[ ! -d "$PERSISTENT_DIR" ] && mkdir -p "$PERSISTENT_DIR"

ARCH=$(uname -m)
case "$ARCH" in
	aarch64|arm64) BIN="$MODPATH/mountify-arm64" ;;
	armv7*|arm*)   BIN="$MODPATH/mountify-arm" ;;
	*)             BIN="$MODPATH/mountify-arm64" ;;
esac
chmod +x "$BIN" 2>/dev/null

$BIN install --modpath "$MODPATH" || abort "[!] mountify install checks failed!"

configs="modules.txt whiteouts.txt config.sh"
for file in $configs; do
	[ ! -f "$PERSISTENT_DIR/$file" ] && cat "$MODPATH/$file" > "$PERSISTENT_DIR/$file" && echo "[+] moving $file"
done

{ [ "$KSU" = true ] && [ ! "$KSU_MAGIC_MOUNT" = true ] && [ "$KSU_VER_CODE" -lt 22098 ]; } ||
{ [ "$APATCH" = true ] && [ ! "$APATCH_BIND_MOUNT" = true ] && [ "$APATCH_VER_CODE" -lt 11170 ]; } && {
	printf "\n\n[!] ERROR: sparse-backed overlayfs detected! abort.\n"; abort
}

SUSFS_BIN="/data/adb/ksu/bin/ksu_susfs"
[ "$KSU" = true ] && [ -f "$SUSFS_BIN" ] && {
	sv="$($SUSFS_BIN show version | head -n1 | sed 's/v//; s/\.//g' 2>/dev/null)"
	[ "$sv" = "1510" ] || [ "$sv" = "1511" ] && { printf "\n\n[!] ERROR: susfs conflict! abort.\n"; abort; }
}

[ -f "$PERSISTENT_DIR/explicit_I_want_symlink" ] && {
	echo "[!] forcing symlink script"
	cat "$MODPATH/symlink/mountify-symlink.sh" > "$MODPATH/post-fs-data.sh"
}

if grep -q "metamodule=true\|metamodule=1" "$MODPATH/module.prop" >/dev/null 2>&1; then
	{ [ "$KSU" = true ] && [ ! "$KSU_MAGIC_MOUNT" = true ] && [ "$KSU_VER_CODE" -ge 22098 ]; } ||
	{ [ "$APATCH" = true ] && [ "$APATCH_VER_CODE" -ge 11170 ]; } && {
		echo "[+] metamodule mode"
		mv "$MODPATH/post-fs-data.sh" "$MODPATH/metamount.sh"
	}
fi

[ "$KSU" = true ] && /data/adb/ksud kernel 2>&1 | grep -q "nuke-ext4-sysfs" >/dev/null 2>&1 &&
	echo "$WARNING_STRING" > "$MODPATH/ksud_has_nuke_ext4"

rm -rf "$MODPATH/symlink"
rm "$MODPATH/modules.txt" "$MODPATH/whiteouts.txt" "$MODPATH/config.sh"
