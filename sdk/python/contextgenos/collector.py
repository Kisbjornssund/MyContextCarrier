"""
BaseCollector — the interface every ContextGenOS collector must implement.
"""

from __future__ import annotations

import abc
from typing import AsyncIterator

from contextgenos.types import ContextItem, CollectorHealth


class BaseCollector(abc.ABC):
    """
    Abstract base class for all ContextGenOS context collectors.

    To build a collector:

    1. Subclass BaseCollector
    2. Set class attributes: name, version, platforms
    3. Implement collect() and health_check()
    4. Optionally implement stream() for large data sources

    Example:

        class ObsidianCollector(BaseCollector):
            name = "obsidian"
            version = "0.1.0"
            platforms = ["macos", "linux", "windows"]

            async def collect(self) -> list[ContextItem]:
                vault_path = self.config.get("vault_path", "~/Documents/ObsidianVault")
                items = []
                # read markdown files from vault_path
                return items

            async def health_check(self) -> CollectorHealth:
                import os
                vault_path = self.config.get("vault_path", "~/Documents/ObsidianVault")
                path = os.path.expanduser(vault_path)
                if os.path.isdir(path):
                    return CollectorHealth(healthy=True, message=f"Vault found at {path}")
                return CollectorHealth(healthy=False, message=f"Vault not found at {path}")

    Constraints:

    - Do NOT make network requests inside collect() or health_check().
      Collectors must read from local files, databases, or sockets only.
    - Raise CollectorError for recoverable errors.
    - Yield or return items as soon as they are available; do not buffer large datasets.
    """

    #: Unique machine-readable collector name (lowercase, hyphens allowed).
    name: str

    #: Collector version (semver string).
    version: str

    #: List of supported platforms: "macos", "linux", "windows".
    platforms: list[str]

    def __init__(self, config: dict | None = None) -> None:
        """
        Args:
            config: Collector-specific configuration dictionary, sourced from
                    the user's ~/.contextgenos/config.yaml collector section.
        """
        self.config: dict = config or {}

    @abc.abstractmethod
    async def collect(self) -> list[ContextItem]:
        """
        Collect context items from this data source.

        Returns:
            A list of ContextItem objects. Return an empty list if nothing
            is available, do not raise an exception for empty results.

        Raises:
            CollectorError: If the data source is unavailable or malformed.

        Important:
            This method must NOT make network requests.
        """
        ...

    @abc.abstractmethod
    async def health_check(self) -> CollectorHealth:
        """
        Check whether this collector can run on the current system.

        Returns:
            CollectorHealth with healthy=True if the data source is accessible,
            healthy=False with a descriptive message if it is not.

        Important:
            This method must NOT make network requests.
        """
        ...

    async def stream(self) -> AsyncIterator[ContextItem]:
        """
        Stream context items one at a time (optional, for large data sources).

        The default implementation calls collect() and yields each item.
        Override this method for memory-efficient streaming.
        """
        for item in await self.collect():
            yield item


class CollectorError(Exception):
    """Raised by a collector when the data source is unavailable or malformed."""

    def __init__(self, message: str, collector: str = "") -> None:
        self.collector = collector
        super().__init__(message)
