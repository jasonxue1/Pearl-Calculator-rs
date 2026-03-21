#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable
from urllib.parse import quote
from urllib.request import Request, urlopen


@dataclass(frozen=True)
class Platform:
    os_name: str
    arch: str
    target: str


PLATFORMS: list[Platform] = [
    Platform("Windows", "x86_64", "x86_64-pc-windows-msvc"),
    Platform("Windows", "aarch64", "aarch64-pc-windows-msvc"),
    Platform("Linux", "x86_64", "x86_64-unknown-linux-gnu"),
    Platform("Linux", "aarch64", "aarch64-unknown-linux-gnu"),
    Platform("macOS", "x86_64", "x86_64-apple-darwin"),
    Platform("macOS", "aarch64", "aarch64-apple-darwin"),
]


def render_table(rows: Iterable[dict[str, str]]) -> str:
    lines = [
        "| OS | Arch | CLI | GUI |",
        "| --- | --- | --- | --- |",
    ]
    for row in rows:
        lines.append(f"| {row['os']} | {row['arch']} | {row['cli']} | {row['gui']} |")
    return "\n".join(lines) + "\n"


def empty_rows() -> list[dict[str, str]]:
    return [
        {"os": p.os_name, "arch": p.arch, "target": p.target, "cli": "-", "gui": "-"}
        for p in PLATFORMS
    ]


def find_row(rows: list[dict[str, str]], target: str) -> dict[str, str] | None:
    for row in rows:
        if row["target"] == target:
            return row
    return None


def release_cli_filename(target: str, tag_name: str) -> str:
    ext = "zip" if "windows" in target else "tar.gz"
    return f"pearl-calculator-{target}-{tag_name}.{ext}"


def join_url(base: str, file_name: str) -> str:
    return f"{base.rstrip('/')}/{quote(file_name)}"


GUI_VERSION_PATTERNS = [
    re.compile(r"^pearl-calculator-gui_(\d+\.\d+\.\d+)_x64-setup\.exe$"),
    re.compile(r"^pearl-calculator-gui_(\d+\.\d+\.\d+)_arm64-setup\.exe$"),
    re.compile(r"^pearl-calculator-gui_(\d+\.\d+\.\d+)_amd64\.deb$"),
    re.compile(r"^pearl-calculator-gui_(\d+\.\d+\.\d+)_arm64\.deb$"),
    re.compile(r"^pearl-calculator-gui_(\d+\.\d+\.\d+)_x86_64\.AppImage$"),
    re.compile(r"^pearl-calculator-gui_(\d+\.\d+\.\d+)_aarch64\.AppImage$"),
    re.compile(r"^pearl-calculator-gui_(\d+\.\d+\.\d+)_x86_64\.tar\.gz$"),
    re.compile(r"^pearl-calculator-gui_(\d+\.\d+\.\d+)_aarch64\.tar\.gz$"),
    re.compile(r"^Pearl Calculator_(\d+\.\d+\.\d+)_x64\.dmg$"),
    re.compile(r"^Pearl Calculator_(\d+\.\d+\.\d+)_aarch64\.dmg$"),
]


def detect_gui_version(dist_root: Path) -> str:
    versions: set[str] = set()
    for platform in PLATFORMS:
        artifact_dir = dist_root / f"pearl-calculator-gui-{platform.target}"
        if not artifact_dir.exists():
            continue
        for file_path in artifact_dir.iterdir():
            if not file_path.is_file():
                continue
            name = file_path.name
            for pattern in GUI_VERSION_PATTERNS:
                m = pattern.match(name)
                if m:
                    versions.add(m.group(1))

    if not versions:
        return ""
    if len(versions) > 1:
        versions_str = ", ".join(sorted(versions))
        raise RuntimeError(
            f"Inconsistent GUI versions found in dist artifacts: {versions_str}"
        )
    return next(iter(versions))


def gui_release_files(target: str, gui_version: str) -> list[tuple[str, str]]:
    if not gui_version:
        return []

    if target == "x86_64-pc-windows-msvc":
        return [("Download", f"pearl-calculator-gui_{gui_version}_x64-setup.exe")]
    if target == "aarch64-pc-windows-msvc":
        return [("Download", f"pearl-calculator-gui_{gui_version}_arm64-setup.exe")]
    if target == "x86_64-apple-darwin":
        return [("Download", f"Pearl Calculator_{gui_version}_x64.dmg")]
    if target == "aarch64-apple-darwin":
        return [("Download", f"Pearl Calculator_{gui_version}_aarch64.dmg")]
    if target == "x86_64-unknown-linux-gnu":
        return [
            ("DEB", f"pearl-calculator-gui_{gui_version}_amd64.deb"),
            ("AppImage", f"pearl-calculator-gui_{gui_version}_x86_64.AppImage"),
            ("Pacman", f"pearl-calculator-gui_{gui_version}_x86_64.tar.gz"),
        ]
    if target == "aarch64-unknown-linux-gnu":
        return [
            ("DEB", f"pearl-calculator-gui_{gui_version}_arm64.deb"),
            ("AppImage", f"pearl-calculator-gui_{gui_version}_aarch64.AppImage"),
            ("Pacman", f"pearl-calculator-gui_{gui_version}_aarch64.tar.gz"),
        ]
    return []


