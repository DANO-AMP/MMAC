import { describe, it, expect } from "vitest";
import {
  formatSize,
  formatSpeed,
  formatPercentage,
  formatTemperature,
} from "../../utils/format";

describe("formatSize", () => {
  it("formats bytes correctly", () => {
    expect(formatSize(0)).toBe("0 B");
    expect(formatSize(500)).toBe("500 B");
    expect(formatSize(1024)).toBe("1 KB");
    expect(formatSize(1536)).toBe("1.5 KB");
    expect(formatSize(1048576)).toBe("1 MB");
    expect(formatSize(1073741824)).toBe("1 GB");
    expect(formatSize(1099511627776)).toBe("1 TB");
  });

  it("handles negative and invalid values", () => {
    expect(formatSize(-100)).toBe("0 B");
    expect(formatSize(NaN)).toBe("0 B");
    expect(formatSize(Infinity)).toBe("0 B");
  });
});

describe("formatSpeed", () => {
  it("formats network speeds correctly", () => {
    expect(formatSpeed(0)).toBe("0 B/s");
    expect(formatSpeed(1024)).toBe("1 KB/s");
    expect(formatSpeed(1048576)).toBe("1 MB/s");
  });

  it("handles invalid values", () => {
    expect(formatSpeed(-100)).toBe("0 B/s");
    expect(formatSpeed(NaN)).toBe("0 B/s");
  });
});

describe("formatPercentage", () => {
  it("formats percentages correctly", () => {
    expect(formatPercentage(0)).toBe("0.0%");
    expect(formatPercentage(50)).toBe("50.0%");
    expect(formatPercentage(100)).toBe("100.0%");
    expect(formatPercentage(75.5)).toBe("75.5%");
  });

  it("respects decimal places parameter", () => {
    expect(formatPercentage(75.555, 2)).toBe("75.56%");
    expect(formatPercentage(50, 0)).toBe("50%");
  });

  it("handles invalid values", () => {
    expect(formatPercentage(Infinity)).toBe("0%");
    expect(formatPercentage(NaN)).toBe("0%");
  });
});

describe("formatTemperature", () => {
  it("formats temperature correctly", () => {
    expect(formatTemperature(25.0)).toBe("25.0°C");
    expect(formatTemperature(0)).toBe("0.0°C");
    expect(formatTemperature(-10.5)).toBe("-10.5°C");
  });

  it("handles invalid values", () => {
    expect(formatTemperature(Infinity)).toBe("N/A");
    expect(formatTemperature(NaN)).toBe("N/A");
  });
});
