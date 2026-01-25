import { useState, useEffect } from 'react';
import { useKnowledge } from '../../hooks/useKnowledge';
import { useToastContext } from '../../contexts/ToastContext';
import { Button } from '../shared/Button';
import './KnowledgeBrowser.css';

export function KnowledgeBrowser() {
  const {
    namespaces,
    selectedNamespace,
    documents,
    selectedDocument,
    chunks,
    loading,
    error,
    loadNamespaces,
    selectNamespace,
    selectDocument,
    deleteNamespace,
    deleteSource,
    deleteDocument,
    clearAll,
  } = useKnowledge();

  const { success: showSuccess, error: showError } = useToastContext();
  const [confirmDelete, setConfirmDelete] = useState<{
    type: 'namespace' | 'source' | 'document' | 'clear';
    id: string;
    name: string;
  } | null>(null);

  useEffect(() => {
    loadNamespaces();
  }, [loadNamespaces]);

  useEffect(() => {
    if (error) {
      showError(error);
    }
  }, [error, showError]);

  const handleDeleteConfirm = async () => {
    if (!confirmDelete) return;

    try {
      switch (confirmDelete.type) {
        case 'namespace':
          await deleteNamespace(confirmDelete.id);
          showSuccess(`Deleted namespace "${confirmDelete.name}"`);
          break;
        case 'source':
          await deleteSource(confirmDelete.id);
          showSuccess(`Deleted source "${confirmDelete.name}"`);
          break;
        case 'document':
          await deleteDocument(confirmDelete.id);
          showSuccess('Document deleted');
          break;
        case 'clear':
          await clearAll(confirmDelete.id === 'all' ? undefined : confirmDelete.id);
          showSuccess(confirmDelete.id === 'all' ? 'All knowledge data cleared' : `Cleared namespace "${confirmDelete.name}"`);
          break;
      }
    } catch (e) {
      showError(`Delete failed: ${e}`);
    } finally {
      setConfirmDelete(null);
    }
  };

  const formatDate = (dateString?: string | null) => {
    if (!dateString) return 'Unknown';
    try {
      return new Date(dateString).toLocaleDateString(undefined, {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      });
    } catch {
      return dateString;
    }
  };

  const getSourceTypeIcon = (sourceType: string) => {
    switch (sourceType) {
      case 'file': return '📄';
      case 'web': return '🌐';
      case 'youtube': return '🎬';
      case 'github': return '🐙';
      default: return '📦';
    }
  };

  return (
    <div className="knowledge-browser">
      {/* Confirmation Modal */}
      {confirmDelete && (
        <div className="confirm-modal-overlay">
          <div className="confirm-modal">
            <h3>Confirm Delete</h3>
            <p>
              {confirmDelete.type === 'clear'
                ? confirmDelete.id === 'all'
                  ? 'This will permanently delete ALL knowledge data from ALL namespaces. This cannot be undone.'
                  : `This will delete all documents and sources from "${confirmDelete.name}". This cannot be undone.`
                : `Are you sure you want to delete ${confirmDelete.type} "${confirmDelete.name}"? This cannot be undone.`}
            </p>
            <div className="confirm-modal-actions">
              <Button variant="secondary" onClick={() => setConfirmDelete(null)}>
                Cancel
              </Button>
              <Button variant="danger" onClick={handleDeleteConfirm}>
                Delete
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Namespace List */}
      <div className="kb-panel kb-namespaces">
        <div className="kb-panel-header">
          <h3>Namespaces</h3>
          <Button
            variant="danger"
            size="small"
            onClick={() => setConfirmDelete({ type: 'clear', id: 'all', name: 'all namespaces' })}
            disabled={loading || namespaces.length === 0}
          >
            Clear All
          </Button>
        </div>
        <div className="kb-panel-content">
          {loading && namespaces.length === 0 ? (
            <div className="kb-loading">Loading...</div>
          ) : namespaces.length === 0 ? (
            <div className="kb-empty">No namespaces found</div>
          ) : (
            <ul className="kb-list">
              {namespaces.map(ns => (
                <li
                  key={ns.id}
                  className={`kb-list-item ${selectedNamespace === ns.id ? 'selected' : ''}`}
                  onClick={() => selectNamespace(ns.id)}
                >
                  <div className="kb-item-main">
                    <span className="kb-item-name">{ns.name}</span>
                    <span className="kb-item-counts">
                      {ns.documentCount} docs, {ns.sourceCount} sources
                    </span>
                  </div>
                  {ns.id !== 'default' && (
                    <button
                      className="kb-item-delete"
                      onClick={(e) => {
                        e.stopPropagation();
                        setConfirmDelete({ type: 'namespace', id: ns.id, name: ns.name });
                      }}
                      title="Delete namespace"
                    >
                      ×
                    </button>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      {/* Documents List */}
      <div className="kb-panel kb-documents">
        <div className="kb-panel-header">
          <h3>Documents</h3>
          {selectedNamespace && (
            <Button
              variant="secondary"
              size="small"
              onClick={() => {
                const ns = namespaces.find(n => n.id === selectedNamespace);
                if (ns) {
                  setConfirmDelete({ type: 'clear', id: ns.id, name: ns.name });
                }
              }}
              disabled={loading || documents.length === 0}
            >
              Clear Namespace
            </Button>
          )}
        </div>
        <div className="kb-panel-content">
          {!selectedNamespace ? (
            <div className="kb-empty">Select a namespace to view documents</div>
          ) : loading && documents.length === 0 ? (
            <div className="kb-loading">Loading...</div>
          ) : documents.length === 0 ? (
            <div className="kb-empty">No documents in this namespace</div>
          ) : (
            <ul className="kb-list">
              {documents.map(doc => (
                <li
                  key={doc.id}
                  className={`kb-list-item ${selectedDocument?.id === doc.id ? 'selected' : ''}`}
                  onClick={() => selectDocument(doc)}
                >
                  <div className="kb-item-main">
                    <span className="kb-item-icon">{getSourceTypeIcon(doc.source_type)}</span>
                    <div className="kb-item-info">
                      <span className="kb-item-title">{doc.title || doc.file_path}</span>
                      <span className="kb-item-meta">
                        {doc.chunk_count} chunks · {formatDate(doc.indexed_at)}
                      </span>
                    </div>
                  </div>
                  <button
                    className="kb-item-delete"
                    onClick={(e) => {
                      e.stopPropagation();
                      setConfirmDelete({
                        type: 'document',
                        id: doc.id,
                        name: doc.title || doc.file_path,
                      });
                    }}
                    title="Delete document"
                  >
                    ×
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      {/* Chunks Preview */}
      <div className="kb-panel kb-chunks">
        <div className="kb-panel-header">
          <h3>Chunks</h3>
          {selectedDocument && (
            <span className="kb-chunk-count">{chunks.length} chunks</span>
          )}
        </div>
        <div className="kb-panel-content">
          {!selectedDocument ? (
            <div className="kb-empty">Select a document to view chunks</div>
          ) : loading && chunks.length === 0 ? (
            <div className="kb-loading">Loading...</div>
          ) : chunks.length === 0 ? (
            <div className="kb-empty">No chunks found</div>
          ) : (
            <div className="kb-chunks-list">
              {chunks.map(chunk => (
                <div key={chunk.id} className="kb-chunk">
                  <div className="kb-chunk-header">
                    <span className="kb-chunk-index">#{chunk.chunk_index + 1}</span>
                    {chunk.heading_path && (
                      <span className="kb-chunk-heading">{chunk.heading_path}</span>
                    )}
                    {chunk.word_count && (
                      <span className="kb-chunk-words">{chunk.word_count} words</span>
                    )}
                  </div>
                  <div className="kb-chunk-content">{chunk.content}</div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
