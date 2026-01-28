import { useState, useRef, useEffect, useCallback } from 'react';
import type { ResponseTemplate, TemplateContext } from '../../types';
import { applyTemplate } from '../../utils/templates';
import { Button } from '../shared/Button';
import './TemplateSelector.css';

interface TemplateSelectorProps {
  templates: ResponseTemplate[];
  onSelectTemplate: (content: string) => void;
  templateContext?: TemplateContext;
  customVariables?: Array<{ id: string; name: string; value: string; created_at: string }>;
}

interface GroupedTemplates {
  [category: string]: ResponseTemplate[];
}

export function TemplateSelector({
  templates,
  onSelectTemplate,
  templateContext,
  customVariables,
}: TemplateSelectorProps) {
  const [open, setOpen] = useState(false);
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Close dropdown on outside click
  useEffect(() => {
    if (!open) return;

    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setOpen(false);
        setHoveredId(null);
      }
    }

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [open]);

  // Close on escape
  useEffect(() => {
    if (!open) return;

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setOpen(false);
        setHoveredId(null);
      }
    }

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [open]);

  const handleToggle = useCallback(() => {
    setOpen(prev => !prev);
    setHoveredId(null);
  }, []);

  const handleSelect = useCallback(
    (template: ResponseTemplate) => {
      const processed = applyTemplate(
        template.content,
        templateContext || {},
        customVariables || []
      );
      onSelectTemplate(processed);
      setOpen(false);
      setHoveredId(null);
    },
    [onSelectTemplate, templateContext, customVariables]
  );

  // Group templates by category
  const grouped: GroupedTemplates = {};
  for (const template of templates) {
    const category = template.category || 'Uncategorized';
    if (!grouped[category]) {
      grouped[category] = [];
    }
    grouped[category].push(template);
  }

  const categoryNames = Object.keys(grouped).sort();
  const hoveredTemplate = hoveredId ? templates.find(t => t.id === hoveredId) : null;

  if (templates.length === 0) {
    return null;
  }

  return (
    <div className="template-selector" ref={dropdownRef}>
      <Button
        variant="ghost"
        size="small"
        onClick={handleToggle}
        aria-expanded={open}
        aria-haspopup="true"
      >
        Templates
      </Button>

      {open && (
        <div className="template-dropdown">
          <div className="template-menu">
            {categoryNames.map(category => (
              <div key={category} className="template-category">
                <div className="template-category-name">{category}</div>
                {grouped[category].map(template => (
                  <button
                    key={template.id}
                    className={`template-item ${hoveredId === template.id ? 'active' : ''}`}
                    onClick={() => handleSelect(template)}
                    onMouseEnter={() => setHoveredId(template.id)}
                    onFocus={() => setHoveredId(template.id)}
                  >
                    {template.name}
                  </button>
                ))}
              </div>
            ))}
          </div>

          {hoveredTemplate && (
            <div className="template-preview">
              <div className="template-preview-title">{hoveredTemplate.name}</div>
              <div className="template-preview-content">
                {hoveredTemplate.content.length > 300
                  ? hoveredTemplate.content.slice(0, 300) + '...'
                  : hoveredTemplate.content}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
