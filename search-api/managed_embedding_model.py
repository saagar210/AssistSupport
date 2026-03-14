#!/usr/bin/env python3
"""
Managed embedding model installation and status for the search-api.

This module keeps the search-api embedding model on a pinned Hugging Face
revision, stores it under the AssistSupport app data directory, and loads it
from local disk only at runtime.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from tempfile import NamedTemporaryFile
from typing import Any


MODEL_NAME = "sentence-transformers/all-MiniLM-L6-v2"
MODEL_REVISION = "c9745ed1d9f207416be6d2e6f8de32d1f16199bf"
MODEL_ID = "search-api-all-minilm-l6-v2"
MANIFEST_VERSION = 1
MODEL_SUBDIR = Path("managed-search-api-models") / MODEL_ID
MANIFEST_NAME = "manifest.json"
REQUIRED_FILES = [
    "1_Pooling/config.json",
    "config.json",
    "config_sentence_transformers.json",
    "model.safetensors",
    "modules.json",
    "sentence_bert_config.json",
    "special_tokens_map.json",
    "tokenizer.json",
    "tokenizer_config.json",
    "vocab.txt",
]


class ManagedEmbeddingModelError(RuntimeError):
    """Raised when the managed embedding model is unavailable or invalid."""


@dataclass(frozen=True)
class SearchApiEmbeddingModelStatus:
    installed: bool
    ready: bool
    model_name: str
    revision: str
    local_path: str | None
    error: str | None


def _default_app_data_dir() -> Path:
    override = os.environ.get("ASSISTSUPPORT_APP_DATA_DIR")
    if override:
        return Path(override).expanduser().resolve()

    home = Path.home()
    if sys_platform() == "darwin":
        return (home / "Library" / "Application Support" / "AssistSupport").resolve()
    if sys_platform() == "win32":
        appdata = os.environ.get("APPDATA")
        base = Path(appdata) if appdata else home / "AppData" / "Roaming"
        return (base / "AssistSupport").resolve()

    xdg = os.environ.get("XDG_DATA_HOME")
    base = Path(xdg) if xdg else home / ".local" / "share"
    return (base / "AssistSupport").resolve()


def sys_platform() -> str:
    import sys

    return sys.platform


def get_app_data_dir(app_data_dir: str | Path | None = None) -> Path:
    if app_data_dir is None:
        return _default_app_data_dir()
    return Path(app_data_dir).expanduser().resolve()


def get_model_dir(app_data_dir: str | Path | None = None) -> Path:
    return get_app_data_dir(app_data_dir) / MODEL_SUBDIR


def get_manifest_path(app_data_dir: str | Path | None = None) -> Path:
    return get_model_dir(app_data_dir) / MANIFEST_NAME


def _compute_sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def _write_json_atomic(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with NamedTemporaryFile("w", encoding="utf-8", delete=False, dir=path.parent) as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")
        temp_path = Path(handle.name)
    temp_path.replace(path)


def _build_manifest(model_dir: Path) -> dict[str, Any]:
    files = {
        relative_path: {
            "sha256": _compute_sha256(model_dir / relative_path),
            "size_bytes": (model_dir / relative_path).stat().st_size,
        }
        for relative_path in REQUIRED_FILES
    }
    return {
        "manifest_version": MANIFEST_VERSION,
        "model_id": MODEL_ID,
        "model_name": MODEL_NAME,
        "revision": MODEL_REVISION,
        "local_path": str(model_dir),
        "required_files": REQUIRED_FILES,
        "files": files,
        "installed_at": datetime.now(timezone.utc).isoformat(),
    }


def _missing_files(model_dir: Path) -> list[str]:
    return [relative_path for relative_path in REQUIRED_FILES if not (model_dir / relative_path).exists()]


def _hash_mismatches(model_dir: Path, manifest: dict[str, Any]) -> list[str]:
    files = manifest.get("files")
    if not isinstance(files, dict):
        return ["manifest files metadata missing"]

    mismatches: list[str] = []
    for relative_path in REQUIRED_FILES:
        expected = files.get(relative_path, {})
        expected_hash = expected.get("sha256") if isinstance(expected, dict) else None
        if not expected_hash:
            mismatches.append(f"{relative_path} (missing expected hash)")
            continue

        actual_path = model_dir / relative_path
        if not actual_path.exists():
            continue

        actual_hash = _compute_sha256(actual_path)
        if actual_hash != expected_hash:
            mismatches.append(relative_path)

    return mismatches


def _load_manifest(app_data_dir: str | Path | None = None) -> dict[str, Any] | None:
    manifest_path = get_manifest_path(app_data_dir)
    if not manifest_path.exists():
        return None
    with manifest_path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def get_model_status(app_data_dir: str | Path | None = None) -> SearchApiEmbeddingModelStatus:
    model_dir = get_model_dir(app_data_dir)
    manifest = _load_manifest(app_data_dir)

    if manifest is None:
        return SearchApiEmbeddingModelStatus(
            installed=False,
            ready=False,
            model_name=MODEL_NAME,
            revision=MODEL_REVISION,
            local_path=None,
            error="Managed search-api embedding model is not installed. Install it from AssistSupport Settings.",
        )

    missing_files = _missing_files(model_dir)
    local_path = str(model_dir)
    if missing_files:
        return SearchApiEmbeddingModelStatus(
            installed=True,
            ready=False,
            model_name=str(manifest.get("model_name", MODEL_NAME)),
            revision=str(manifest.get("revision", MODEL_REVISION)),
            local_path=local_path,
            error=f"Managed embedding model is incomplete. Missing files: {', '.join(missing_files)}",
        )

    if str(manifest.get("revision", MODEL_REVISION)) != MODEL_REVISION:
        return SearchApiEmbeddingModelStatus(
            installed=True,
            ready=False,
            model_name=str(manifest.get("model_name", MODEL_NAME)),
            revision=str(manifest.get("revision", MODEL_REVISION)),
            local_path=local_path,
            error="Managed embedding model revision does not match the pinned release. Reinstall it from AssistSupport Settings.",
        )

    mismatches = _hash_mismatches(model_dir, manifest)
    if mismatches:
        return SearchApiEmbeddingModelStatus(
            installed=True,
            ready=False,
            model_name=str(manifest.get("model_name", MODEL_NAME)),
            revision=str(manifest.get("revision", MODEL_REVISION)),
            local_path=local_path,
            error=f"Managed embedding model failed integrity verification. Reinstall it from AssistSupport Settings. Mismatched files: {', '.join(mismatches)}",
        )

    return SearchApiEmbeddingModelStatus(
        installed=True,
        ready=True,
        model_name=str(manifest.get("model_name", MODEL_NAME)),
        revision=str(manifest.get("revision", MODEL_REVISION)),
        local_path=local_path,
        error=None,
    )


def install_model(app_data_dir: str | Path | None = None) -> SearchApiEmbeddingModelStatus:
    from huggingface_hub import snapshot_download

    model_dir = get_model_dir(app_data_dir)
    model_dir.mkdir(parents=True, exist_ok=True)

    snapshot_download(
        repo_id=MODEL_NAME,
        revision=MODEL_REVISION,
        local_dir=str(model_dir),
        local_dir_use_symlinks=False,
        allow_patterns=REQUIRED_FILES,
    )

    missing_files = _missing_files(model_dir)
    if missing_files:
        raise ManagedEmbeddingModelError(
            f"Managed embedding install is incomplete. Missing files: {', '.join(missing_files)}"
        )

    manifest = _build_manifest(model_dir)
    _write_json_atomic(get_manifest_path(app_data_dir), manifest)
    return get_model_status(app_data_dir)


def resolve_model_path(app_data_dir: str | Path | None = None) -> Path:
    status = get_model_status(app_data_dir)
    if not status.ready or not status.local_path:
        raise ManagedEmbeddingModelError(status.error or "Managed embedding model is not ready")
    return Path(status.local_path)


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Manage the search-api embedding model")
    parser.add_argument("action", choices=["status", "install"])
    parser.add_argument("--app-data-dir", dest="app_data_dir")
    parser.add_argument("--json", action="store_true", dest="as_json")
    return parser.parse_args()


def _print_status(status: SearchApiEmbeddingModelStatus, as_json_output: bool) -> None:
    if as_json_output:
        print(json.dumps(asdict(status)))
        return

    print(f"installed={status.installed}")
    print(f"ready={status.ready}")
    print(f"model_name={status.model_name}")
    print(f"revision={status.revision}")
    if status.local_path:
        print(f"local_path={status.local_path}")
    if status.error:
        print(f"error={status.error}")


def main() -> int:
    args = _parse_args()
    try:
        if args.action == "status":
            status = get_model_status(args.app_data_dir)
        else:
            status = install_model(args.app_data_dir)
        _print_status(status, args.as_json)
        return 0
    except Exception as exc:
        if args.as_json:
            print(
                json.dumps(
                    {
                        "installed": False,
                        "ready": False,
                        "model_name": MODEL_NAME,
                        "revision": MODEL_REVISION,
                        "local_path": None,
                        "error": str(exc),
                    }
                )
            )
        else:
            print(str(exc))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
