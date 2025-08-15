# RSLife

A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics, featuring an elegant **builder pattern** that makes complex actuarial calculations intuitive and type-safe.

[![Crates.io](https://img.shields.io/crates/v/rslife.svg)](https://crates.io/crates/rslife)
[![Documentation](https://docs.rs/rslife/badge.svg)](https://docs.rs/rslife)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Why RSLife?

**🚀 Performance & Memory Efficiency:**

- Built on Rust's zero-cost abstractions for maximum performance
- Polars integration for efficient DataFrame operations with zero-copy optimization
- Minimal memory allocation with smart data reuse and lazy evaluation
- Compile-time optimizations eliminate runtime overhead

**🎯 Developer Experience:**

- **Intuitive Builder Pattern**: Only specify parameters you need, no confusing parameter lists. RSLife ensures that the low level interfaces are even more approachable than the high level ones.
- **Type Safety**: Compile-time validation prevents common actuarial calculation errors
- **Auto-Completion**: IDEs provide intelligent suggestions for all parameters
- **Self-Documenting**: Parameter names make code intent crystal clear
- **Cross-Field Validation**: Parameter combinations validated automatically

**📊 Intelligent Data Processing:**

- **Universal Input**: DataFrames, XLSX/ODS files, and loading directly from Society of Actuary (US) and Institute and Faculty of Actuaries (UK) Mortality Database with automatic format detection
- **Format Agnostic**: Seamlessly detects `qx` rates or `lx` survivor functions without manual specification
- **Smart Table Recognition**: Automatically determines ultimate vs select mortality tables
- **Validation Built-In**: Comprehensive data integrity checks prevent runtime errors before calculations
- **Select & Ultimate**: Full support for both table types with automatic recognition

**🔧 Production Ready:**

- **Complete Actuarial Coverage**: Life insurance, annuities,survival functions and commutations with standard notation
- **Multiple Assumptions**: Uniform Death Distribution (UDD), Constant Force of Mortality (CFM), and Hyperbolic (HPB) methods for fractional age calculations
- **Multiple Parametric Life Table Models**: Constant Force Law, Gompertz, and Makeham, Weibull etc...
- **Consistent API**: All functions use the same parameter structure with builder pattern
- **Battle-Tested**: Validated against standard actuarial references from SOA and IFOA most trusted materials.
- **Error Handling**: Clear, actionable error messages for debugging

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rslife = "0.2.5"
```

### The Builder Pattern Advantage

- **🎯 Intentional**: Only specify parameters that matter for each calculation
- **🔒 Safe**: Compile-time validation prevents parameter mistakes
- **📖 Readable**: Self-documenting code that's easy to understand
- **🔧 Maintainable**: Adding new parameters doesn't break existing code
- **⚡ Efficient**: Automatic cross-field validation catches errors early

```rust
use rslife::prelude::*;

fn main() -> RSLifeResult<()> {
    // Load mortality data from SOA database (Society of Actuaries)
    let soa_data = MortData::from_soa_url_id(1704)?;

    // Load mortality data from IFOA database (Institute and Faculty of Actuaries)
    let ifoa_data = MortData::from_ifoa_url_id("AM92")?;

    // Most commonly known parametric life table model - This is in fact SULT from SOA
    // Note: There are more direct method to call SULT built-in the crate though
    let makeham_data = MortData:::from_Makeham_law()
      .A(0.00022)
      .B(2.7e-6)
      .C(1.124)
      .start_age(20)
      .call()?;

    // Construct Mortality Table Config
    let mt_builder = MortTableConfig::builder()
        .data(ifoa_data)
        .radix(100_000) // Radix of 100k instead of default 10k
        .pct(1.5) // 150% mortality rate instead of default 100%
        .assumption(AssumptionEnum::CFM) // CFM assumption instead of default UDD assumtpion
        .build()?;

    // New builder pattern for actuarial calculations!
    let fractional_age_time_survival_rate = tpx()
        .mt(&mt)
        .x(35.5)
        .t(5.8)
        .call()?;

    let life_annuity = aax()
        .mt(&mt)
        .i(0.03)
        .x(65)
        .m(12) // monthly payable m=12
        .call()?;

    let deferred_term = Ax1n()
        .mt(&mt_builder)
        .i(0.03)
        .x(35)
        .n(15)
        .t(5) // Deferred 5 years
        .call()?;

    Ok(())
}
```

**vs. Traditional Approaches:**

```rust
// ❌ Other libraries: **verbose** structs, need to declare all parameters, easy to mess up order
let params = ComplexConfig {
    mt: config,
    i: 0.03,
    x: 35,
    n: None,
    t: 10,
    m: 1,
    moment: 1,
    entry_age: None,
};

// ❌ What does this even mean? Not intuitive but a common practise
let result = some_function(&config, 35, 0.03, 1, 0, 1, 1, Some(30))?;

// ✅ RSLife: crystal clear, only specify what matters
let result = Ax()
    .mt(&config)
    .i(0.03)
    .x(35)
    .entry_age(34)
    .call()?;
```

### Custom Data Sources

RSLife supports flexible mortality data input with automatic `qx`/`lx` detection.

Beside several well-known parametric life table (Constant Force, Gomprtz, Makeham, Weibull, etc ...), users can even load the data directly from most trusted mortality database or use their own custom data under various methods.

Details guide can be found on project [wiki](https://github.com/hnlearndev/rslife/wiki)

```rust
use polars::prelude::*;
use rslife::prelude::*;

// Parametric life table model
  let makeham_model_data = MortData:::from_Makeham_law()
    .A(0.00022)
    .B(2.7e-6)
    .C(1.124)
    .start_age(20)
    .call()?;

// DataFrames - mortality rates or survivor functions
let df_qx = df! {
    "age" => [25_u32, 26, 27],
    "qx" => [0.001_f64, 0.0012, 0.0015],
}?;

let df_lx = df! {
    "age" => [25_u32, 26.0, 27.0],
    "lx" => [100000.0_f64, 99900.0, 99780.0],
}?;

// Load data from various sources
// Custom data from dataframe
let data_from_df_with_qx = MortData::from_df(df_qx)?;
let data_from_df_with_lx = MortData::from_df(df_lx)?;

// Custom data from spreadsheet XLSX
let data_from_xlsx = MortData::from_xlsx("data/mortality.xlsx", "select")?;

// Custom data from spreadsheet ODS
let data_from_ods = MortData::from_ods("data/mortality.ods", "select")?;

// ELT No.15 Female
let data_from_soa = MortData::from_soa_url_id(1704)?;

// AM92 Selected Mortality Table
let data_from_ifoa = MortData::from_ifoa_url_id("AM92")?;
```

## Actuarial Functions & Naming Convention

### Function Structure

**Systematic Modifiers**:

- **Immediate**: Single letter → `Ax`, `Axn` (payments at end of year)
- **Due**: Double letter → `aax`, `aaxn` (payments at start of year)
- **Increasing**: `I` prefix → `IAx`, `Iaax` (arithmetic growth)
- **Decreasing**: `D` prefix → `DAx1n`, `Daaxn` (arithmetic decrease)
- **Geometric**: `g` prefix → `gAx`, `gaax` (geometric growth)

These modifiers are applicable to most but not all functions. (eg: There is no modified version for Exn/Axn1 - pure endowment function)

All functions now use the builder pattern with `SingleLifeParams` and `SurvivalFunctionParams` for consistent parameter passing and automatic validation.

### Full list of actuarial functions available via `rslife::prelude::*`

**Cetain annuities:**

Present value and future value

- `aan`, `an`, `ssn`,`sn`
- `Iaan`, `Ian`, `Issn`, `Isn`,
- `Daan`, `Dan`, `Dssn`, `Dsn`,

**Annuities:**

Due/In-advance version:

- `aax`, `aaxn`
- `Iaax`, `Iaaxn`
- `Daaxn`
- `gaax`, `gaaxn`

Immediate/In-arrears version:

- `ax`, `axn`
- `Iax`, `Iaxn`
- `Daxn`
- `gax`, `gaxn`

**Benefits and Life Insurance:**

- `Ax`, `Ax1n`, `Exn` or `Axn1`, `Axn`
- `IAx`, `IAx1n`, `IAxn`
- `DAx1n`, `DAxn`
- `gAx`, `gAx1n`, `gExn`, `gAxn`

**Survival Probabilities:**

- `tpx`, `tqx`

**Commutation:**

- `Cx`, `Dx`, `Mx`, `Nx`, `Rx`, `Sx`

All functions are developed following Test-Driven Development principles, using the most trusted reference materials from SOA and IFOA.

The package is also routinely re-tested by solving the latest actuarial examination problems.

## Examples

Check out the `examples/` directory for comprehensive examples:

- **`basic_usage.rs`** - Demonstrates basic usage of the package.
- **`cm1_april_2025.rs`** - Using RSLife package to provide solution for [CM1 exam from IFOA](https://actuaries.org.uk/qualify/curriculum/actuarial-mathematics/).

These examples will be updated when CM1  papers and examiners' report are published.

SOA examination materials are also under consideration to be added as a re-testing medium in the near future.

### Running Examples

```bash
# Basic usage example
cargo run --example basic_usage

# April 2025 CM1 exam solution using RSLife
cargo run --example cm1_april_2025
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

**Willian Nguyen** - [hieunt(dot)hello(at)gmail(dot)com](mailto:)

**Project Link** - [https://github.com/hnlearndev/rslife](https://github.com/hnlearndev/rslife)

## References

- [Actuarial Mathematics for Life Contingent Risks](https://www.goodreads.com/book/show/58306503-actuarial-mathematics-for-life-contingent-risks)
- [Actuarial Mathematics](https://www.goodreads.com/book/show/1715653.Actuarial_Mathematics)
- [Society of Actuaries Mortality and Morbidity Tables](https://mort.soa.org)
- [Institute and Faculty of Actuaries Mortality and Morbidity Tables](https://www.actuaries.org.uk/learn-and-develop/continuous-mortality-investigation/cmi-mortality-and-morbidity-tables)
- Standard actuarial notation and practices

### Similar Projects

**Python:**

- [pyliferisk](https://github.com/franciscogarate/pyliferisk) - Python library for actuarial calculations and life insurance mathematics
- [pymort](https://github.com/actuarialopensource/pymort) - Python mortality table library with XML parsing capabilities

**R:**

- [lifecontingencies](https://github.com/spedygiorgio/lifecontingencies) - R package for actuarial life contingencies calculations
- [MortalityTables](https://github.com/kainhofer/r-mortality-tables) - R package for working with life and pension tables
- [demography](https://github.com/robjhyndman/demography) - R package for demographic analysis and mortality forecasting

**Julia:**

- [MortalityTables.jl](https://github.com/JuliaActuary/MortalityTables.jl) - Julia package for mortality table calculations and life contingencies
- [ActuaryUtilities.jl](https://github.com/JuliaActuary/ActuaryUtilities.jl) - Julia utilities for actuarial modeling and analysisk.
