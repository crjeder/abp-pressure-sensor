## Why

The `AbpConfig` struct uses a free-form `(p_max: f32, unit: PressureUnit)` pair that accepts any numeric value, making invalid sensor configurations (e.g. 4 mbar, which is below the 60 mbar minimum and does not exist as a Honeywell part) silently constructable. Additionally, the `G` unit letter (MPa, used by the `001GG` range) is entirely absent, causing `from_part_number` to incorrectly reject a valid production sensor.

## What Changes

- **NEW** `PressureRange` enum with one variant per valid Honeywell ABP pressure range (57 variants covering mbar/bar/kPa/MPa/psi in gauge and differential types).
- **BREAKING** `AbpConfig` fields `p_max: f32`, `p_min: f32`, and `unit: PressureUnit` are replaced by a single `range: PressureRange` field. `PressureRange` exposes `p_min_pa()` and `p_max_pa()` methods.
- **FIX** `from_part_number` now matches the full 5-character pressure+type code (`"001GG"`, `"004BD"`, `"1.6BG"`, etc.) against the enum, adding MPa support and enforcing the exact Honeywell allowlist.
- **BREAKING** `ParseError::InvalidUnit` and `ParseError::InvalidType` are replaced by a single `ParseError::InvalidPressureRange` variant, since the 5-char code is now validated as an atomic unit.
- `PressureUnit` enum gains `Mpa` variant (needed by `PressureRange` for unit display and conversion).
- Direct `AbpConfig` construction now requires a valid `PressureRange` variant, making all invalid states unrepresentable.

## Capabilities

### New Capabilities

- `pressure-range`: The `PressureRange` enum — all valid Honeywell ABP pressure+type combinations, with methods for `p_min_pa()`, `p_max_pa()`, and `unit()`.

### Modified Capabilities

- `sensor-config`: `AbpConfig` struct changes (replaces `p_max`/`p_min`/`unit` with `range: PressureRange`); `from_part_number` validation logic changes (5-char code match instead of separate unit/type parsing; new `InvalidPressureRange` error replaces `InvalidUnit`+`InvalidType`); `ParseError` variants change.

## Impact

- `AbpConfig` — struct field layout changes (breaking for direct constructors and pattern matchers)
- `ParseError` — two variants removed, one added (breaking for exhaustive match users)
- `PressureUnit` — gains `Mpa` variant
- `convert_pressure()` — reads `range.p_min_pa()` / `range.p_max_pa()` instead of config fields
- All existing tests that construct `AbpConfig` directly or match `ParseError` variants need updating
- Public API is `no_std` with no new dependencies
