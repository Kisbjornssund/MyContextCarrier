"""
Calendar Collector

Collects upcoming and recent calendar events from local iCal (.ics) files.
The local iCal file path is configurable. Google Calendar can sync locally
via the macOS Calendar app, making its events available without cloud access.

Ships in: v0.2
"""

from __future__ import annotations

import os
from pathlib import Path

from contextgenos import BaseCollector, CollectorHealth, ContextItem
from contextgenos.types import Sensitivity


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

    DEFAULT_MACOS_CALENDAR_DIR = "~/Library/Calendars"

    def _ical_paths(self) -> list[Path]:
        configured = self.config.get("ical_paths", [])
        if configured:
            return [Path(os.path.expanduser(p)) for p in configured]

        # Auto-discover on macOS
        cal_dir = Path(os.path.expanduser(self.DEFAULT_MACOS_CALENDAR_DIR))
        if cal_dir.exists():
            return list(cal_dir.rglob("*.ics"))

        return []

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
