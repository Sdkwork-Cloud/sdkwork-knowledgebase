#!/usr/bin/env python3
"""Generate http_route_manifest.rs from manifest.rs RouteManifestEntry tables."""

from __future__ import annotations

import re
import sys
from pathlib import Path

METHOD_MAP = {
    "GET": "Get",
    "POST": "Post",
    "PATCH": "Patch",
    "DELETE": "Delete",
    "PUT": "Put",
}


def parse_routes(manifest_path: Path) -> list[tuple[str, str, str]]:
    text = manifest_path.read_text(encoding="utf-8")
    pattern = (
        r'RouteManifestEntry \{\s*method: "(\w+)",\s*path: "([^"]+)",'
        r'\s*operation_id: "([^"]+)",\s*\}'
    )
    return re.findall(pattern, text)


def generate(
    manifest_path: Path,
    fn_name: str,
    tag: str,
    auth: str = "dual_token",
) -> tuple[str, int]:
    routes = parse_routes(manifest_path)
    lines = [
        "use sdkwork_web_core::{HttpMethod, HttpRoute, HttpRouteManifest};",
        "",
        "const HTTP_ROUTES: &[HttpRoute] = &[",
    ]
    for method, path, op in routes:
        hm = METHOD_MAP[method]
        lines.append(f"    HttpRoute::{auth}(")
        lines.append(f"        HttpMethod::{hm},")
        lines.append(f'        "{path}",')
        lines.append(f'        "{tag}",')
        lines.append(f'        "{op}",')
        lines.append("    ),")
    lines.extend(
        [
            "];",
            "",
            f"pub fn {fn_name}() -> HttpRouteManifest {{",
            "    HttpRouteManifest::new(HTTP_ROUTES)",
            "}",
            "",
        ]
    )
    return "\n".join(lines), len(routes)


def main() -> int:
    if len(sys.argv) != 5:
        print(
            "usage: gen_http_route_manifest.py <manifest.rs> <output.rs> <fn_name> <tag>",
            file=sys.stderr,
        )
        return 1

    manifest_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    fn_name = sys.argv[3]
    tag = sys.argv[4]

    content, count = generate(manifest_path, fn_name, tag)
    output_path.write_text(content, encoding="utf-8")
    print(f"wrote {count} routes to {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
