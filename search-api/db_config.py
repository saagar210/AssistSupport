#!/usr/bin/env python3
"""
Database configuration helpers for search-api scripts.

Connection values come from environment variables:
- ASSISTSUPPORT_DB_HOST (default: localhost)
- ASSISTSUPPORT_DB_PORT (default: 5432)
- ASSISTSUPPORT_DB_USER (default: assistsupport_dev)
- ASSISTSUPPORT_DB_PASSWORD (default: empty)
- ASSISTSUPPORT_DB_NAME (default: assistsupport_dev)
"""

import os


def get_db_conn_kwargs():
    """Build psycopg2 connection kwargs from environment variables."""
    kwargs = {
        "host": os.environ.get("ASSISTSUPPORT_DB_HOST", "localhost"),
        "port": int(os.environ.get("ASSISTSUPPORT_DB_PORT", "5432")),
        "user": os.environ.get("ASSISTSUPPORT_DB_USER", "assistsupport_dev"),
        "database": os.environ.get("ASSISTSUPPORT_DB_NAME", "assistsupport_dev"),
    }

    # Only include password when explicitly configured.
    password = os.environ.get("ASSISTSUPPORT_DB_PASSWORD")
    if password:
        kwargs["password"] = password

    return kwargs


def connect_db():
    """Create a PostgreSQL connection using environment-backed config."""
    import psycopg2

    return psycopg2.connect(**get_db_conn_kwargs())
