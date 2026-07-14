import { describe, expect, it } from "vitest";

import {
  formatDateRange,
  formatDayLabel,
  formatMinutes,
  nonEmpty,
} from "./display";

describe("display helpers", () => {
  it("formats weekday and short date in UTC", () => {
    expect(formatDayLabel("2026-04-26")).toBe("Sun · Apr 26");
    expect(formatDayLabel("2026-04-27")).toBe("Mon · Apr 27");
  });

  it("returns original string for invalid dates", () => {
    expect(formatDayLabel("not-a-date")).toBe("not-a-date");
  });

  it("formats date ranges and omits empty", () => {
    expect(formatDateRange("2026-01-01", "2026-01-03")).toBe(
      "2026-01-01 — 2026-01-03",
    );
    expect(formatDateRange(null, null)).toBeNull();
  });

  it("formats minutes and trims empty strings", () => {
    expect(formatMinutes(45)).toBe("45 min");
    expect(formatMinutes(null)).toBeNull();
    expect(nonEmpty("  Naha  ")).toBe("Naha");
    expect(nonEmpty("   ")).toBeNull();
  });
});
