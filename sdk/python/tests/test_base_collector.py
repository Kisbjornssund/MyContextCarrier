"""Tests for the BaseCollector interface and ContextItem types."""

import pytest
from contextgenos import BaseCollector, ContextItem, CollectorHealth, Sensitivity
from contextgenos.collector import CollectorError


class MinimalCollector(BaseCollector):
    """Minimal concrete collector for testing."""

    name = "test"
    version = "0.1.0"
    platforms = ["macos", "linux"]

    async def collect(self) -> list[ContextItem]:
        return [
            ContextItem(
                content="Test context item",
                source=self.name,
                sensitivity=Sensitivity.WORK,
            )
        ]

    async def health_check(self) -> CollectorHealth:
        return CollectorHealth(healthy=True, message="Test collector ready")


class EmptyCollector(BaseCollector):
    """Collector that returns no items — valid behavior."""

    name = "empty"
    version = "0.1.0"
    platforms = ["macos"]

    async def collect(self) -> list[ContextItem]:
        return []

    async def health_check(self) -> CollectorHealth:
        return CollectorHealth(healthy=True, message="Ready")


class UnhealthyCollector(BaseCollector):
    """Collector whose health check fails — data source not found."""

    name = "unhealthy"
    version = "0.1.0"
    platforms = ["macos"]

    async def collect(self) -> list[ContextItem]:
        raise CollectorError("Data source not found", collector=self.name)

    async def health_check(self) -> CollectorHealth:
        return CollectorHealth(healthy=False, message="Data source not found at expected path")


@pytest.mark.asyncio
async def test_collect_returns_items():
    collector = MinimalCollector()
    items = await collector.collect()
    assert len(items) == 1
    assert items[0].content == "Test context item"
    assert items[0].source == "test"


@pytest.mark.asyncio
async def test_health_check_returns_healthy():
    collector = MinimalCollector()
    health = await collector.health_check()
    assert health.healthy is True
    assert isinstance(health.message, str)
    assert len(health.message) > 0


@pytest.mark.asyncio
async def test_empty_collect_is_valid():
    collector = EmptyCollector()
    items = await collector.collect()
    assert items == []


@pytest.mark.asyncio
async def test_unhealthy_collector_reports_correctly():
    collector = UnhealthyCollector()
    health = await collector.health_check()
    assert health.healthy is False


@pytest.mark.asyncio
async def test_stream_yields_same_as_collect():
    collector = MinimalCollector()
    collected = await collector.collect()
    streamed = [item async for item in collector.stream()]
    assert len(streamed) == len(collected)
    assert streamed[0].content == collected[0].content


def test_context_item_has_auto_id():
    item = ContextItem(content="Hello", source="test")
    assert item.id is not None
    assert len(item.id) > 0


def test_context_item_has_auto_timestamp():
    item = ContextItem(content="Hello", source="test")
    assert item.collected_at > 0


def test_context_item_to_dict():
    item = ContextItem(
        content="Hello",
        source="test",
        sensitivity=Sensitivity.PERSONAL,
        url="https://example.com",
    )
    d = item.to_dict()
    assert d["content"] == "Hello"
    assert d["source"] == "test"
    assert d["sensitivity"] == "personal"
    assert d["url"] == "https://example.com"


def test_collector_receives_config():
    collector = MinimalCollector(config={"key": "value"})
    assert collector.config["key"] == "value"


def test_collector_error_carries_name():
    err = CollectorError("Something went wrong", collector="browser")
    assert err.collector == "browser"
    assert "Something went wrong" in str(err)
