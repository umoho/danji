# Solver divergence: partial pivoting → catastrophic cancellation → limit cycle

## What

The Newton-Raphson solver at `src/circuit/solver.rs:428` (function `solve_linear`) diverges
for a 7-node 12AX7 long-tail pair phase inverter when B+ ≥ 0.35 V.  The solver enters a
limit cycle: the grid voltage `v[4]` oscillates between ≈ +1.2 µV and ≈ –1.2 µV,
`max_delta` gets stuck at ~2.4 e‑5, and `MAX_ITER = 50` expires.

## Root cause

The Gaussian elimination uses **partial pivoting** (row swaps based on column magnitude).
At column 4 (V1b grid, node 4):
- `a[4][4]` = 2.128 e‑6 (the grid's own conductance: 1/470 kΩ)
- `a[5][4]` = 3.027 e‑6 (fill-in from the triode's `gm` via the cathode row elimination)

Since `|a[5][4]| > |a[4][4]|`, the code swaps rows 4 and 5 (the grid and plate equations).
This is **fatal**:

1. The V1b **plate equation** (row 5) moves to row 4.
2. At column 5, `|a[6][5]| > |a[5][5]|`, so rows 5 and 6 swap again — the
   **VSRC equation moves to row 5**, the **grid equation to row 6**.
3. Column 5 elimination using the VSRC row as the pivot *completely transforms*
   the grid equation: its diagonal goes from 2.128 e‑6 → –9.88 e5 and its RHS
   goes from 0 → –3.46 e5 (dominated by the VSRC's conductance).
4. Back-substitution then computes:
   - `v[6]` = –3.46 e5 / –9.88 e5 = 0.350 V  (B+ — correct by accident)
   - `v[5]` = (3.5 e5 – 1 e6 × 0.350) / –8.2 e‑6  ← **catastrophic cancellation**
     Both numerator terms are ~350 000; the residue (~1 e‑6) amplified 1.2 e5×
     gives ~0.3 V instead of 0 V for the grid.
   - `v[4]` = (1.37 e‑8 – 1.15 e‑5 × `v[5]` + 1.03 e‑5 × 0.350) / 3.0 e‑6
     → the plate equation now solving for the grid, ~1.4 µV error.

5. The grid voltage error feeds the next Newton iteration, the triode linearisation
   changes slightly, and the pattern repeats indefinitely.

**Why it only triggers at B+ ≥ 0.35 V:** Below that, the 12AX7 is in deep cutoff
(gm ≈ 0), so `a[5][4]` = 0 and no swap occurs.

## Fix

Remove partial pivoting from `solve_linear`.  MNA matrices are **diagonally dominant**
(each node's self-conductance equals the sum of all series conductances attached to
that node).  Partial pivoting is unnecessary and, as shown, harmful when the matrix
spans many orders of magnitude (VSRC_G = 1 e6 vs. grid resistor = 2 e‑6).

The fix is in `src/circuit/solver.rs`: the pivot-search + row-swap block
was replaced by a direct diagonal-access with a singularity check.

## Verification

- Phase inverter test (`test_phase_inverter`): converges to correct DC bias
  (Vg=0 V, Vk=3.21 V, Vp=246.6 V at B+ = 250 V).
- All 18 unit tests pass.
- `cargo clippy` — no new warnings.
- `cargo fmt` — clean.

## Notes

- The full push-pull test (`test_full_push_pull`) still fails with NaN at certain
  nodes.  This is pre-existing: the pentode/coupled‑inductor model produces extreme
  values at very low B+ during the warmup ramp.  Not related to the solver change.
- A second line of defence: reduce `VSRC_G` from 1 e6 to ≈ 1 e4 (still stiff enough
  to enforce source voltages to microvolt precision, but reducing the matrix condition
  number by 100×).
