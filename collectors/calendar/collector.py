"""
Calendar Collector

Collects upcoming and recent calendar events from local iCal (.ics) files.
The local iCal file path is configurable. Google Calendar can sync locally
via the macOS Calendar app, making its events available without cloud access.

Ships in: v0.2
"""

from __future__ import annotations

import hashlib
import os
import platform
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

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
        try:
            from icalendar import Calendar  # type: ignore[import]
        except ImportError:
            return []

        days_ahead = int(self.config.get("days_ahead", 14))
        days_behind = int(self.config.get("days_behind", 7))
        now = datetime.now(tz=timezone.utc)
        cutoff_past = now.timestamp() - days_behind * 86400
        cutoff_future = now.timestamp() + days_ahead * 86400

        items: list[ContextItem] = []

        for ics_path in self._ical_paths():
            try:
                raw = ics_path.read_bytes()
            except OSError:
                continue

            try:
                cal = Calendar.from_ical(raw)
            except Exception:
                continue

            for component in cal.walk("VEVENT"):
                item = self._parse_vevent(component, ics_path, cutoff_past, cutoff_future)
                if item is not None:
                    items.append(item)

        return items

    def _parse_vevent(
        self,
        component: Any,
        ics_path: Path,
        cutoff_past: float,
        cutoff_future: float,
    ) -> ContextItem | None:
        dtstart = component.get("DTSTART")
        if dtstart is None:
            return None

        start_dt = dtstart.dt
        # Normalize to UTC timestamp — handle date-only (all-day) and datetime
        if isinstance(start_dt, datetime):
            if start_dt.tzinfo is None:
                start_ts = start_dt.replace(tzinfo=timezone.utc).timestamp()
            else:
                start_ts = start_dt.astimezone(timezone.utc).timestamp()
        else:
            # date object (all-day event)
            start_ts = datetime(
                start_dt.year, start_dt.month, start_dt.day, tzinfo=timezone.utc
            ).timestamp()

        if not (cutoff_past <= start_ts <= cutoff_future):
            return None

        summary = str(component.get("SUMMARY", "")).strip() or "(No title)"
        location = str(component.get("LOCATION", "")).strip()
        uid = str(component.get("UID", ""))

        # Stable deterministic ID: prefer UID, fall back to hash of summary+start
        if uid:
            item_id = f"calendar-{hashlib.sha1(uid.encode()).hexdigest()[:16]}"
        else:
            item_id = f"calendar-{hashlib.sha1(f'{summary}{start_ts}'.encode()).hexdigest()[:16]}"

        content = summary
        if location:
            content = f"{summary} ({location})"

        dtend = component.get("DTEND")
        end_ts: float | None = None
        if dtend is not None:
            end_dt = dtend.dt
            if isinstance(end_dt, datetime):
                end_ts = (
                    end_dt.astimezone(timezone.utc).timestamp()
                    if end_dt.tzinfo
                    else end_dt.replace(tzinfo=timezone.utc).timestamp()
                )
            else:
                end_ts = datetime(
                    end_dt.year, end_dt.month, end_dt.day, tzinfo=timezone.utc
                ).timestamp()

        organizer = str(component.get("ORGANIZER", "")).replace("mailto:", "")
        attendees_raw = component.get("ATTENDEE", [])
        if not isinstance(attendees_raw, list):
            attendees_raw = [attendees_raw]
        attendees = [str(a).replace("mailto:", "") for a in attendees_raw]

        metadata: dict[str, Any] = {
            "uid": uid,
            "start": int(start_ts),
            "summary": summary,
        }
        if end_ts is not None:
            metadata["end"] = int(end_ts)
        if location:
            metadata["location"] = location
        if organizer:
            metadata["organizer"] = organizer
        if attendees:
            metadata["attendees"] = attendees

        return ContextItem(
            id=item_id,
            content=content,
            source=self.name,
            collected_at=int(datetime.now(tz=timezone.utc).timestamp()),
            url=str(ics_path),
            sensitivity=Sensitivity.PERSONAL,
            metadata=metadata,
        )

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
