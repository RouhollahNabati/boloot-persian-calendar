#!/usr/bin/env bash
set -euo pipefail

UUID="boloot-calendar@boloot.ir"
EXT_DIR="${HOME}/.local/share/gnome-shell/extensions/${UUID}"
META="${EXT_DIR}/metadata.json"
STALE=0

echo "=== BOLOOT GNOME extension check ==="
echo "GNOME Shell: $(gnome-shell --version 2>/dev/null || echo unknown)"

if pgrep -x gnome-shell >/dev/null 2>&1; then
    SHELL_PID=$(pgrep -x gnome-shell | head -1)
    SHELL_STARTED=$(ps -o lstart= -p "${SHELL_PID}" 2>/dev/null | sed 's/^ *//')
    echo "gnome-shell PID ${SHELL_PID} started: ${SHELL_STARTED}"
fi

echo "Extension path: ${EXT_DIR}"
echo ""

if [[ ! -f "${META}" ]]; then
    echo "ERROR: metadata.json not found. Run: make setup-gnome-extension"
    exit 1
fi

DISK_VERSION=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "${META}")
DISK_SHELL=$(python3 -c 'import json,sys; print(",".join(json.load(open(sys.argv[1]))["shell-version"]))' "${META}")

echo "On disk (metadata.json):"
echo "  version: ${DISK_VERSION}"
echo "  shell-version: [${DISK_SHELL}]"

raw=$(gdbus call --session --dest org.gnome.Shell.Extensions \
    --object-path /org/gnome/Shell/Extensions \
    --method org.gnome.Shell.Extensions.GetExtensionInfo "${UUID}" 2>/dev/null || true)
ver=$(echo "${raw}" | sed -n "s/.*'version': <\([^>]*\)>.*/\1/p" | head -1)
state=$(echo "${raw}" | sed -n "s/.*'state': <\([^>]*\)>.*/\1/p" | head -1)
enabled=$(echo "${raw}" | sed -n "s/.*'enabled': <\([^>]*\)>.*/\1/p" | head -1)

echo ""
echo "D-Bus cache (${UUID}):"
echo "  version: ${ver:-not loaded}"
echo "  state: ${state:-not loaded} (1 = ACTIVE, 4 = OUT_OF_DATE)"
echo "  enabled: ${enabled:-unknown}"

state_int=${state%%.*}
if [[ "${state_int}" == "1" && "${enabled}" == "true" ]]; then
    :
elif [[ -n "${ver}" && "${ver%%.*}" == "${DISK_VERSION}" && "${state_int}" == "1" ]]; then
    :
else
    STALE=1
fi

# GNOME Shell caches extension.js for the whole session; metadata version in
# D-Bus can lag behind metadata.json on disk until shell restart.
if [[ -n "${ver}" && "${ver%%.*}" != "${DISK_VERSION}" ]]; then
    STALE=1
    VERSION_MISMATCH=1
fi

echo ""
echo "version-validation disabled:" \
    "$(gsettings get org.gnome.shell disable-extension-version-validation 2>/dev/null || echo unknown)"
USER_EXTS_DISABLED=$(gsettings get org.gnome.shell disable-user-extensions 2>/dev/null || echo unknown)
echo "user-extensions disabled: ${USER_EXTS_DISABLED}"
if [[ "${USER_EXTS_DISABLED}" == "true" ]]; then
    STALE=1
    USER_EXTS_BLOCKER=1
fi

if [[ -f /tmp/boloot-debug-e0145b.log ]]; then
    echo ""
    echo "Extension debug log (last 3 lines):"
    tail -3 /tmp/boloot-debug-e0145b.log
elif [[ "${STALE}" -eq 0 ]]; then
    echo ""
    echo "Extension debug log: missing (waiting for first _apply)"
else
    echo ""
    echo "Extension debug log: missing (extension enable() not run yet)"
fi

if [[ "${STALE}" -eq 1 ]]; then
    echo ""
    if [[ "${USER_EXTS_BLOCKER:-0}" -eq 1 ]]; then
        echo "BLOCKER: All GNOME user extensions are disabled"
        echo "          (org.gnome.shell disable-user-extensions = true)."
        echo "          Fix: gsettings set org.gnome.shell disable-user-extensions false"
        echo "          Then: bash scripts/enable-gnome-extension.sh"
    elif [[ "${VERSION_MISMATCH:-0}" -eq 1 ]]; then
        echo "BLOCKER: Extension on disk is v${DISK_VERSION} but gnome-shell still runs v${ver%%.*}."
        echo "          JavaScript is cached for the whole session."
        echo "          Fix: Alt+F2 → r → Enter  (or log out and back in)."
    elif [[ -z "${ver}" ]]; then
        echo "BLOCKER: Extension files exist on disk but this gnome-shell session"
        echo "          has not registered them (GetExtensionInfo is empty)."
        if [[ -n "${SHELL_STARTED:-}" ]]; then
            echo "          gnome-shell started: ${SHELL_STARTED}"
        fi
        echo "          Fix: log out completely and log back in (lock screen is not enough)."
    else
        echo "Files on disk are correct; GNOME Shell must rescan after logout."
        echo "Run: make setup-gnome-extension"
        echo "Then: log out and log back in once."
    fi
fi

exit "${STALE}"
