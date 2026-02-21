import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import RegionSelector from "../../components/RegionSelector";
import { AWS_REGIONS } from "../../lib/types";

describe("RegionSelector", () => {
  it("renders a select element", () => {
    const onChange = vi.fn();
    render(<RegionSelector value="us-east-1" onChange={onChange} />);
    const select = screen.getByRole("combobox");
    expect(select).toBeInTheDocument();
  });

  it("renders all AWS regions as options", () => {
    const onChange = vi.fn();
    render(<RegionSelector value="us-east-1" onChange={onChange} />);
    const options = screen.getAllByRole("option");
    expect(options.length).toBe(AWS_REGIONS.length);
  });

  it("has correct initial value", () => {
    const onChange = vi.fn();
    render(<RegionSelector value="eu-west-1" onChange={onChange} />);
    const select = screen.getByRole("combobox") as HTMLSelectElement;
    expect(select.value).toBe("eu-west-1");
  });

  it("calls onChange when selection changes", () => {
    const onChange = vi.fn();
    render(<RegionSelector value="us-east-1" onChange={onChange} />);
    const select = screen.getByRole("combobox");
    fireEvent.change(select, { target: { value: "us-west-2" } });
    expect(onChange).toHaveBeenCalledWith("us-west-2");
  });

  it("can be disabled", () => {
    const onChange = vi.fn();
    render(<RegionSelector value="us-east-1" onChange={onChange} disabled />);
    const select = screen.getByRole("combobox") as HTMLSelectElement;
    expect(select.disabled).toBe(true);
  });

  it("each option shows region name and code", () => {
    const onChange = vi.fn();
    render(<RegionSelector value="us-east-1" onChange={onChange} />);
    const option = screen.getByText("US East (N. Virginia) (us-east-1)");
    expect(option).toBeInTheDocument();
  });
});
