"""
Core data types for ContextGenOS collectors.
"""

from __future__ import annotations

import time
import uuid
from dataclasses import dataclass, field
from enum import Enum
from typing import Any


class Sensitivity(str, Enum):
    """Sensitivity classification for context items."""

    UNKNOWN = "unknown"
    PUBLIC = "public"
    WORK = "work"
    PERSONAL = "personal"
    HEALTH = "health"
    FINANCIAL = "financial"


@dataclass
class ContextItem:
    """
    A single unit of context produced by a collector.

    Attributes:
        content:      The text content of this context item.
        source:       The collector name that produced this item (e.g. "browser", "obsidian").
        id:           Unique identifier, auto-generated if not provided.
        collected_at: Unix timestamp of when this item was collected.
        url:          Optional URL or file path associated with this item.
        sensitivity:  Sensitivity classification used by the privacy rules engine.
        metadata:     Arbitrary key-value metadata from the collector.
    """

    content: str
    source: str
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    collected_at: int = field(default_factory=lambda: int(time.time()))
    url: str | None = None
    sensitivity: Sensitivity = Sensitivity.UNKNOWN
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "content": self.content,
            "source": self.source,
            "collected_at": self.collected_at,
            "url": self.url,
            "sensitivity": self.sensitivity.value,
            "metadata": self.metadata,
        }


@dataclass
class CollectorHealth:
    """
    Health status returned by a collector's health_check() method.

    Attributes:
        healthy: True if the collector can run on this system.
        message: Human-readable explanation of the health status.
    """

    healthy: bool
    message: str
