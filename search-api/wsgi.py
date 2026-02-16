#!/usr/bin/env python3
"""WSGI entrypoint for production Search API serving."""

from runtime_config import load_runtime_config, ensure_valid_runtime_config

config = load_runtime_config()
ensure_valid_runtime_config(config, check_backends=True)

from search_api import app  # noqa: E402

__all__ = ["app"]
