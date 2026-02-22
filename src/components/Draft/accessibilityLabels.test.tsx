// @vitest-environment jsdom
import React from "react";
import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { InputPanel } from "./InputPanel";
import { DiagnosisPanel } from "./DiagnosisPanel";
import type { ChecklistItem } from "../../types";

vi.mock("../../hooks/useJira", () => ({
  useJira: () => ({
    checkConfiguration: vi.fn(),
    getTicket: vi.fn(),
    configured: false,
  }),
}));

vi.mock("../../hooks/useDecisionTrees", () => ({
  useDecisionTrees: () => ({
    trees: [{ id: "net-1", name: "Network triage", category: "Network" }],
    loading: false,
    error: null,
    loadTrees: vi.fn(),
    getTree: vi.fn(),
  }),
}));

vi.mock("./AutoSuggest", () => ({
  AutoSuggest: () => <div data-testid="auto-suggest" />,
}));

vi.mock("./VoiceInput", () => ({
  VoiceInput: () => <button type="button">Voice</button>,
}));

vi.mock("./TemplateSelector", () => ({
  TemplateSelector: () => <div data-testid="template-selector" />,
}));

vi.mock("../Batch/BatchPanel", () => ({
  BatchPanel: () => <div data-testid="batch-panel" />,
}));

vi.mock("../Trees/TreeRunner", () => ({
  TreeRunner: () => <div data-testid="tree-runner" />,
}));

vi.mock("../shared/Button", () => ({
  Button: ({
    children,
    onClick,
    disabled,
  }: {
    children: React.ReactNode;
    onClick?: () => void;
    disabled?: boolean;
  }) => (
    <button type="button" onClick={onClick} disabled={disabled}>
      {children}
    </button>
  ),
}));

describe("Draft panel accessibility labels", () => {
  it("renders labeled selects in InputPanel", () => {
    render(
      <InputPanel
        value="Ticket detail"
        onChange={vi.fn()}
        ocrText={null}
        onOcrTextChange={vi.fn()}
        onGenerate={vi.fn()}
        onClear={vi.fn()}
        generating={false}
        modelLoaded={true}
        responseLength="Medium"
        onResponseLengthChange={vi.fn()}
        ticketId={null}
        onTicketIdChange={vi.fn()}
        ticket={null}
        onTicketChange={vi.fn()}
        firstResponse=""
        onFirstResponseChange={vi.fn()}
        firstResponseTone="slack"
        onFirstResponseToneChange={vi.fn()}
        onGenerateFirstResponse={vi.fn()}
        onCopyFirstResponse={vi.fn()}
        onClearFirstResponse={vi.fn()}
        firstResponseGenerating={false}
      />
    );

    expect(screen.getByLabelText("Ticket task preset")).toBeTruthy();
    expect(screen.getByLabelText("Response length")).toBeTruthy();
    expect(screen.getByLabelText("First response tone")).toBeTruthy();
  });

  it("renders labeled troubleshooting tree dropdown in DiagnosisPanel", () => {
    const checklistItems: ChecklistItem[] = [];

    render(
      <DiagnosisPanel
        input="Agent cannot connect"
        ocrText={null}
        notes=""
        onNotesChange={vi.fn()}
        treeResult={null}
        onTreeComplete={vi.fn()}
        onTreeClear={vi.fn()}
        checklistItems={checklistItems}
        checklistCompleted={{}}
        checklistGenerating={false}
        checklistUpdating={false}
        checklistError={null}
        onChecklistToggle={vi.fn()}
        onChecklistGenerate={vi.fn()}
        onChecklistUpdate={vi.fn()}
        onChecklistClear={vi.fn()}
        approvalQuery=""
        onApprovalQueryChange={vi.fn()}
        approvalResults={[]}
        approvalSearching={false}
        approvalSummary=""
        approvalSummarizing={false}
        approvalSources={[]}
        onApprovalSearch={vi.fn()}
        onApprovalSummarize={vi.fn()}
        approvalError={null}
        modelLoaded={true}
        hasTicket={false}
        collapsed={false}
        onToggleCollapse={vi.fn()}
      />
    );

    expect(screen.getByLabelText("Troubleshooting tree")).toBeTruthy();
  });
});
