## 1. Add PressureUnit::Mpa variant

- [x] 1.1 Add `Mpa` variant to `PressureUnit` enum
- [x] 1.2 Add `Mpa => 1_000_000.0` arm to `to_pa_factor()`
- [x] 1.3 Add `Mpa => "MPa"` arm to `Display` impl
- [x] 1.4 Add unit test: `pressure_unit_mpa_factor` — asserts `to_pa_factor()` equals `1_000_000.0`

## 2. Add PressureRange enum

- [x] 2.1 Define `PressureRange` enum with all 56 variants (mbar/bar/kPa/MPa/psi × gauge/differential as applicable) — derive `Copy, Clone, Debug, PartialEq`
- [x] 2.2 Implement `p_max_pa(self) -> f32` — return p_max in Pascals for each variant
- [x] 2.3 Implement `p_min_pa(self) -> f32` — return `0.0` for gauge variants, `-p_max_pa()` for differential variants
- [x] 2.4 Implement `unit(self) -> PressureUnit` — return the correct unit for each variant
- [x] 2.5 Add doc comments to `PressureRange` and its three methods

## 3. Update ParseError

- [x] 3.1 Remove `InvalidUnit` and `InvalidType` variants from `ParseError`
- [x] 3.2 Add `InvalidPressureRange` variant to `ParseError` with doc comment explaining it covers unknown or non-orderable pressure+type codes

## 4. Rewrite AbpConfig

- [x] 4.1 Remove `p_max: f32`, `p_min: f32`, `unit: PressureUnit` fields from `AbpConfig`
- [x] 4.2 Add `range: PressureRange` field to `AbpConfig`
- [x] 4.3 Update `AbpConfig` doc comment and example to reflect new struct layout
- [x] 4.4 Rewrite `from_part_number`: replace position-by-position parsing of unit+type with a single `match &part_nr[7..12]` block covering all 56 valid codes → `PressureRange` variants; unknown codes return `Err(ParseError::InvalidPressureRange)`
- [x] 4.5 Update `from_part_number` doc comment: update part number layout table, error table (remove `InvalidUnit`/`InvalidType`, add `InvalidPressureRange`), and examples

## 5. Update convert_pressure

- [x] 5.1 Update `convert_pressure` to call `self.config.range.p_min_pa()` and `self.config.range.p_max_pa()` instead of reading `p_min`, `p_max`, `unit` fields directly

## 6. Update tests

- [x] 6.1 Update `gauge_psi_config()` helper to use `range: PressureRange::Psi150Gauge`
- [x] 6.2 Update `pressure_native()` helper to derive p_max/p_min from `config.range`
- [x] 6.3 Update `from_part_number_valid_gauge_psi` test: assert `config.range == PressureRange::Psi150Gauge` (adjust part number if needed for 150 psi)
- [x] 6.4 Update `from_part_number_valid_differential_kpa_thermometer` test: assert `config.range == PressureRange::Kpa100Differential`
- [x] 6.5 Replace `from_part_number_invalid_unit` test with `from_part_number_invalid_pressure_range` — use a syntactically valid but non-existent code (e.g. `"ABPDNNN004MG2A3"` → `Err(InvalidPressureRange)`)
- [x] 6.6 Replace `from_part_number_invalid_type` test with a case covered by `InvalidPressureRange` (e.g. `"ABPDNNN010BD2A3"` — bar 10 differential does not exist)
- [x] 6.7 Add `from_part_number_mpa_gauge` test: `"ABPDNNN001GG2A3"` → `Ok(config)` with `range == Mpa1Gauge`
- [x] 6.8 Add `from_part_number_decimal_bar` test: `"ABPDNNN1.6BD2A3"` → `Ok(config)` with `range == Bar1_6Differential`
- [x] 6.9 Update `convert_pressure_differential_at_o_min` test: construct config with `PressureRange::Kpa100Differential`
- [x] 6.10 Verify all existing pressure formula tests still pass unchanged (formula is unaffected)

## 7. Verify

- [x] 7.1 `cargo build` — must compile with no errors
- [x] 7.2 `cargo test` — all tests pass
- [x] 7.3 `cargo clippy` — no warnings
