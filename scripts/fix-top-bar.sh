#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}"

echo "=== BOLOOT — اعمال افزونه نوار بالا ==="
make setup-gnome-extension >/dev/null

if bash scripts/check-gnome-extension.sh; then
    echo ""
    echo "افزونه فعال است. اگر ساعت فارسی دیده نمی‌شود:"
    echo "  gnome-extensions disable boloot-calendar@boloot.ir"
    echo "  gnome-extensions enable boloot-calendar@boloot.ir"
    exit 0
fi

echo ""
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│  برای نمایش ساعت فارسی در نوار بالا، یک‌بار از حساب       │"
echo "│  کاربری خارج شوید و دوباره وارد شوید.                    │"
echo "│                                                             │"
echo "│  قفل کردن صفحه (Lock) کافی نیست.                           │"
echo "│  منو → Power Off / خاموش کردن → Log Out / خروج از حساب    │"
echo "└─────────────────────────────────────────────────────────────┘"
echo ""
echo "بعد از ورود مجدد:"
echo "  bash scripts/check-gnome-extension.sh"
echo "  boloot-calendar-ctl preview"
exit 1
