//! Embedded scaffold templates for new collectors.

pub fn collector_py(name: &str, platforms: &str) -> String {
    let platforms_list = platforms
        .split(',')
        .map(|p| format!("\"{}\"", p.trim()))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"""
{name} Collector

TODO: describe what this collector reads.
"""

from __future__ import annotations

from mycontextport import BaseCollector, CollectorHealth, ContextItem
from mycontextport.types import Sensitivity


class {class_name}Collector(BaseCollector):
    name = "{name}"
    version = "0.1.0"
    platforms = [{platforms}]

    async def collect(self) -> list[ContextItem]:
        # TODO: read from local files or databases here.
        # Do NOT make network requests.
        items: list[ContextItem] = []
        return items

    async def health_check(self) -> CollectorHealth:
        # TODO: check whether the data source is accessible.
        return CollectorHealth(healthy=True, message="{name} collector ready")
"#,
        name = name,
        class_name = to_pascal_case(name),
        platforms = platforms_list,
    )
}

pub fn main_py(name: &str) -> String {
    format!(
        r#"from mycontextport.runner import run_collector
from collectors.{name}.collector import {class_name}Collector

run_collector({class_name}Collector)
"#,
        name = name,
        class_name = to_pascal_case(name),
    )
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
