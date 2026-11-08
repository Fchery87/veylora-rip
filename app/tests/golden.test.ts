import { describe, expect, it } from "vite-plus/test";
import goldenJson from "../../crates/veylora-app/tests/golden/presenters.json";
import {
  inspectView,
  jobActionsView,
  preflightView,
  reviewView,
  setInspectComposite,
  toggleInspectSelection,
} from "../src/lib/services/presentation";
import type {
  CapabilityState,
  InspectSelection,
  JobDetail,
  Plane,
} from "../src/lib/services/types";

// Written by the Rust test `golden_presenters`. The Rust presenters are the
// authority; this suite proves the TypeScript fallback agrees with them.
interface Case {
  name: string;
  input: { caps: CapabilityState; job: JobDetail; selection: InspectSelection };
  expected: {
    preflight: unknown;
    actions: unknown;
    inspect: { ok: unknown } | { error: string };
    review: unknown;
  };
}
interface Toggle {
  name: string;
  input: { selection: InspectSelection; op: "plane" | "composite"; plane: Plane; on: boolean };
  expected: InspectSelection;
}

const golden = goldenJson as unknown as { cases: Case[]; toggles: Toggle[] };

// JSON has no undefined, so compare through a JSON round trip.
const plain = (value: unknown) => JSON.parse(JSON.stringify(value ?? null));

describe("TypeScript fallback presenters agree with the Rust golden fixture", () => {
  it("has cases to compare", () => {
    expect(golden.cases.length).toBeGreaterThan(10);
    expect(golden.toggles.length).toBeGreaterThan(3);
  });

  for (const c of golden.cases) {
    it(`matches Rust for: ${c.name}`, () => {
      const { caps, job, selection } = c.input;
      expect(plain(preflightView(job, caps))).toEqual(c.expected.preflight);
      expect(plain(jobActionsView(job, caps))).toEqual(c.expected.actions);
      expect(plain(reviewView(job, caps))).toEqual(c.expected.review);
      let inspect: { ok: unknown } | { error: string };
      try {
        inspect = { ok: inspectView(job, caps, selection) };
      } catch (e) {
        inspect = { error: e instanceof Error ? e.message : String(e) };
      }
      expect(plain(inspect)).toEqual(c.expected.inspect);
    });
  }

  for (const t of golden.toggles) {
    it(`selects like Rust for: ${t.name}`, () => {
      const { selection, op, plane, on } = t.input;
      const next =
        op === "plane"
          ? toggleInspectSelection(selection, plane, on)
          : setInspectComposite(selection, on);
      expect(plain(next)).toEqual(t.expected);
    });
  }
});
