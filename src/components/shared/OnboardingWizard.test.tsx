// @vitest-environment jsdom
import React from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { OnboardingWizard } from "./OnboardingWizard";

const openMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: (...args: unknown[]) => openMock(...args),
}));

vi.mock("../../hooks/useDownload", () => ({
  useDownload: () => ({
    isDownloading: false,
    downloadProgress: null,
    error: null,
    downloadModel: vi.fn(),
    cancelDownload: vi.fn(),
  }),
}));

vi.mock("./Dialog", () => ({
  Dialog: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

vi.mock("./Icon", () => ({
  Icon: ({ name }: { name: string }) => <span>{name}</span>,
}));

afterEach(() => {
  cleanup();
  openMock.mockReset();
});

describe("OnboardingWizard", () => {
  it("treats security setup as informational instead of fake completion", () => {
    render(<OnboardingWizard onComplete={vi.fn()} onSkip={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "Get Started" }));

    expect(
      screen.getByText(/no extra setup is required during onboarding/i),
    ).toBeTruthy();
    expect(
      screen.getByText(
        /advanced security options and recovery settings live in settings/i,
      ),
    ).toBeTruthy();
    expect(screen.queryByText(/security mode configured/i)).toBeNull();
  });

  it("points users to Settings for semantic-search model setup", () => {
    render(<OnboardingWizard onComplete={vi.fn()} onSkip={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: "Get Started" }));
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(
      screen.getByText(/semantic-search models are also managed there/i),
    ).toBeTruthy();

    fireEvent.click(
      screen.getByRole("button", { name: "Continue Without Model" }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(
      screen.getByText(
        /including semantic-search downloads for the knowledge base and search api/i,
      ),
    ).toBeTruthy();
  });
});
