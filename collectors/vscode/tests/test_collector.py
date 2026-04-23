"""Tests for VSCodeCollector."""

from __future__ import annotations

import json
import sqlite3
import time
from pathlib import Path

import pytest

from collectors.vscode.collector import VSCodeCollector


@pytest.fixture
def vscdb(tmp_path: Path) -> Path:
    db_path = tmp_path / "state.vscdb"
    conn = sqlite3.connect(str(db_path))
    conn.execute("CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT)")
    recent = {
        "entries": [
            {"folderUri": "file:///Users/dev/my-project", "label": "my-project"},
            {"folderUri": "file:///Users/dev/another-repo", "label": "another-repo"},
        ]
    }
    conn.execute(
        "INSERT INTO ItemTable VALUES (?, ?)",
        ("history.recentlyOpenedPathsList", json.dumps(recent)),
    )
    conn.commit()
    conn.close()
    return db_path


@pytest.fixture
def history_dir(tmp_path: Path) -> Path:
    hist = tmp_path / "History"
    hist.mkdir()
    # Create two history entries
    for name, resource in [("abc123", "file:///Users/dev/main.rs"), ("def456", "file:///Users/dev/lib.rs")]:
        d = hist / name
        d.mkdir()
        (d / "entries.json").write_text(json.dumps({"resource": resource, "entries": []}))
    return hist


@pytest.fixture
def collector_with_db(vscdb: Path) -> VSCodeCollector:
    c = VSCodeCollector(config={"since_days": 14})
    c._state_db_paths = lambda: [vscdb]  # type: ignore[method-assign]
    c._history_root_paths = lambda: []  # type: ignore[method-assign]
    return c


@pytest.fixture
def collector_with_history(history_dir: Path) -> VSCodeCollector:
    c = VSCodeCollector(config={"since_days": 14, "include_file_history": True})
    c._state_db_paths = lambda: []  # type: ignore[method-assign]
    c._history_root_paths = lambda: [history_dir]  # type: ignore[method-assign]
    return c


@pytest.mark.asyncio
async def test_collect_workspaces(collector_with_db: VSCodeCollector) -> None:
    items = await collector_with_db.collect()
    uris = [i.url for i in items]
    assert "file:///Users/dev/my-project" in uris
    assert "file:///Users/dev/another-repo" in uris


@pytest.mark.asyncio
async def test_workspace_content_label(collector_with_db: VSCodeCollector) -> None:
    items = await collector_with_db.collect()
    contents = [i.content for i in items]
    assert any("my-project" in c for c in contents)


@pytest.mark.asyncio
async def test_workspace_sensitivity(collector_with_db: VSCodeCollector) -> None:
    items = await collector_with_db.collect()
    assert all(i.sensitivity.value == "work" for i in items)


@pytest.mark.asyncio
async def test_collect_history_files(collector_with_history: VSCodeCollector) -> None:
    items = await collector_with_history.collect()
    filenames = [i.metadata.get("file") for i in items]
    assert "main.rs" in filenames
    assert "lib.rs" in filenames


@pytest.mark.asyncio
async def test_deduplication(collector_with_db: VSCodeCollector) -> None:
    items1 = await collector_with_db.collect()
    items2 = await collector_with_db.collect()
    ids1 = [i.id for i in items1]
    assert len(ids1) == len(set(ids1)), "IDs should be unique"
    assert {i.id for i in items1} == {i.id for i in items2}


@pytest.mark.asyncio
async def test_health_no_vscode() -> None:
    c = VSCodeCollector(config={})
    c._state_db_paths = lambda: []  # type: ignore[method-assign]
    c._history_root_paths = lambda: []  # type: ignore[method-assign]
    health = await c.health_check()
    assert not health.healthy


@pytest.mark.asyncio
async def test_health_with_db(vscdb: Path) -> None:
    c = VSCodeCollector(config={})
    c._state_db_paths = lambda: [vscdb]  # type: ignore[method-assign]
    c._history_root_paths = lambda: []  # type: ignore[method-assign]
    health = await c.health_check()
    assert health.healthy
