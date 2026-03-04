"""
Calendar Collector

Collects upcoming and recent calendar events from local iCal (.ics) files.
The local iCal file path is configurable. Google Calendar can sync locally
via the macOS Calendar app, making its events available without cloud access.

Ships in: v0.2
"""

from __future__ import annotations

import os
import platform
from pathlib import Path

from mycontextport import BaseCollector, CollectorHealth, ContextItem
from mycontextport.types import Sensitivity


class CalendarCollector(BaseCollector):
    """
    Collect context from local iCal files.

    Configuration keys:
        ical_paths: list of .ics file paths to read (supports ~ expansion)
        days_ahead: how many days into the future to include (default: 14)
        days_behind: how many days into the past to include (default: 7)
    """

    name = "calendar"
    version = "0.1.0"
    platforms = ["macos", "linux", "windows"]

    # Auto-discovery paths per platform (checked in order, all that exist are used)
    _DISCOVERY_PATHS: dict[str, list[str]] = {
        "Darwin": [
            "~/Library/Calendars",                          # macOS Calendar app (iCloud, Google sync)
        ],
        "Linux": [
            "~/.local/share/evolution/calendar",            # GNOME Evolution
            "~/.local/share/gnome-calendar",                # GNOME Calendar
            "~/.thunderbird",                               # Thunderbird (scanned recursively for .ics)
            "~/.local/share/korganizer",                    # KDE Organizer
        ],
        "Windows": [
            "~/AppData/Local/Packages",                     # Windows Calendar (UWP) — scanned for .ics
            "~/AppData/Roaming/Thunderbird",                # Thunderbird on Windows
        ],
    }

    def _ical_paths(self) -> list[Path]:
        configured = self.config.get("ical_paths", [])
        if configured:
            return [Path(os.path.expanduser(p)) for p in configured]

        system = platform.system()
        candidates = self._DISCOVERY_PATHS.get(system, [])
        found: list[Path] = []
        for raw in candidates:
            base = Path(os.path.expanduser(raw))
            if base.exists():
                found.extend(base.rglob("*.ics"))

        return found

    async def collect(self) -> list[ContextItem]:
        # TODO: implement iCal parsing using the icalendar library
        # For now, return an empty list — stub implementation
        return []

    async def health_check(self) -> CollectorHealth:
        paths = self._ical_paths()
        if paths:
            return CollectorHealth(
                healthy=True,
                message=f"Found {len(paths)} iCal file(s)",
            )
        return CollectorHealth(
            healthy=False,
            message=(
                "No iCal files found. Set 'ical_paths' in config or "
                "sync a calendar app to generate local .ics files."
            ),
        )