def gui_cell_for_release(target: str, gui_version: str, release_base_url: str) -> str:
    assets = gui_release_files(target, gui_version)
    if not assets:
        return "-"

    return " ".join(
        f"[{label}]({join_url(release_base_url, file_name)})"
        for label, file_name in assets
    )


def run_release_body(dist_root: Path, output: Path) -> None:
    tag_name = os.environ["GITHUB_REF_NAME"]
    server_url = os.environ["GITHUB_SERVER_URL"]
    repository = os.environ["GITHUB_REPOSITORY"]

    release_base_url = f"{server_url}/{repository}/releases/download/{tag_name}"
    gui_version = detect_gui_version(dist_root)

    rows = empty_rows()
    for platform in PLATFORMS:
        row = find_row(rows, platform.target)
        if row is None:
            continue
        cli_file = release_cli_filename(platform.target, tag_name)
        row["cli"] = f"[Download]({release_base_url}/{cli_file})"
        row["gui"] = gui_cell_for_release(
            platform.target, gui_version, release_base_url
        )

    body = "## Downloads\n\n" + render_table(rows)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(body, encoding="utf-8")


def parse_artifact_name(name: str) -> tuple[str, str] | None:
    if name.startswith("pearl-calculator-gui-"):
        return ("gui", name.replace("pearl-calculator-gui-", "", 1))
    if name.startswith("pearl-calculator-"):
        return ("cli", name.replace("pearl-calculator-", "", 1))
    return None


def list_artifacts() -> list[dict]:
    token = os.environ["GITHUB_TOKEN"]
    api_url = os.environ.get("GITHUB_API_URL", "https://api.github.com")
    repository = os.environ["GITHUB_REPOSITORY"]
    run_id = os.environ["GITHUB_RUN_ID"]

    artifacts: list[dict] = []
    page = 1
    while True:
        url = f"{api_url}/repos/{repository}/actions/runs/{run_id}/artifacts?per_page=100&page={page}"
        req = Request(
            url,
            headers={
                "Accept": "application/vnd.github+json",
                "Authorization": f"Bearer {token}",
                "X-GitHub-Api-Version": "2022-11-28",
                "User-Agent": "pearl-calculator-actions",
            },
        )
        with urlopen(req) as resp:
            payload = json.loads(resp.read().decode("utf-8"))

        page_artifacts = payload.get("artifacts", [])
        artifacts.extend(page_artifacts)
        if len(page_artifacts) < 100:
            break
        page += 1

    return artifacts


def run_build_summary(output: Path) -> None:
    server_url = os.environ["GITHUB_SERVER_URL"]
    repository = os.environ["GITHUB_REPOSITORY"]
    run_id = os.environ["GITHUB_RUN_ID"]

    rows = empty_rows()
    artifacts = list_artifacts()

    for artifact in artifacts:
        if artifact.get("expired"):
            continue
        parsed = parse_artifact_name(artifact.get("name", ""))
        if parsed is None:
            continue

        artifact_type, target = parsed
        row = find_row(rows, target)
        if row is None:
            continue

        artifact_url = f"{server_url}/{repository}/actions/runs/{run_id}/artifacts/{artifact['id']}"
        if artifact_type == "cli":
            row["cli"] = f"[Download]({artifact_url})"
        elif artifact_type == "gui":
            row["gui"] = f"[Download]({artifact_url})"

    summary = "## Build Artifacts Summary\n\n" + render_table(rows)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(summary, encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Generate download tables for release and build summary"
    )
    subparsers = parser.add_subparsers(dest="mode", required=True)

    release = subparsers.add_parser("release-body")
    release.add_argument("--dist-root", default="dist")
    release.add_argument("--output", default="release-assets/release-body.md")

    summary = subparsers.add_parser("build-summary")
    summary.add_argument(
        "--output", default=os.environ.get("GITHUB_STEP_SUMMARY", "build-summary.md")
    )

    args = parser.parse_args()

    if args.mode == "release-body":
        run_release_body(Path(args.dist_root), Path(args.output))
    elif args.mode == "build-summary":
        run_build_summary(Path(args.output))


if __name__ == "__main__":
    main()
