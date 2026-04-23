"""Tests for EmailCollector."""

from __future__ import annotations

import mailbox
import time
from email.mime.text import MIMEText
from pathlib import Path

import pytest

from collectors.email.collector import EmailCollector


def make_message(subject: str, from_addr: str, body: str = "Hello", days_ago: int = 1) -> mailbox.mboxMessage:
    msg = MIMEText(body)
    msg["Subject"] = subject
    msg["From"] = from_addr
    msg["To"] = "me@example.com"
    msg["Message-ID"] = f"<{subject.replace(' ', '-')}@test>"
    # Set date N days ago
    ts = time.time() - days_ago * 86400
    from email.utils import formatdate
    msg["Date"] = formatdate(ts)
    return mailbox.mboxMessage(msg)


@pytest.fixture
def mbox_file(tmp_path: Path) -> Path:
    path = tmp_path / "Inbox"
    box = mailbox.mbox(str(path))
    box.add(make_message("Team meeting tomorrow", "Alice <alice@company.com>", days_ago=1))
    box.add(make_message("Your invoice #1234", "billing@bank.com", days_ago=2))
    box.add(make_message("Old newsletter", "news@example.com", days_ago=60))
    box.flush()
    box.close()
    return path


@pytest.fixture
def collector(mbox_file: Path) -> EmailCollector:
    return EmailCollector(config={
        "mbox_paths": [str(mbox_file)],
        "since_days": 30,
        "work_domains": ["company.com"],
    })


@pytest.mark.asyncio
async def test_collects_recent_emails(collector: EmailCollector) -> None:
    items = await collector.collect()
    subjects = [i.metadata["subject"] for i in items]
    assert "Team meeting tomorrow" in subjects
    assert "Your invoice #1234" in subjects
    # 60 days old — outside window
    assert "Old newsletter" not in subjects


@pytest.mark.asyncio
async def test_work_sensitivity(collector: EmailCollector) -> None:
    items = await collector.collect()
    meeting = next(i for i in items if "meeting" in i.metadata["subject"])
    assert meeting.sensitivity.value == "work"


@pytest.mark.asyncio
async def test_personal_sensitivity(collector: EmailCollector) -> None:
    items = await collector.collect()
    invoice = next(i for i in items if "invoice" in i.metadata["subject"])
    assert invoice.sensitivity.value == "personal"


@pytest.mark.asyncio
async def test_stable_ids(collector: EmailCollector) -> None:
    items1 = await collector.collect()
    items2 = await collector.collect()
    assert {i.id for i in items1} == {i.id for i in items2}


@pytest.mark.asyncio
async def test_content_format(collector: EmailCollector) -> None:
    items = await collector.collect()
    meeting = next(i for i in items if "meeting" in i.metadata["subject"])
    assert "Alice" in meeting.content
    assert "Team meeting" in meeting.content


@pytest.mark.asyncio
async def test_include_body(mbox_file: Path) -> None:
    c = EmailCollector(config={
        "mbox_paths": [str(mbox_file)],
        "include_body": True,
        "since_days": 30,
    })
    items = await c.collect()
    assert any("Hello" in i.content for i in items)


@pytest.mark.asyncio
async def test_health_with_valid_path(collector: EmailCollector) -> None:
    health = await collector.health_check()
    assert health.healthy


@pytest.mark.asyncio
async def test_health_no_files() -> None:
    c = EmailCollector(config={"mbox_paths": ["/nonexistent/Inbox"]})
    health = await c.health_check()
    assert not health.healthy
