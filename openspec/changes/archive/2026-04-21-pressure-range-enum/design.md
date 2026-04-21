## Context

`AbpConfig` currently holds `p_max: f32`, `p_min: f32`, and `unit: PressureUnit` as separate fields parsed from positions 7–11 of the Honeywell part number string. The parser decodes position 7–9 as a float, position 10 as a unit letter (M/B/K/P), and position 11 as type (D/G). This approach:

- Accepts arbitrary `(p_max, unit)` combinations that do not correspond to any real sensor (e.g. `p_max=4.0, unit=Mbar` → 4 mbar, below the 60 mbar minimum)
- Silently rejects `001GG` (0–1 MPa gauge) because `"G"` is not a recognised unit letter
- Requires users to consult the datasheet to know which combinations are valid when constructing `AbpConfig` directly
- Decodes decimal values (`"1.6"`, `"2.5"`) by luck — `f32::from_str` happens to handle them

The driver is `no_std` with no heap allocation. All changes must preserve that constraint.

## Goals / Non-Goals

**Goals:**
- Make all 57 valid Honeywell ABP pressure+type combinations directly expressible and all others statically or dynamically unrepresentable
- Fix the `001GG` (MPa gauge) rejection bug
- Keep `from_part_number` as the single authoritative parser; no runtime cost increase
- Preserve `no_std` — no allocator, no `std` imports

**Non-Goals:**
- Supporting custom / non-standard pressure ranges (footnote 1 in datasheet: "contact Honeywell")
- Changing the I2C read or pressure conversion formula
- Adding `serde` or other serialisation

## Decisions

### Decision 1: One enum with one variant per (magnitude, type) combination

**Chosen:** A single `PressureRange` enum with 57 variants, one per valid Honeywell orderable code (e.g. `Mbar060Differential`, `Bar4Gauge`, `Mpa1Gauge`). The enum derives `Copy, Clone, Debug, PartialEq`.

**Alternatives considered:**

- *Separate `PressureMagnitude` + `SensorType` enums, combined in `AbpConfig`*: Simpler variant count (~28 magnitude + 2 type), but some combinations are gauge-only (`006BG`, `010BG`, `001GG`) — a separate-field design cannot prevent `Bar6Differential` from being constructed, which doesn't exist. Rejected: invalid states remain representable.

- *`(p_max: f32, unit: PressureUnit)` with a validation method*: No structural change; invalid values still exist at the type level. Rejected: does not achieve the goal.

**Variant naming convention:** `{Unit}{MagnitudeNoDot}{GaugeOrDifferential}` — e.g. `Bar1Gauge`, `Bar1_6Differential`, `Mpa1Gauge`, `Mbar060Gauge`. Underscores replace decimal points for Rust identifier validity.

### Decision 2: Match the full 5-character code in `from_part_number`

**Chosen:** `from_part_number` slices `&part_nr[7..12]` and matches it against all 57 literal strings in a single `match` block. Anything not in the match returns `Err(ParseError::InvalidPressureRange)`.

**Alternatives considered:**

- *Keep position-by-position parsing, add MPa case*: Requires special-casing `"G"` at position 10 (conflicts with `"G"` meaning gauge at position 11), and still cannot enforce the allowlist. Rejected: complexity without benefit.

### Decision 3: Collapse `InvalidUnit` + `InvalidType` into `InvalidPressureRange`

**Chosen:** Remove `ParseError::InvalidUnit` and `ParseError::InvalidType`; add `ParseError::InvalidPressureRange`. The 5-char code is now atomic — there is no meaningful distinction between "wrong unit" and "wrong type" when matching a combined code.

This is a breaking change to `ParseError`. Downstream exhaustive `match` expressions will get a compile error, which is the correct signal.

### Decision 4: `PressureRange` methods

`PressureRange` exposes:
- `p_min_pa(self) -> f32` — minimum pressure in Pascals (0.0 for gauge, −p_max_pa for differential)
- `p_max_pa(self) -> f32` — maximum pressure in Pascals
- `unit(self) -> PressureUnit` — for Display and native-unit formatting

`AbpConfig.convert_pressure()` reads `self.config.range.p_min_pa()` and `self.config.range.p_max_pa()` directly — no change to the formula, only the data source.

`PressureUnit` gains `Mpa` variant with `to_pa_factor() = 1_000_000.0` and `Display = "MPa"`.

## Risks / Trade-offs

- **57-variant enum is verbose** → It is generated once from the datasheet and never needs to change unless Honeywell adds new ranges. The match block in `from_part_number` serves as the complete allowlist and is self-documenting.
- **Breaking API change** → Any crate user constructing `AbpConfig` directly or matching `ParseError` gets a compile error. This is intentional and correct; they were constructing potentially invalid configs before.
- **`p_min`/`p_max` as f32 accuracy** → Decimal bar values (1.6, 2.5) cannot be represented exactly in f32, but this is the same approximation the current code uses and is well within sensor accuracy (±0.25% FS). No change in behaviour.

## Migration Plan

1. Add `PressureRange` enum and `PressureUnit::Mpa` variant in `src/lib.rs`
2. Update `AbpConfig`: replace `p_max`, `p_min`, `unit` with `range: PressureRange`
3. Update `ParseError`: remove `InvalidUnit`, `InvalidType`; add `InvalidPressureRange`
4. Rewrite `from_part_number` to use the 5-char match
5. Update `convert_pressure` to call `range.p_min_pa()` / `range.p_max_pa()`
6. Update all tests: direct `AbpConfig` constructors use `PressureRange` variants; `ParseError` matches update
7. `cargo build` → `cargo test` → `cargo clippy` — must all pass cleanly

No migration shim is needed; this is a library at pre-1.0 on a feature branch.

## Open Questions

- None. All 57 ranges are enumerated from Table 7 / Figure 3 of the datasheet.
