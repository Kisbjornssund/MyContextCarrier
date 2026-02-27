"""
ContextGenOS Python SDK

The SDK for building context collectors — data source plugins that feed
personal context into the ContextGenOS local store.

Quick start:

    from contextgenos import BaseCollector, ContextItem, CollectorHealth

    class MyCollector(BaseCollector):
        name = "my-tool"
        version = "0.1.0"
        platforms = ["macos", "linux"]

        async def collect(self) -> list[ContextItem]:
            return [ContextItem(content="...", source="my-tool")]

        async def health_check(self) -> CollectorHealth:
            return CollectorHealth(healthy=True, message="Ready")
"""

from contextgenos.collector import BaseCollector
from contextgenos.types import CollectorHealth, ContextItem, Sensitivity

__version__ = "0.1.0"
__all__ = [
    "BaseCollector",
    "ContextItem",
    "CollectorHealth",
    "Sensitivity",
]
