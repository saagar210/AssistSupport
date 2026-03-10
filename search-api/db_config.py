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
from contextlib import contextmanager
from threading import Lock


_DB_POOL = None
_DB_POOL_LOCK = Lock()


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


def get_db_pool():
    """Create or return the process-global threaded connection pool."""
    global _DB_POOL
    if _DB_POOL is None:
        with _DB_POOL_LOCK:
            if _DB_POOL is None:
                from psycopg2.pool import ThreadedConnectionPool

                minconn = int(os.environ.get("ASSISTSUPPORT_DB_POOL_MIN", "1"))
                maxconn = int(os.environ.get("ASSISTSUPPORT_DB_POOL_MAX", "4"))
                _DB_POOL = ThreadedConnectionPool(minconn, maxconn, **get_db_conn_kwargs())
    return _DB_POOL


def close_db_pool():
    """Close the pooled database connections."""
    global _DB_POOL
    if _DB_POOL is not None:
        _DB_POOL.closeall()
        _DB_POOL = None


@contextmanager
def pooled_connection():
    """Yield an autocommit connection from the shared threaded pool."""
    pool = get_db_pool()
    conn = pool.getconn()
    try:
        conn.autocommit = True
        yield conn
    finally:
        pool.putconn(conn)


def check_db_connection():
    """Verify the configured PostgreSQL backend is reachable."""
    with pooled_connection() as conn:
        with conn.cursor() as cur:
            cur.execute("SELECT 1")
            return cur.fetchone()[0] == 1
