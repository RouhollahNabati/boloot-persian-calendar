#!/usr/bin/env python3
"""Compile .po catalogs to GNU .mo (stdlib only)."""

from __future__ import annotations

import argparse
import struct
from pathlib import Path


def _unquote_po_string(raw: str) -> str:
    raw = raw.strip()
    if not raw.startswith('"'):
        return raw
    out: list[str] = []
    i = 1
    while i < len(raw):
        ch = raw[i]
        if ch == '"':
            break
        if ch == "\\" and i + 1 < len(raw):
            nxt = raw[i + 1]
            mapping = {"n": "\n", "t": "\t", '"': '"', "\\": "\\"}
            out.append(mapping.get(nxt, nxt))
            i += 2
            continue
        out.append(ch)
        i += 1
    return "".join(out)


def parse_po(text: str) -> dict[str, str]:
    entries: dict[str, str] = {}
    msgid: str | None = None
    msgstr: str | None = None
    field: str | None = None

    def flush() -> None:
        nonlocal msgid, msgstr, field
        if msgid is not None and msgstr is not None:
            entries[msgid] = msgstr
        msgid = msgstr = None
        field = None

    for line in text.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            flush()
            continue
        if stripped.startswith("msgid "):
            flush()
            msgid = _unquote_po_string(stripped[6:])
            field = "msgid"
        elif stripped.startswith("msgstr "):
            msgstr = _unquote_po_string(stripped[7:])
            field = "msgstr"
        elif stripped.startswith('"') and field:
            chunk = _unquote_po_string(stripped)
            if field == "msgid" and msgid is not None:
                msgid += chunk
            elif field == "msgstr" and msgstr is not None:
                msgstr += chunk
        else:
            flush()

    flush()
    return entries


def write_mo(entries: dict[str, str], path: Path) -> None:
    keys = [""] + sorted(k for k in entries if k)
    if "" not in entries:
        entries = {"": entries.get("", ""), **entries}

    originals: list[bytes] = []
    translations: list[bytes] = []
    for key in keys:
        originals.append(key.encode("utf-8"))
        translations.append(entries[key].encode("utf-8"))

    count = len(keys)
    header_size = 28
    table_size = count * 8
    orig_table_offset = header_size
    trans_table_offset = orig_table_offset + table_size
    strings_offset = trans_table_offset + table_size

    orig_table: list[tuple[int, int]] = []
    trans_table: list[tuple[int, int]] = []
    blob = bytearray()
    cursor = strings_offset

    for orig, trans in zip(originals, translations):
        orig_table.append((len(orig), cursor))
        blob.extend(orig)
        blob.append(0)
        cursor += len(orig) + 1
        trans_table.append((len(trans), cursor))
        blob.extend(trans)
        blob.append(0)
        cursor += len(trans) + 1

    out = bytearray()
    out.extend(struct.pack("<I", 0x950412DE))  # magic
    out.extend(struct.pack("<I", 0))  # revision
    out.extend(struct.pack("<I", count))
    out.extend(struct.pack("<I", orig_table_offset))
    out.extend(struct.pack("<I", trans_table_offset))
    out.extend(struct.pack("<I", 0))  # hash size
    out.extend(struct.pack("<I", 0))  # hash offset

    for length, offset in orig_table:
        out.extend(struct.pack("<II", length, offset))
    for length, offset in trans_table:
        out.extend(struct.pack("<II", length, offset))
    out.extend(blob)

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(out)


def compile_tree(po_dir: Path, locale_dir: Path, domain: str) -> int:
    count = 0
    for po in sorted(po_dir.glob("*.po")):
        lang = po.stem
        mo = locale_dir / lang / "LC_MESSAGES" / f"{domain}.mo"
        entries = parse_po(po.read_text(encoding="utf-8"))
        write_mo(entries, mo)
        print(mo)
        count += 1
    return count


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
    )
    args = parser.parse_args()
    root = args.root

    n = 0
    n += compile_tree(root / "settings/po", root / "settings/locale", "boloot-settings")
    n += compile_tree(
        root / "gnome-shell/locale/po",
        root / "gnome-shell/locale",
        "boloot-calendar",
    )
    if n == 0:
        raise SystemExit("no .po files found")
    print(f"compiled {n} catalogs")


if __name__ == "__main__":
    main()
