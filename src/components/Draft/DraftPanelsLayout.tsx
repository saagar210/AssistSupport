import type { ComponentProps } from 'react';
import { InputPanel } from './InputPanel';
import { DiagnosisPanel } from './DiagnosisPanel';
import { ResponsePanel } from './ResponsePanel';
import { AlternativePanel } from './AlternativePanel';
import { SavedResponsesSuggestion } from './SavedResponsesSuggestion';

interface DraftPanelsLayoutProps {
  diagnosisCollapsed: boolean;
  onToggleDiagnosisCollapse: () => void;
  inputPanelProps: ComponentProps<typeof InputPanel>;
  diagnosisPanelProps: Omit<ComponentProps<typeof DiagnosisPanel>, 'collapsed' | 'onToggleCollapse'>;
  responsePanelProps: ComponentProps<typeof ResponsePanel>;
  showSuggestions: boolean;
  suggestionProps: ComponentProps<typeof SavedResponsesSuggestion>;
  showAlternatives: boolean;
  alternativesProps: ComponentProps<typeof AlternativePanel>;
}

export function DraftPanelsLayout({
  diagnosisCollapsed,
  onToggleDiagnosisCollapse,
  inputPanelProps,
  diagnosisPanelProps,
  responsePanelProps,
  showSuggestions,
  suggestionProps,
  showAlternatives,
  alternativesProps,
}: DraftPanelsLayoutProps) {
  return (
    <div className="draft-panels-container">
      <div className="draft-panel input-panel">
        <InputPanel {...inputPanelProps} />
      </div>

      <div className={`draft-panel diagnosis-panel ${diagnosisCollapsed ? 'collapsed' : ''}`}>
        <DiagnosisPanel
          {...diagnosisPanelProps}
          collapsed={diagnosisCollapsed}
          onToggleCollapse={onToggleDiagnosisCollapse}
        />
      </div>

      <div className="draft-panel response-panel">
        {showSuggestions && <SavedResponsesSuggestion {...suggestionProps} />}
        <ResponsePanel {...responsePanelProps} />
        {showAlternatives && <AlternativePanel {...alternativesProps} />}
      </div>
    </div>
  );
}
