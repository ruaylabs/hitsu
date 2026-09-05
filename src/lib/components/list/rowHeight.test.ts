import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/** The virtualized list positions rows by arithmetic on a JS constant while
 *  the rows themselves are sized by CSS. Nothing at runtime notices when the
 *  two disagree — rows just drift out of their slots — so pin them here. */
describe("list row height", () => {
  const read = (path: string) => readFileSync(new URL(path, import.meta.url), "utf8");

  it("keeps ROW_HEIGHT in sync with --row-lg", () => {
    const token = read("../../styles/tokens.css").match(/--row-lg:\s*(\d+)px/);
    const constant = read("./ItemList.svelte").match(/const ROW_HEIGHT = (\d+);/);

    expect(token?.[1]).toBeDefined();
    expect(constant?.[1]).toBe(token?.[1]);
  });

  it("sizes .list-row from the token rather than a literal", () => {
    expect(read("./ItemListRow.svelte")).toContain("height: var(--row-lg);");
  });
});
