#!/usr/bin/env python3
"""Validate Search API runtime configuration."""

from __future__ import annotations

import argparse
import json
import os
import sys
from typing import Dict

from runtime_config import load_runtime_config, validate_runtime_config



def _build_environ_from_args(args: argparse.Namespace) -> Dict[str, str]:
    env = dict(os.environ)

    if args.environment:
        env["ENVIRONMENT"] = args.environment
    if args.api_key:
        env["ASSISTSUPPORT_API_KEY"] = args.api_key
    if args.api_port is not None:
        env["ASSISTSUPPORT_API_PORT"] = str(args.api_port)
    if args.rate_limit_storage_uri:
        env["ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI"] = args.rate_limit_storage_uri

    return env



def main() -> int:
    parser = argparse.ArgumentParser(description="Validate Search API runtime configuration")
    parser.add_argument("--environment", help="Override ENVIRONMENT")
    parser.add_argument("--api-key", help="Override ASSISTSUPPORT_API_KEY")
    parser.add_argument("--api-port", type=int, help="Override ASSISTSUPPORT_API_PORT")
    parser.add_argument(
        "--rate-limit-storage-uri",
        help="Override ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI",
    )
    parser.add_argument(
        "--check-backends",
        action="store_true",
        help="Verify connectivity to configured backend services (for example redis)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit machine-readable JSON output",
    )

    args = parser.parse_args()
    env = _build_environ_from_args(args)
    config = load_runtime_config(env)
    errors = validate_runtime_config(config, check_backends=args.check_backends)

    payload = {
        "valid": len(errors) == 0,
        "environment": config.environment,
        "api_port": config.api_port,
        "rate_limit_storage_uri": config.rate_limit_storage_uri,
        "errors": errors,
    }

    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print(f"valid={payload['valid']}")
        print(f"environment={payload['environment']}")
        print(f"api_port={payload['api_port']}")
        print(f"rate_limit_storage_uri={payload['rate_limit_storage_uri']}")
        if errors:
            print("errors:")
            for err in errors:
                print(f"- {err}")

    return 0 if payload["valid"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
