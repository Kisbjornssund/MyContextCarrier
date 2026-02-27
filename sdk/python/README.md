# ContextGenOS Python SDK

Build context collectors for any data source and plug them into the ContextGenOS daemon.

## Installation

```bash
pip install contextgenos
```

## Quick Start

```python
from contextgenos import BaseCollector, CollectorHealth, ContextItem

class MyCollector(BaseCollector):
    name = "my-source"
    version = "0.1.0"

    async def collect(self) -> list[ContextItem]:
        return [
            ContextItem(
                source=self.name,
                content="Today's standup is at 10:00",
                tags=["meeting"],
            )
        ]

    async def health_check(self) -> CollectorHealth:
        return CollectorHealth(healthy=True, message="OK")
```

## Links

- [Documentation](https://docs.contextgenos.dev)
- [GitHub](https://github.com/Kisbjornssund/ContextGenOS)
