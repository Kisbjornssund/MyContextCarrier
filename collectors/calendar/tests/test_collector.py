"""Tests for CalendarCollector iCal parsing."""

from __future__ import annotations

import textwrap
from pathlib import Path

import pytest

from collectors.calendar.collector import CalendarCollector


FIXTURE_ICS = textwrap.dedent("""\
    BEGIN:VCALENDAR
    VERSION:2.0
    PRODID:-//Test//Test//EN
    BEGIN:VEVENT
    UID:test-event-uid-001@example.com
    SUMMARY:Team Standup
    LOCATION:Conference Room B
    DTSTART:20260423T090000Z
    DTEND:20260423T093000Z
    ORGANIZER:mailto:alice@example.com
    ATTENDEE:mailto:bob@example.com
    ATTENDEE:mailto:carol@example.com
    END:VEVENT
    BEGIN:VEVENT
    UID:allday-event-uid-002@example.com
    SUMMARY:Company Holiday
    DTSTART;VALUE=DATE:20260424
    DTEND;VALUE=DATE:20260425
    END:VEVENT
    BEGIN:VEVENT
    UID:old-event-uid-003@example.com
    SUMMARY:Very Old Meeting
    DTSTART:20200101T090000Z
    DTEND:20200101T100000Z
    END:VEVENT
    END:VCALENDAR
""")


@pytest.fixture
def ics_file(tmp_path: Path) -> Path:
    p = tmp_path / "test.ics"
    p.write_text(FIXTURE_ICS)
    return p


@pytest.fixture
def collector(ics_file: Path) -> CalendarCollector:
    return CalendarCollector(config={"ical_paths": [str(ics_file)], "days_ahead": 30, "days_behind": 7})


@pytest.mark.asyncio
async def test_collect_returns_in_window_events(collector: CalendarCollector) -> None:
    items = await collector.collect()
    summaries = [i.metadata["summary"] for i in items]
    assert "Team Standup" in summaries
    assert "Company Holiday" in summaries
    # Old event is outside the window
    assert "Very Old Meeting" not in summaries


@pytest.mark.asyncio
async def test_collect_item_fields(collector: CalendarCollector) -> None:
    items = await collector.collect()
    standup = next(i for i in items if i.metadata["summary"] == "Team Standup")

    assert standup.id == "calendar-" + __import__("hashlib").sha1(
        b"test-event-uid-001@example.com"
    ).hexdigest()[:16]
    assert standup.content == "Team Standup (Conference Room B)"
    assert standup.source == "calendar"
    assert standup.sensitivity.value == "personal"
    assert standup.metadata["organizer"] == "alice@example.com"
    assert "bob@example.com" in standup.metadata["attendees"]
    assert "end" in standup.metadata


@pytest.mark.asyncio
async def test_collect_allday_event(collector: CalendarCollector) -> None:
    items = await collector.collect()
    holiday = next(i for i in items if i.metadata["summary"] == "Company Holiday")
    assert holiday.content == "Company Holiday"
    assert "end" in holiday.metadata


@pytest.mark.asyncio
async def test_collect_idempotent_ids(collector: CalendarCollector) -> None:
    items1 = await collector.collect()
    items2 = await collector.collect()
    ids1 = {i.id for i in items1}
    ids2 = {i.id for i in items2}
    assert ids1 == ids2


@pytest.mark.asyncio
async def test_health_check_with_valid_path(collector: CalendarCollector) -> None:
    health = await collector.health_check()
    assert health.healthy is True


@pytest.mark.asyncio
async def test_health_check_no_files() -> None:
    c = CalendarCollector(config={"ical_paths": ["/nonexistent/path.ics"]})
    health = await c.health_check()
    assert health.healthy is False


@pytest.mark.asyncio
async def test_collect_missing_icalendar_graceful(
    monkeypatch: pytest.MonkeyPatch, collector: CalendarCollector
) -> None:
    import builtins
    real_import = builtins.__import__

    def mock_import(name: str, *args: object, **kwargs: object) -> object:
        if name == "icalendar":
            raise ImportError("icalendar not installed")
        return real_import(name, *args, **kwargs)

    monkeypatch.setattr(builtins, "__import__", mock_import)
    items = await collector.collect()
    assert items == []
