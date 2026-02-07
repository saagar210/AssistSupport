import sys
from pathlib import Path

import pytest

SEARCH_API_DIR = Path(__file__).resolve().parents[1]
if str(SEARCH_API_DIR) not in sys.path:
    sys.path.insert(0, str(SEARCH_API_DIR))

from runtime_config import (  # noqa: E402
    RuntimeConfig,
    RuntimeConfigError,
    DEFAULT_API_KEY,
    load_runtime_config,
    validate_runtime_config,
)


def test_load_runtime_config_rejects_invalid_port():
    with pytest.raises(RuntimeConfigError, match="ASSISTSUPPORT_API_PORT"):
        load_runtime_config({"ASSISTSUPPORT_API_PORT": "not-a-number"})


def test_validate_runtime_config_rejects_default_key_in_production():
    config = RuntimeConfig(
        environment="production",
        api_key=DEFAULT_API_KEY,
        api_port=3000,
        rate_limit_storage_uri="redis://127.0.0.1:6379/0",
        db_host="localhost",
        db_port=5432,
        db_user="assistsupport_dev",
        db_password=None,
        db_name="assistsupport_dev",
    )

    errors = validate_runtime_config(config)
    assert any("ASSISTSUPPORT_API_KEY" in e for e in errors)


def test_validate_runtime_config_rejects_memory_rate_limit_in_production():
    config = RuntimeConfig(
        environment="production",
        api_key="real-key",
        api_port=3000,
        rate_limit_storage_uri="memory://",
        db_host="localhost",
        db_port=5432,
        db_user="assistsupport_dev",
        db_password=None,
        db_name="assistsupport_dev",
    )

    errors = validate_runtime_config(config)
    assert any("ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI" in e for e in errors)


def test_validate_runtime_config_reports_backend_connectivity_errors():
    config = RuntimeConfig(
        environment="production",
        api_key="real-key",
        api_port=3000,
        rate_limit_storage_uri="redis://127.0.0.1:6399/0",
        db_host="localhost",
        db_port=5432,
        db_user="assistsupport_dev",
        db_password=None,
        db_name="assistsupport_dev",
    )

    errors = validate_runtime_config(config, check_backends=True)
    assert any("Could not connect to rate-limit backend" in e for e in errors)


def test_validate_runtime_config_allows_development_defaults():
    config = RuntimeConfig(
        environment="development",
        api_key=DEFAULT_API_KEY,
        api_port=3000,
        rate_limit_storage_uri="memory://",
        db_host="localhost",
        db_port=5432,
        db_user="assistsupport_dev",
        db_password=None,
        db_name="assistsupport_dev",
    )

    assert validate_runtime_config(config) == []
