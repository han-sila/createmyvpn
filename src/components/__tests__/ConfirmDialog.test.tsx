import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import ConfirmDialog from "../../components/ConfirmDialog";

describe("ConfirmDialog", () => {
  const defaultProps = {
    isOpen: true,
    title: "Delete Server",
    message: "This will destroy all resources.",
    onConfirm: vi.fn(),
    onCancel: vi.fn(),
  };

  it("renders nothing when isOpen is false", () => {
    const { container } = render(
      <ConfirmDialog {...defaultProps} isOpen={false} />
    );
    expect(container.innerHTML).toBe("");
  });

  it("renders dialog when isOpen is true", () => {
    render(<ConfirmDialog {...defaultProps} />);
    expect(screen.getByText("Delete Server")).toBeInTheDocument();
    expect(screen.getByText("This will destroy all resources.")).toBeInTheDocument();
  });

  it("renders default confirm label", () => {
    render(<ConfirmDialog {...defaultProps} />);
    expect(screen.getByText("Confirm")).toBeInTheDocument();
  });

  it("renders custom confirm label", () => {
    render(<ConfirmDialog {...defaultProps} confirmLabel="Yes, Destroy" />);
    expect(screen.getByText("Yes, Destroy")).toBeInTheDocument();
  });

  it("renders Cancel button", () => {
    render(<ConfirmDialog {...defaultProps} />);
    expect(screen.getByText("Cancel")).toBeInTheDocument();
  });

  it("calls onConfirm when confirm button clicked", () => {
    const onConfirm = vi.fn();
    render(<ConfirmDialog {...defaultProps} onConfirm={onConfirm} />);
    fireEvent.click(screen.getByText("Confirm"));
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it("calls onCancel when cancel button clicked", () => {
    const onCancel = vi.fn();
    render(<ConfirmDialog {...defaultProps} onCancel={onCancel} />);
    fireEvent.click(screen.getByText("Cancel"));
    expect(onCancel).toHaveBeenCalledOnce();
  });

  it("calls onCancel when X button clicked", () => {
    const onCancel = vi.fn();
    render(<ConfirmDialog {...defaultProps} onCancel={onCancel} />);
    // X button is the first button-like element with lucide X icon
    const buttons = screen.getAllByRole("button");
    // X close button is the one that's not Cancel or Confirm
    const closeButton = buttons.find(
      (btn) => !btn.textContent?.includes("Cancel") && !btn.textContent?.includes("Confirm")
    );
    expect(closeButton).toBeDefined();
    fireEvent.click(closeButton!);
    expect(onCancel).toHaveBeenCalledOnce();
  });
});
