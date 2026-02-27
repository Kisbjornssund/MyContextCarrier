"""
Browser History Collector

Collects recent browser history from Chrome and Firefox by reading their
local SQLite databases. Does not make network requests.

Supported browsers: Chrome, Firefox
Supported platforms: macOS, Linux
"""

from __future__ import annotations

import os
import platform
import shutil
import sqlite3
import tempfile
import time
from pathlib import Path

from contextgenos import BaseCollector, CollectorHealth, ContextItem
from contextgenos.collector import CollectorError
from contextgenos.types import Sensitivity


class BrowserCollector(BaseCollector):
    """Collect context from local browser history (Chrome and Firefox)."""

    name = "browser"
    version = "0.1.0"
    platforms = ["macos", "linux"]

    # Exclude these domains — sensitive or low-signal
    DEFAULT_EXCLUDED_DOMAINS: list[str] = [
        "localhost",
        "127.0.0.1",
        "accounts.google.com",
        "login.",
        "signin.",
    ]

    def _chrome_db_path(self) -> Path | None:
        """Return the path to Chrome's History SQLite database."""
        system = platform.system()
        if system == "Darwin":
            base = Path.home() / "Library/Application Support/Google/Chrome/Default"
        elif system == "Linux":
            base = Path.home() / ".config/google-chrome/Default"
        else:
            return None
        path = base / "History"
        return path if path.exists() else None

    def _firefox_db_path(self) -> Path | None:
        """Return the path to Firefox's places.sqlite database."""
        system = platform.system()
        if system == "Darwin":
            firefox_dir = Path.home() / "Library/Application Support/Firefox/Profiles"
        elif system == "Linux":
            firefox_dir = Path.home() / ".mozilla/firefox"
        else:
            return None

        if not firefox_dir.exists():
            return None

        for profile in firefox_dir.iterdir():
            places = profile / "places.sqlite"
            if places.exists():
                return places
        return None

    def _is_excluded(self, url: str) -> bool:
        excluded = self.config.get("excluded_domains", self.DEFAULT_EXCLUDED_DOMAINS)
        return any(domain in url for domain in excluded)

    def _collect_from_chrome(self, db_path: Path, since_hours: int) -> list[ContextItem]:
        """Read recent history from Chrome's SQLite database."""
        items: list[ContextItem] = []
        since_ts = int((time.time() - since_hours * 3600) * 1_000_000)

        # Chrome keeps the DB locked while running — copy it first
        with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as tmp:
            tmp_path = tmp.name

        try:
            shutil.copy2(db_path, tmp_path)
            conn = sqlite3.connect(tmp_path)
            cursor = conn.execute(
                """
                SELECT url, title, last_visit_time
                FROM urls
                WHERE last_visit_time > ?
                ORDER BY last_visit_time DESC
                LIMIT ?
                """,
                (since_ts, self.config.get("max_items", 200)),
            )
            for url, title, _ in cursor.fetchall():
                if not self._is_excluded(url):
                    content = f"{title}\n{url}" if title else url
                    items.append(
                        ContextItem(
                            content=content,
                            source=self.name,
                            url=url,
                            sensitivity=Sensitivity.PERSONAL,
                            metadata={"browser": "chrome", "title": title},
                        )
                    )
            conn.close()
        except sqlite3.Error as e:
            raise CollectorError(f"Chrome database error: {e}", collector=self.name) from e
        finally:
            os.unlink(tmp_path)

        return items

    def _collect_from_firefox(self, db_path: Path, since_hours: int) -> list[ContextItem]:
        """Read recent history from Firefox's places.sqlite."""
        items: list[ContextItem] = []
        since_ts = int((time.time() - since_hours * 3600) * 1_000_000)

        with tempfile.NamedTemporaryFile(suffix=".db", delete=False) as tmp:
            tmp_path = tmp.name

        try:
            shutil.copy2(db_path, tmp_path)
            conn = sqlite3.connect(tmp_path)
            cursor = conn.execute(
                """
                SELECT p.url, p.title, h.visit_date
                FROM moz_places p
                JOIN moz_historyvisits h ON p.id = h.place_id
                WHERE h.visit_date > ?
                ORDER BY h.visit_date DESC
                LIMIT ?
                """,
                (since_ts, self.config.get("max_items", 200)),
            )
            for url, title, _ in cursor.fetchall():
                if not self._is_excluded(url):
                    content = f"{title}\n{url}" if title else url
                    items.append(
                        ContextItem(
                            content=content,
                            source=self.name,
                            url=url,
                            sensitivity=Sensitivity.PERSONAL,
                            metadata={"browser": "firefox", "title": title},
                        )
                    )
            conn.close()
        except sqlite3.Error as e:
            raise CollectorError(f"Firefox database error: {e}", collector=self.name) from e
        finally:
            os.unlink(tmp_path)

        return items

    async def collect(self) -> list[ContextItem]:
        since_hours = self.config.get("since_hours", 168)  # 7 days
        items: list[ContextItem] = []

        chrome_path = self._chrome_db_path()
        if chrome_path:
            items.extend(self._collect_from_chrome(chrome_path, since_hours))

        firefox_path = self._firefox_db_path()
        if firefox_path:
            items.extend(self._collect_from_firefox(firefox_path, since_hours))

        return items

    async def health_check(self) -> CollectorHealth:
        chrome = self._chrome_db_path()
        firefox = self._firefox_db_path()

        if chrome or firefox:
            found = []
            if chrome:
                found.append("Chrome")
            if firefox:
                found.append("Firefox")
            return CollectorHealth(
                healthy=True,
                message=f"Found browser databases: {', '.join(found)}",
            )

        return CollectorHealth(
            healthy=False,
            message="No Chrome or Firefox database found on this system.",
        )
