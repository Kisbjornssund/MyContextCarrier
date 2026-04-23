"""
Email Collector

Collects recent email metadata from local mbox files (Thunderbird, Apple Mail,
Evolution) or from a local IMAP cache directory. Never makes network requests
inside collect() — IMAP syncing is a separate optional step.

Default: metadata only (subject, sender name, date).
Set include_body: true in config to store email bodies.

Ships in: v0.4
"""

from __future__ import annotations

import email
import email.header
import hashlib
import mailbox
import os
import platform
import time
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime
from pathlib import Path
from typing import Any

from mycontextport import BaseCollector, CollectorHealth, ContextItem
from mycontextport.types import Sensitivity


def _decode_header(value: str | None) -> str:
    if not value:
        return ""
    parts = email.header.decode_header(value)
    decoded = []
    for part, charset in parts:
        if isinstance(part, bytes):
            try:
                decoded.append(part.decode(charset or "utf-8", errors="replace"))
            except (LookupError, UnicodeDecodeError):
                decoded.append(part.decode("utf-8", errors="replace"))
        else:
            decoded.append(str(part))
    return " ".join(decoded).strip()


def _sender_name(from_field: str) -> str:
    """Extract display name from 'Name <addr>' or return the address."""
    decoded = _decode_header(from_field)
    if "<" in decoded:
        return decoded[: decoded.index("<")].strip().strip('"') or decoded
    return decoded


def _parse_date(msg: email.message.Message) -> int:
    raw = msg.get("Date", "")
    try:
        dt = parsedate_to_datetime(raw)
        return int(dt.astimezone(timezone.utc).timestamp())
    except Exception:
        return int(time.time())


class EmailCollector(BaseCollector):
    """
    Collect context from local email files.

    Configuration keys:
        mbox_paths:     list of .mbox file or directory paths (supports ~ expansion)
        work_domains:   list of sender domains treated as work email (e.g. ["company.com"])
        include_body:   bool — include email body in content (default: false)
        max_items:      max emails to return (default: 100)
        since_days:     only emails from the last N days (default: 30)
    """

    name = "email"
    version = "0.1.0"
    platforms = ["macos", "linux", "windows"]

    _DISCOVERY_PATHS: dict[str, list[str]] = {
        "Darwin": [
            "~/Library/Mail",                           # Apple Mail
            "~/Library/Thunderbird/Profiles",           # Thunderbird
        ],
        "Linux": [
            "~/.thunderbird",                           # Thunderbird
            "~/.local/share/evolution/mail",            # Evolution
            "~/Mail",                                   # mutt / local mail
            "~/Maildir",                                # Maildir format
        ],
        "Windows": [
            "~/AppData/Roaming/Thunderbird/Profiles",   # Thunderbird
        ],
    }

    def _mbox_paths(self) -> list[Path]:
        configured = self.config.get("mbox_paths", [])
        if configured:
            paths: list[Path] = []
            for p in configured:
                expanded = Path(os.path.expanduser(p))
                if expanded.is_dir():
                    paths.extend(expanded.rglob("*.mbox"))
                    paths.extend(expanded.rglob("Inbox"))
                    paths.extend(expanded.rglob("INBOX"))
                elif expanded.exists():
                    paths.append(expanded)
            return paths

        system = platform.system()
        candidates = self._DISCOVERY_PATHS.get(system, [])
        found: list[Path] = []
        for raw in candidates:
            base = Path(os.path.expanduser(raw))
            if base.exists():
                found.extend(base.rglob("*.mbox"))
                # Thunderbird stores mail without extension in "Inbox" files
                for p in base.rglob("*"):
                    if p.is_file() and p.suffix == "" and p.name in ("Inbox", "Sent", "INBOX"):
                        found.append(p)
        return found[:20]  # cap discovery to avoid scanning huge trees

    def _classify_sensitivity(self, sender_addr: str) -> Sensitivity:
        work_domains = self.config.get("work_domains", [])
        if any(sender_addr.lower().endswith(d.lower()) for d in work_domains if d):
            return Sensitivity.Work
        return Sensitivity.Personal

    async def collect(self) -> list[ContextItem]:
        max_items = int(self.config.get("max_items", 100))
        since_days = int(self.config.get("since_days", 30))
        include_body = bool(self.config.get("include_body", False))
        cutoff_ts = time.time() - since_days * 86400

        items: list[ContextItem] = []

        for mbox_path in self._mbox_paths():
            if len(items) >= max_items:
                break
            try:
                box = mailbox.mbox(str(mbox_path))
            except Exception:
                continue

            try:
                for msg in box:
                    if len(items) >= max_items:
                        break

                    msg_ts = _parse_date(msg)
                    if msg_ts < cutoff_ts:
                        continue

                    subject = _decode_header(msg.get("Subject", "(no subject)"))
                    from_field = msg.get("From", "")
                    sender_name = _sender_name(from_field)
                    sender_addr = ""
                    if "<" in from_field and ">" in from_field:
                        sender_addr = from_field[from_field.index("<") + 1 : from_field.index(">")].strip()
                    else:
                        sender_addr = _decode_header(from_field)

                    msg_id = msg.get("Message-ID", "").strip()
                    if msg_id:
                        item_id = f"email-{hashlib.sha1(msg_id.encode()).hexdigest()[:16]}"
                    else:
                        item_id = f"email-{hashlib.sha1(f'{subject}{msg_ts}'.encode()).hexdigest()[:16]}"

                    sensitivity = self._classify_sensitivity(sender_addr)

                    content = f"{subject} — from {sender_name}"
                    if include_body:
                        body = _extract_body(msg)
                        if body:
                            content = f"{subject}\nFrom: {sender_name}\n\n{body[:1000]}"

                    metadata: dict[str, Any] = {
                        "subject": subject,
                        "sender_name": sender_name,
                        "date": msg_ts,
                        "mbox": str(mbox_path),
                    }
                    if msg.get("To"):
                        metadata["to"] = _decode_header(msg.get("To"))

                    items.append(ContextItem(
                        id=item_id,
                        content=content,
                        source=self.name,
                        collected_at=int(time.time()),
                        url=str(mbox_path),
                        sensitivity=sensitivity,
                        metadata=metadata,
                    ))
            finally:
                box.close()

        # Sort by date descending
        items.sort(key=lambda i: i.metadata.get("date", 0), reverse=True)
        return items[:max_items]

    async def health_check(self) -> CollectorHealth:
        paths = self._mbox_paths()
        if paths:
            return CollectorHealth(
                healthy=True,
                message=f"Found {len(paths)} mbox file(s)",
            )
        return CollectorHealth(
            healthy=False,
            message=(
                "No mbox files found. Set 'mbox_paths' in config or ensure a local "
                "mail client (Thunderbird, Apple Mail, Evolution) is installed."
            ),
        )


def _extract_body(msg: email.message.Message) -> str:
    """Extract plain-text body, preferring text/plain over text/html."""
    if msg.is_multipart():
        for part in msg.walk():
            if part.get_content_type() == "text/plain":
                try:
                    charset = part.get_content_charset() or "utf-8"
                    return part.get_payload(decode=True).decode(charset, errors="replace")
                except Exception:
                    pass
        return ""
    else:
        try:
            charset = msg.get_content_charset() or "utf-8"
            payload = msg.get_payload(decode=True)
            if payload:
                return payload.decode(charset, errors="replace")
        except Exception:
            pass
        return ""
