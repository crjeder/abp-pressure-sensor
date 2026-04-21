## ADDED Requirements

### Requirement: PressureRange enumerates all valid Honeywell ABP orderable pressure ranges
The driver SHALL expose a `PressureRange` enum with exactly one variant per valid pressure+type combination listed in the Honeywell ABP Series datasheet (Figure 3 / Table 7). No variants outside the datasheet allowlist SHALL exist. The enum SHALL derive `Copy`, `Clone`, `Debug`, `PartialEq`.

Variant naming convention: `{Unit}{Magnitude}{GaugeOrDifferential}` where decimal points in magnitude are replaced by underscores (e.g. `Bar1_6Gauge`).

Valid variants (57 total):
- mbar differential: `Mbar060Differential`, `Mbar100Differential`, `Mbar160Differential`, `Mbar250Differential`, `Mbar400Differential`, `Mbar600Differential`
- mbar gauge: `Mbar060Gauge`, `Mbar100Gauge`, `Mbar160Gauge`, `Mbar250Gauge`, `Mbar400Gauge`, `Mbar600Gauge`
- bar differential: `Bar1Differential`, `Bar1_6Differential`, `Bar2_5Differential`, `Bar4Differential`
- bar gauge: `Bar1Gauge`, `Bar1_6Gauge`, `Bar2_5Gauge`, `Bar4Gauge`, `Bar6Gauge`, `Bar10Gauge`
- kPa differential: `Kpa6Differential`, `Kpa10Differential`, `Kpa16Differential`, `Kpa25Differential`, `Kpa40Differential`, `Kpa60Differential`, `Kpa100Differential`, `Kpa160Differential`, `Kpa250Differential`, `Kpa400Differential`
- kPa gauge: `Kpa6Gauge`, `Kpa10Gauge`, `Kpa16Gauge`, `Kpa25Gauge`, `Kpa40Gauge`, `Kpa60Gauge`, `Kpa100Gauge`, `Kpa160Gauge`, `Kpa250Gauge`, `Kpa400Gauge`, `Kpa600Gauge`
- MPa gauge: `Mpa1Gauge`
- psi differential: `Psi1Differential`, `Psi5Differential`, `Psi15Differential`, `Psi30Differential`, `Psi60Differential`
- psi gauge: `Psi1Gauge`, `Psi5Gauge`, `Psi15Gauge`, `Psi30Gauge`, `Psi60Gauge`, `Psi100Gauge`, `Psi150Gauge`

#### Scenario: Gauge variant has p_min of zero
- **WHEN** `p_min_pa()` is called on any `*Gauge` variant
- **THEN** it returns `0.0`

#### Scenario: Differential variant has symmetric p_min
- **WHEN** `p_min_pa()` is called on any `*Differential` variant
- **THEN** it returns the negation of `p_max_pa()` for that variant

#### Scenario: Gauge-only ranges have no differential counterpart
- **WHEN** a part number for `Bar6`, `Bar10`, `Kpa160`, `Kpa250`, `Kpa400`, `Kpa600`, or `Mpa1` is parsed
- **THEN** only the `Gauge` suffix variant exists in the enum (no `Differential` variant for these magnitudes in those units)

### Requirement: PressureRange exposes p_min_pa, p_max_pa, and unit methods
`PressureRange` SHALL expose `p_min_pa(self) -> f32`, `p_max_pa(self) -> f32`, and `unit(self) -> PressureUnit`, all taking `self` by value (Copy).

#### Scenario: p_max_pa returns correct value in Pascals for a mbar gauge range
- **WHEN** `Mbar060Gauge.p_max_pa()` is called
- **THEN** it returns `6_000.0` (60 mbar × 100 Pa/mbar)

#### Scenario: p_max_pa returns correct value in Pascals for a bar differential range
- **WHEN** `Bar4Differential.p_max_pa()` is called
- **THEN** it returns `400_000.0` (4 bar × 100_000 Pa/bar)

#### Scenario: p_max_pa returns correct value in Pascals for an MPa gauge range
- **WHEN** `Mpa1Gauge.p_max_pa()` is called
- **THEN** it returns `1_000_000.0` (1 MPa × 1_000_000 Pa/MPa)

#### Scenario: p_max_pa returns correct value in Pascals for a psi gauge range
- **WHEN** `Psi150Gauge.p_max_pa()` is called
- **THEN** it returns `150.0 × 6_894.757_3` within float tolerance

#### Scenario: unit returns the correct PressureUnit variant
- **WHEN** `unit()` is called on a kPa range
- **THEN** it returns `PressureUnit::Kpa`
