#!/usr/bin/env bash
set -euo pipefail

UUID="boloot-calendar@boloot.ir"
LEGACY_WRONG="boloot@boloot.ir"

gsettings set org.gnome.shell disable-extension-version-validation true
gsettings set org.gnome.shell disable-user-extensions false

raw=$(gsettings get org.gnome.shell enabled-extensions)
body="${raw#@as }"
exts=()
if [[ "${body}" != "[]" ]]; then
    mapfile -t exts < <(
        python3 -c 'import ast,sys; print("\n".join(ast.literal_eval(sys.argv[1])))' "${body}"
    )
fi

filtered=()
for e in "${exts[@]}"; do
    [[ "${e}" == "${LEGACY_WRONG}" ]] && continue
    filtered+=("${e}")
done

found=0
for e in "${filtered[@]}"; do
    [[ "${e}" == "${UUID}" ]] && found=1
done
[[ "${found}" -eq 0 ]] && filtered+=("${UUID}")

quoted=$(printf "'%s', " "${filtered[@]}")
quoted="${quoted%, }"
gsettings set org.gnome.shell enabled-extensions "[${quoted}]"

gnome-extensions disable "${LEGACY_WRONG}" 2>/dev/null || true
gnome-extensions enable "${UUID}" 2>/dev/null || true
