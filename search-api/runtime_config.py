#!/usr/bin/env python3
"""Runtime configuration parsing and validation for Search API."""

from __future__ import annotations

from dataclasses import dataclass
import os
from typing import Mapping

DEFAULT_API_KEY = "dev-key-change-in-production"
DEFAULT_API_PORT = 3000
DEFAULT_RATE_LIMIT_STORAGE_URI = "memory://"


class RuntimeConfigError(ValueError):
    """Raised when runtime configuration is invalid."""


@dataclass(frozen=True)
class RuntimeConfig:
    environment: str
    api_key: str
    api_port: int
    rate_limit_storage_uri: str
    db_host: str
    db_port: int
    db_user: str
    db_password: str | None
    db_name: str

    @property
    def is_production(self) -> bool:
        return self.environment.lower() == "production"



def _parse_int(value: str, *, key: str) -> int:
    try:
        return int(value)
    except (TypeError, ValueError) as exc:
        raise RuntimeConfigError(f"{key} must be an integer") from exc



def load_runtime_config(environ: Mapping[str, str] | None = None) -> RuntimeConfig:
    env = dict(os.environ if environ is None else environ)

    environment = env.get("ENVIRONMENT", "development")
    api_key = env.get("ASSISTSUPPORT_API_KEY", DEFAULT_API_KEY)
    api_port = _parse_int(env.get("ASSISTSUPPORT_API_PORT", str(DEFAULT_API_PORT)), key="ASSISTSUPPORT_API_PORT")
    rate_limit_storage_uri = env.get(
        "ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI", DEFAULT_RATE_LIMIT_STORAGE_URI
    )

    db_host = env.get("ASSISTSUPPORT_DB_HOST", "localhost")
    db_port = _parse_int(env.get("ASSISTSUPPORT_DB_PORT", "5432"), key="ASSISTSUPPORT_DB_PORT")
    db_user = env.get("ASSISTSUPPORT_DB_USER", "assistsupport_dev")
    db_password = env.get("ASSISTSUPPORT_DB_PASSWORD") or None
    db_name = env.get("ASSISTSUPPORT_DB_NAME", "assistsupport_dev")

    return RuntimeConfig(
        environment=environment,
        api_key=api_key,
        api_port=api_port,
        rate_limit_storage_uri=rate_limit_storage_uri,
        db_host=db_host,
        db_port=db_port,
        db_user=db_user,
        db_password=db_password,
        db_name=db_name,
    )



def validate_runtime_config(
    config: RuntimeConfig,
    *,
    check_backends: bool = False,
) -> list[str]:
    errors: list[str] = []

    if config.environment.lower() not in {"development", "production", "test"}:
        errors.append("ENVIRONMENT must be one of development, production, or test")

    if not (1 <= config.api_port <= 65535):
        errors.append("ASSISTSUPPORT_API_PORT must be between 1 and 65535")

    if not (1 <= config.db_port <= 65535):
        errors.append("ASSISTSUPPORT_DB_PORT must be between 1 and 65535")

    if config.is_production:
        if config.api_key == DEFAULT_API_KEY:
            errors.append("ASSISTSUPPORT_API_KEY must be set to a non-default value in production")
        if config.rate_limit_storage_uri == DEFAULT_RATE_LIMIT_STORAGE_URI:
            errors.append("ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI must not use memory:// in production")

    if check_backends and config.rate_limit_storage_uri.startswith("redis://"):
        try:
            import redis  # type: ignore

            client = redis.Redis.from_url(
                config.rate_limit_storage_uri,
                socket_connect_timeout=2,
                socket_timeout=2,
            )
            client.ping()
        except Exception as exc:  # pragma: no cover - exercised in integration/smoke
            errors.append(f"Could not connect to rate-limit backend: {exc}")

    return errors



def ensure_valid_runtime_config(
    config: RuntimeConfig,
    *,
    check_backends: bool = False,
) -> None:
    errors = validate_runtime_config(config, check_backends=check_backends)
    if errors:
        raise RuntimeConfigError("; ".join(errors))
