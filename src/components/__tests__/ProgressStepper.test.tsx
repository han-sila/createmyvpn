import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import ProgressStepper from "../../components/ProgressStepper";
import type { ProgressEvent } from "../../lib/types";

describe("ProgressStepper", () => {
  const makeSteps = (count: number, errorAt?: number): ProgressEvent[] =>
    Array.from({ length: count }, (_, i) => ({
      step: i + 1,
      total_steps: count,
      message: `Step ${i + 1}`,
      status: i + 1 === errorAt ? "error" : i + 1 < count ? "done" : "running",
    }));

  it("renders all steps", () => {
    const steps = makeSteps(5);
    render(<ProgressStepper steps={steps} currentStep={3} />);
    expect(screen.getByText("Step 1")).toBeInTheDocument();
    expect(screen.getByText("Step 2")).toBeInTheDocument();
    expect(screen.getByText("Step 3")).toBeInTheDocument();
    expect(screen.getByText("Step 4")).toBeInTheDocument();
    expect(screen.getByText("Step 5")).toBeInTheDocument();
  });

  it("shows step messages", () => {
    const steps: ProgressEvent[] = [
      { step: 1, total_steps: 3, message: "Creating VPC", status: "done" },
      { step: 2, total_steps: 3, message: "Launching instance", status: "running" },
      { step: 3, total_steps: 3, message: "Configuring WireGuard", status: "running" },
    ];
    render(<ProgressStepper steps={steps} currentStep={2} />);
    expect(screen.getByText("Creating VPC")).toBeInTheDocument();
    expect(screen.getByText("Launching instance")).toBeInTheDocument();
    expect(screen.getByText("Configuring WireGuard")).toBeInTheDocument();
  });

  it("renders error step with error styling", () => {
    const steps = makeSteps(3, 2);
    const { container } = render(<ProgressStepper steps={steps} currentStep={2} />);
    const errorElements = container.querySelectorAll(".text-red-400");
    expect(errorElements.length).toBeGreaterThan(0);
  });

  it("renders completed step with green styling", () => {
    const steps = makeSteps(3);
    const { container } = render(<ProgressStepper steps={steps} currentStep={3} />);
    const greenElements = container.querySelectorAll(".text-green-400");
    expect(greenElements.length).toBeGreaterThan(0);
  });

  it("renders empty steps list", () => {
    const { container } = render(<ProgressStepper steps={[]} currentStep={0} />);
    expect(container.firstChild).toBeInTheDocument();
  });
});
