"""
VSCode Collector

Collects recently opened workspaces and recently edited files from VSCode's
local SQLite state database and file history. Reads only local files — no
network requests.

Supported editors: VSCode, VSCodium, VSCode Insiders
Supported platforms: macOS, Linux, Windows

Ships in: v0.4
"""

from __future__ import annotations

import hashlib
import json
import os
import platform
import shutil
import sqlite3
import tempfile
import time
from pathlib import Path
from typing import Any

from mycontextport import BaseCollector, CollectorHealth, ContextItem
from mycontextport.types import Sensitivity


class VSCodeCollector(BaseCollector):
    """
    Collect context from VSCode recent workspaces and file edit history.

    Configuration keys:
        since_days:   only activity from the last N days (default: 14)
        max_items:    max items to return (default: 50)
        include_file_history: bool — include recently edited file paths (default: true)
    """

    name = "vscode"
    version = "0.1.0"
    platforms = ["macos", "linux", "windows"]

    # State DB locations per platform, per editor variant
    _STATE_DB_PATHS: dict[str, list[str]] = {
        "Darwin": [
            "~/Library/Application Support/Code/User/globalStorage/storage.json",
            "~/Library/Application Support/Code/User/globalStorage/state.vscdb",
            "~/Library/Application Support/VSCodium/User/globalStorage/state.vscdb",
            "~/Library/Application Support/Code - Insiders/User/globalStorage/state.vscdb",
        ],
        "Linux": [
            "~/.config/Code/User/globalStorage/state.vscdb",
            "~/.config/VSCodium/User/globalStorage/state.vscdb",
            "~/.config/Code - Insiders/User/globalStorage/state.vscdb",
        ],
        "Windows": [
            "~/AppData/Roaming/Code/User/globalStorage/state.vscdb",
            "~/AppData/Roaming/VSCodium/User/globalStorage/state.vscdb",
        ],
    }

    _HISTORY_PATHS: dict[str, list[str]] = {
        "Darwin": [
            "~/Library/Application Support/Code/User/History",
            "~/Library/Application Support/VSCodium/User/History",
        ],
        "Linux": [
            "~/.config/Code/User/History",
            "~/.config/VSCodium/User/History",
        ],
        "Windows": [
            "~/AppData/Roaming/Code/User/History",
        ],
    }

    def _state_db_paths(self) -> list[Path]:
        system = platform.system()
        candidates = self._STATE_DB_PATHS.get(system, [])
        return [
            Path(os.path.expanduser(p))
            for p in candidates
            if Path(os.path.expanduser(p)).exists()
        ]

    def _history_root_paths(self) -> list[Path]:
        system = platform.system()
        candidates = self._HISTORY_PATHS.get(system, [])
        return [
            Path(os.path.expanduser(p))
            for p in candidates
            if Path(os.path.expanduser(p)).is_dir()
        ]

    async def collect(self) -> list[ContextItem]:
        since_days = int(self.config.get("since_days", 14))
        max_items = int(self.config.get("max_items", 50))
        include_history = bool(self.config.get("include_file_history", True))
        cutoff_ts = time.time() - since_days * 86400

        items: list[ContextItem] = []

        # Collect recent workspaces from state DB
        for db_path in self._state_db_paths():
            if db_path.suffix == ".vscdb":
                items.extend(self._read_vscdb(db_path, cutoff_ts))
            elif db_path.suffix == ".json":
                items.extend(self._read_storage_json(db_path, cutoff_ts))

        # Collect recently edited files from History
        if include_history:
            for hist_root in self._history_root_paths():
                items.extend(self._read_history(hist_root, cutoff_ts))

        # Deduplicate by id, sort by time
        seen: set[str] = set()
        unique: list[ContextItem] = []
        for item in sorted(items, key=lambda i: i.metadata.get("timestamp", 0), reverse=True):
            if item.id not in seen:
                seen.add(item.id)
                unique.append(item)

        return unique[:max_items]

    def _read_vscdb(self, db_path: Path, cutoff_ts: float) -> list[ContextItem]:
        items: list[ContextItem] = []
        tmp = None
        try:
            # VSCode holds a lock on the live DB; copy before reading
            tmp = tempfile.NamedTemporaryFile(suffix=".vscdb", delete=False)
            shutil.copy2(str(db_path), tmp.name)
            tmp.close()

            conn = sqlite3.connect(tmp.name)
            cursor = conn.cursor()

            # Recent workspaces are stored under the key "history.recentlyOpenedPathsList"
            try:
                cursor.execute(
                    "SELECT value FROM ItemTable WHERE key = ?",
                    ("history.recentlyOpenedPathsList",),
                )
                row = cursor.fetchone()
                if row:
                    data = json.loads(row[0])
                    entries = data.get("entries", [])
                    for entry in entries:
                        workspace_uri = (
                            entry.get("folderUri")
                            or entry.get("workspace", {}).get("configPath")
                            or entry.get("fileUri")
                            or ""
                        )
                        if not workspace_uri:
                            continue
                        label = entry.get("label") or workspace_uri.split("/")[-1]
                        item_id = f"vscode-ws-{hashlib.sha1(workspace_uri.encode()).hexdigest()[:16]}"
                        items.append(ContextItem(
                            id=item_id,
                            content=f"VSCode workspace: {label}",
                            source=self.name,
                            collected_at=int(time.time()),
                            url=workspace_uri,
                            sensitivity=Sensitivity.Work,
                            metadata={"type": "workspace", "uri": workspace_uri, "timestamp": time.time()},
                        ))
            except sqlite3.OperationalError:
                pass

            conn.close()
        except Exception:
            pass
        finally:
            if tmp and os.path.exists(tmp.name):
                os.unlink(tmp.name)
        return items

    def _read_storage_json(self, json_path: Path, cutoff_ts: float) -> list[ContextItem]:
        items: list[ContextItem] = []
        try:
            data = json.loads(json_path.read_text(encoding="utf-8"))
            entries = (
                data.get("lastKnownMenubarData", {})
                or data.get("openedPathsList", {}).get("workspaces3", [])
            )
            if isinstance(entries, list):
                for entry in entries:
                    uri = entry if isinstance(entry, str) else entry.get("configPath", "")
                    if not uri:
                        continue
                    label = uri.split("/")[-1].replace(".code-workspace", "")
                    item_id = f"vscode-ws-{hashlib.sha1(uri.encode()).hexdigest()[:16]}"
                    items.append(ContextItem(
                        id=item_id,
                        content=f"VSCode workspace: {label}",
                        source=self.name,
                        collected_at=int(time.time()),
                        url=uri,
                        sensitivity=Sensitivity.Work,
                        metadata={"type": "workspace", "uri": uri, "timestamp": time.time()},
                    ))
        except Exception:
            pass
        return items

    def _read_history(self, hist_root: Path, cutoff_ts: float) -> list[ContextItem]:
        """Each subdirectory in History/ contains a 'entries.json' with file edit history."""
        items: list[ContextItem] = []
        try:
            for entry_dir in hist_root.iterdir():
                if not entry_dir.is_dir():
                    continue
                entries_file = entry_dir / "entries.json"
                if not entries_file.exists():
                    continue
                try:
                    data = json.loads(entries_file.read_text(encoding="utf-8"))
                    resource = data.get("resource", "")
                    label = resource.split("/")[-1] if resource else ""
                    if not label:
                        continue
                    # Use the mtime of the directory as a proxy for last edit time
                    mtime = entry_dir.stat().st_mtime
                    if mtime < cutoff_ts:
                        continue
                    item_id = f"vscode-hist-{hashlib.sha1(resource.encode()).hexdigest()[:16]}"
                    items.append(ContextItem(
                        id=item_id,
                        content=f"Edited: {label}",
                        source=self.name,
                        collected_at=int(time.time()),
                        url=resource,
                        sensitivity=Sensitivity.Work,
                        metadata={"type": "file_edit", "file": label, "timestamp": mtime},
                    ))
                except Exception:
                    continue
        except Exception:
            pass
        return items

    async def health_check(self) -> CollectorHealth:
        db_paths = self._state_db_paths()
        hist_paths = self._history_root_paths()
        if db_paths or hist_paths:
            parts = []
            if db_paths:
                parts.append(f"{len(db_paths)} state DB(s)")
            if hist_paths:
                parts.append(f"{len(hist_paths)} history dir(s)")
            return CollectorHealth(healthy=True, message=f"Found {', '.join(parts)}")
        return CollectorHealth(
            healthy=False,
            message="VSCode not found. Install VSCode or VSCodium to enable this collector.",
        )
