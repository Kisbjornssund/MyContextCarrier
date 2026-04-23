"""
Subprocess entrypoint for Python collectors.

Usage in a collector's __main__.py:

    from mycontextport.runner import run_collector
    from .collector import MyCollector

    run_collector(MyCollector)

The runner handles CLI argument parsing, config injection, and output
serialisation so each collector does not need its own boilerplate.

Protocol (consumed by PythonCollector in the Rust core):
  argv  --config '<json>'  → run collect(), print JSON array to stdout
  argv  --health           → run health_check(), print {"healthy":…} to stdout
"""

from __future__ import annotations

import argparse
import asyncio
import json
import sys
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from mycontextport.collector import BaseCollector


def run_collector(collector_class: type[BaseCollector]) -> None:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--config", default="{}")
    parser.add_argument("--health", action="store_true")
    args, _ = parser.parse_known_args()

    try:
        config = json.loads(args.config)
    except json.JSONDecodeError:
        config = {}

    collector = collector_class(config=config)

    if args.health:
        result = asyncio.run(collector.health_check())
        print(json.dumps({"healthy": result.healthy, "message": result.message}))
    else:
        items = asyncio.run(collector.collect())
        print(json.dumps([item.to_dict() for item in items]))
