import os
import subprocess
import sys
from pathlib import Path


SEARCH_API_DIR = Path(__file__).resolve().parents[1]
REPO_ROOT = SEARCH_API_DIR.parent
WSGI_PATH = SEARCH_API_DIR / "wsgi.py"


def _run_wsgi_import(*, cwd: Path, env_overrides: dict[str, str] | None = None):
    env = os.environ.copy()
    env.update(
        {
            "ENVIRONMENT": "development",
            "ASSISTSUPPORT_SEARCH_API_REQUIRE_AUTH": "0",
        }
    )
    if env_overrides:
        env.update(env_overrides)

    return subprocess.run(
        [
            sys.executable,
            "-c",
            (
                "import runpy; "
                f"module_globals = runpy.run_path({str(WSGI_PATH)!r}); "
                "print(module_globals['app'].__class__.__name__)"
            ),
        ],
        cwd=str(cwd),
        env=env,
        capture_output=True,
        text=True,
    )


def test_wsgi_entrypoint_imports_from_repo_root():
    result = _run_wsgi_import(cwd=REPO_ROOT)

    assert result.returncode == 0, result.stderr
    assert "Flask" in result.stdout


def test_wsgi_entrypoint_imports_from_search_api_cwd():
    result = _run_wsgi_import(cwd=SEARCH_API_DIR)

    assert result.returncode == 0, result.stderr
    assert "Flask" in result.stdout


def test_wsgi_entrypoint_fails_fast_on_invalid_runtime():
    result = _run_wsgi_import(
        cwd=REPO_ROOT,
        env_overrides={
            "ENVIRONMENT": "production",
            "ASSISTSUPPORT_API_KEY": "secret-key",
            "ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI": "memory://",
        },
    )

    assert result.returncode != 0
    assert "ASSISTSUPPORT_RATE_LIMIT_STORAGE_URI" in result.stderr
