from pathlib import Path

from managed_embedding_model import (
    MODEL_NAME,
    MODEL_REVISION,
    REQUIRED_FILES,
    get_manifest_path,
    get_model_status,
    install_model,
)


def _write_required_files(model_dir: Path) -> None:
    for relative_path in REQUIRED_FILES:
        target = model_dir / relative_path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(f"test:{relative_path}".encode("utf-8"))


def test_status_reports_missing_install(tmp_path):
    status = get_model_status(tmp_path)

    assert status.installed is False
    assert status.ready is False
    assert status.model_name == MODEL_NAME
    assert "not installed" in (status.error or "").lower()


def test_install_writes_manifest_and_reports_ready(tmp_path, monkeypatch):
    def fake_snapshot_download(*, local_dir, **_kwargs):
        _write_required_files(Path(local_dir))
        return local_dir

    monkeypatch.setattr('huggingface_hub.snapshot_download', fake_snapshot_download)

    status = install_model(tmp_path)
    manifest_path = get_manifest_path(tmp_path)

    assert status.installed is True
    assert status.ready is True
    assert status.revision == MODEL_REVISION
    assert manifest_path.exists()

    manifest = manifest_path.read_text(encoding='utf-8')
    assert MODEL_NAME in manifest
    assert MODEL_REVISION in manifest


def test_status_reports_missing_files_after_install(tmp_path, monkeypatch):
    def fake_snapshot_download(*, local_dir, **_kwargs):
        _write_required_files(Path(local_dir))
        return local_dir

    monkeypatch.setattr('huggingface_hub.snapshot_download', fake_snapshot_download)

    ready_status = install_model(tmp_path)
    assert ready_status.ready is True

    missing_file = Path(ready_status.local_path) / REQUIRED_FILES[0]
    missing_file.unlink()

    degraded_status = get_model_status(tmp_path)
    assert degraded_status.installed is True
    assert degraded_status.ready is False
    assert REQUIRED_FILES[0] in (degraded_status.error or '')


def test_status_reports_hash_mismatch(tmp_path, monkeypatch):
    def fake_snapshot_download(*, local_dir, **_kwargs):
        _write_required_files(Path(local_dir))
        return local_dir

    monkeypatch.setattr('huggingface_hub.snapshot_download', fake_snapshot_download)

    ready_status = install_model(tmp_path)
    assert ready_status.ready is True

    tampered_file = Path(ready_status.local_path) / REQUIRED_FILES[1]
    tampered_file.write_text('tampered', encoding='utf-8')

    degraded_status = get_model_status(tmp_path)
    assert degraded_status.installed is True
    assert degraded_status.ready is False
    assert 'integrity verification' in (degraded_status.error or '').lower()
