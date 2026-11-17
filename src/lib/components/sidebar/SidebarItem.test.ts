import { readFileSync } from "node:fs";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import SidebarItem from "./SidebarItem.svelte";

const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");
const source = read("./SidebarItem.svelte");
const rule = (selector: string) => {
  const start = source.indexOf(`  ${selector} {`);
  expect(start).toBeGreaterThan(-1);
  return source.slice(start, source.indexOf("}", start));
};

describe("SidebarItem action spacing", () => {
  it("reserves action space independently of hover and keyboard focus", () => {
    expect(rule(".sidebar-row")).toContain("display: flex");
    expect(rule(".sidebar-item")).toContain("min-width: 0");
    expect(rule(".sidebar-item")).toContain("flex: 1");
    expect(rule(".sidebar-actions")).toContain("flex-shrink: 0");
    expect(rule(".sidebar-actions")).not.toMatch(/position: absolute|display: none/);
    expect(rule(".sidebar-count")).toContain("flex-shrink: 0");
    expect(source).not.toContain(".has-actions:hover .sidebar-count");
    expect(source).toContain(".sidebar-row:hover .sidebar-actions,");
    expect(source).toContain(".sidebar-row:focus-within .sidebar-actions");
    expect(rule(".sidebar-label")).toContain("text-overflow: ellipsis");
  });

  it("keeps a nested long-label row's count and all actions available", async () => {
    const label = "A very long nested folder name that must truncate before the count";
    const onclick = vi.fn();
    const onadd = vi.fn();
    const onedit = vi.fn();
    const ondelete = vi.fn();
    render(SidebarItem, {
      label,
      count: 123456,
      indent: 3,
      onclick,
      onadd,
      onedit,
      ondelete,
    });

    const row = screen.getByRole("button", { name: `${label} 123456` });
    expect(row.style.paddingLeft).toContain("48px");
    await fireEvent.mouseOver(row);
    expect(screen.getByText("123456")).toBeVisible();
    const add = screen.getByRole("button", { name: `Add folder inside ${label}` });
    add.focus();
    expect(add).toHaveFocus();
    expect(screen.getByText("123456")).toBeVisible();
    await fireEvent.click(add);
    await fireEvent.click(screen.getByRole("button", { name: `Rename ${label}` }));
    await fireEvent.click(screen.getByRole("button", { name: `Delete ${label}` }));
    expect(onadd).toHaveBeenCalledOnce();
    expect(onedit).toHaveBeenCalledOnce();
    expect(ondelete).toHaveBeenCalledOnce();
    expect(onclick).not.toHaveBeenCalled();
    await fireEvent.click(row);
    expect(onclick).toHaveBeenCalledOnce();
  });

  it("does not reserve an action group for rows without actions", () => {
    const { container } = render(SidebarItem, { label: "All items", count: 123 });
    expect(container.querySelector(".sidebar-actions")).toBeNull();
    expect(screen.getByRole("button", { name: "All items 123" })).toBeVisible();
  });
});
