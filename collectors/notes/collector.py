"""
Markdown / Notes Collector

Collects context from local Markdown files and Obsidian vaults.
Reads .md files from a configured directory — no network access required.

Compatible with: Obsidian, Bear exports, Logseq, plain Markdown folders.
"""

from __future__ import annotations

import os
import time
from pathlib import Path

from contextgenos import BaseCollector, CollectorHealth, ContextItem
from contextgenos.types import Sensitivity


class NotesCollector(BaseCollector):
    """Collect context from local Markdown files and Obsidian vaults."""

    name = "notes"
    version = "0.1.0"
    platforms = ["macos", "linux", "windows"]

    DEFAULT_VAULT_PATH = "~/Documents"
    MAX_FILE_SIZE_BYTES = 100_000  # Skip files larger than 100KB
    MAX_CONTENT_LENGTH = 2000      # Truncate content to this length

    def _vault_path(self) -> Path:
        raw = self.config.get("vault_path", self.DEFAULT_VAULT_PATH)
        return Path(os.path.expanduser(raw))

    def _modified_since(self) -> float:
        since_days = self.config.get("since_days", 30)
        return time.time() - since_days * 86400

    def _should_exclude(self, path: Path) -> bool:
        excluded = self.config.get("excluded_patterns", [".obsidian", ".trash", "templates"])
        return any(part in str(path) for part in excluded)

    def _read_file(self, path: Path) -> ContextItem | None:
        try:
            if path.stat().st_size > self.MAX_FILE_SIZE_BYTES:
                return None

            text = path.read_text(encoding="utf-8", errors="replace")
            content = text[: self.MAX_CONTENT_LENGTH]
            if len(text) > self.MAX_CONTENT_LENGTH:
                content += "\n[truncated]"

            if not content.strip():
                return None

            return ContextItem(
                content=content,
                source=self.name,
                url=str(path),
                sensitivity=Sensitivity.PERSONAL,
                metadata={
                    "filename": path.name,
                    "relative_path": str(path.relative_to(self._vault_path())),
                    "modified_at": int(path.stat().st_mtime),
                },
            )
        except (OSError, UnicodeDecodeError):
            return None

    async def collect(self) -> list[ContextItem]:
        vault = self._vault_path()
        if not vault.exists():
            return []

        since = self._modified_since()
        items: list[ContextItem] = []
        max_items = self.config.get("max_items", 100)

        for md_file in sorted(vault.rglob("*.md"), key=lambda p: p.stat().st_mtime, reverse=True):
            if len(items) >= max_items:
                break
            if self._should_exclude(md_file):
                continue
            if md_file.stat().st_mtime < since:
                continue
            item = self._read_file(md_file)
            if item:
                items.append(item)

        return items

    async def health_check(self) -> CollectorHealth:
        vault = self._vault_path()
        if vault.is_dir():
            md_count = sum(1 for _ in vault.rglob("*.md"))
            return CollectorHealth(
                healthy=True,
                message=f"Found {md_count} Markdown files in {vault}",
            )
        return CollectorHealth(
            healthy=False,
            message=f"Notes directory not found: {vault}",
        )
