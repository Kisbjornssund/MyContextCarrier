# MyContextPort Python SDK

Build context collectors for any data source and plug them into the MyContextPort daemon.

## Installation

```bash
pip install mycontextport
```

## Quick Start

```python
from mycontextport import BaseCollector, CollectorHealth, ContextItem

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

- [Documentation](https://docs.mycontextport.dev)
- [GitHub](https://github.com/Kisbjornssund/MyContextPort)
