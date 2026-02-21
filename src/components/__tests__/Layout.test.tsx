import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import Layout from "../Layout";

// Mock logo import
vi.mock("../../assets/logo.png", () => ({ default: "logo.png" }));

function renderLayout(initialRoute = "/dashboard") {
  return render(
    <MemoryRouter initialEntries={[initialRoute]}>
      <Layout />
    </MemoryRouter>,
  );
}

describe("Layout", () => {
  it("renders the app title", () => {
    renderLayout();
    expect(screen.getByText("CreateMyVPN")).toBeInTheDocument();
  });

  it("renders logo image", () => {
    renderLayout();
    const logo = screen.getByAltText("CreateMyVPN");
    expect(logo).toBeInTheDocument();
    expect(logo).toHaveAttribute("src", "logo.png");
  });

  it("shows version number", () => {
    renderLayout();
    expect(screen.getByText("v0.1.0")).toBeInTheDocument();
  });

  it("renders all 5 navigation links", () => {
    renderLayout();
    expect(screen.getByText("Dashboard")).toBeInTheDocument();
    expect(screen.getByText("Setup")).toBeInTheDocument();
    expect(screen.getByText("Deploy")).toBeInTheDocument();
    expect(screen.getByText("Settings")).toBeInTheDocument();
    expect(screen.getByText("Logs")).toBeInTheDocument();
  });

  it("navigation links have correct hrefs", () => {
    renderLayout();
    const links = screen.getAllByRole("link");
    const hrefs = links.map((l) => l.getAttribute("href"));
    expect(hrefs).toContain("/dashboard");
    expect(hrefs).toContain("/setup");
    expect(hrefs).toContain("/deploy");
    expect(hrefs).toContain("/settings");
    expect(hrefs).toContain("/logs");
  });

  it("highlights active nav item", () => {
    renderLayout("/settings");
    const settingsLink = screen.getByText("Settings").closest("a");
    // Active class should contain the primary color class
    expect(settingsLink?.className).toContain("primary");
  });

  it("renders main content outlet area", () => {
    renderLayout();
    // The main element should exist (it wraps the Outlet)
    const main = document.querySelector("main");
    expect(main).toBeInTheDocument();
  });

  it("has sidebar with correct width class", () => {
    renderLayout();
    const aside = document.querySelector("aside");
    expect(aside).toBeInTheDocument();
    expect(aside?.className).toContain("w-56");
  });
});
