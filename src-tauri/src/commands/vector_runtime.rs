use crate::db::{get_vectors_dir, CURRENT_VECTOR_STORE_VERSION};
use crate::kb::vectors::{VectorStore, VectorStoreConfig};
use crate::AppState;

fn current_vector_embedding_dim(state: &AppState) -> usize {
    let emb_guard = state.embeddings.read();
    emb_guard
        .as_ref()
        .and_then(|engine| engine.embedding_dim())
        .unwrap_or(768)
}

pub(crate) async fn ensure_vector_store_initialized(state: &AppState) -> Result<(), String> {
    let embedding_dim = current_vector_embedding_dim(state);
    let mut vectors_lock = state.vectors.write().await;
    if vectors_lock.is_some() {
        return Ok(());
    }

    let config = VectorStoreConfig {
        path: get_vectors_dir(),
        embedding_dim,
        encryption_enabled: false,
    };

    let mut store = VectorStore::new(config);
    store.init().await.map_err(|e| e.to_string())?;
    store.create_table().await.map_err(|e| e.to_string())?;
    *vectors_lock = Some(store);
    Ok(())
}

pub(crate) async fn vector_store_requires_rebuild(
    tracked_version: i32,
    store: &VectorStore,
) -> Result<bool, String> {
    if tracked_version < CURRENT_VECTOR_STORE_VERSION {
        return Ok(true);
    }

    store.requires_rebuild().await.map_err(|e| e.to_string())
}

pub(crate) async fn quarantine_vector_store(state: &AppState) -> Result<(), String> {
    ensure_vector_store_initialized(state).await?;

    {
        let mut vectors_lock = state.vectors.write().await;
        let store = vectors_lock.as_mut().ok_or("Vector store not initialized")?;
        store.reset_table().await.map_err(|e| e.to_string())?;
        store.disable();
    }

    let db_lock = state.db.lock().map_err(|e| e.to_string())?;
    let db = db_lock.as_ref().ok_or("Database not initialized")?;
    db.set_vector_store_version(0).map_err(|e| e.to_string())
}

pub(crate) async fn purge_vectors_for_document(
    state: &AppState,
    document_id: &str,
) -> Result<(), String> {
    ensure_vector_store_initialized(state).await?;

    let tracked_version = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        db.get_vector_store_version().map_err(|e| e.to_string())?
    };

    let requires_rebuild = {
        let vectors_lock = state.vectors.read().await;
        let vectors = vectors_lock.as_ref().ok_or("Vector store not initialized")?;
        vector_store_requires_rebuild(tracked_version, vectors).await?
    };

    if requires_rebuild {
        quarantine_vector_store(state).await?;
        return Ok(());
    }

    let vectors_lock = state.vectors.read().await;
    let vectors = vectors_lock.as_ref().ok_or("Vector store not initialized")?;
    vectors
        .delete_by_document(document_id)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn purge_vectors_for_namespace(
    state: &AppState,
    namespace_id: &str,
) -> Result<(), String> {
    ensure_vector_store_initialized(state).await?;

    let tracked_version = {
        let db_lock = state.db.lock().map_err(|e| e.to_string())?;
        let db = db_lock.as_ref().ok_or("Database not initialized")?;
        db.get_vector_store_version().map_err(|e| e.to_string())?
    };

    let requires_rebuild = {
        let vectors_lock = state.vectors.read().await;
        let vectors = vectors_lock.as_ref().ok_or("Vector store not initialized")?;
        vector_store_requires_rebuild(tracked_version, vectors).await?
    };

    if requires_rebuild {
        quarantine_vector_store(state).await?;
        return Ok(());
    }

    let vectors_lock = state.vectors.read().await;
    let vectors = vectors_lock.as_ref().ok_or("Vector store not initialized")?;
    vectors
        .delete_by_namespace(namespace_id)
        .await
        .map_err(|e| e.to_string())
}
