<h1 align="center">

<a href="https://crates.io/crates/rslife">
  <picture>
    <source srcset="https://raw.githubusercontent.com/hnlearndev/static/refs/heads/main/rslife/banner/banner_dark.svg" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/hnlearndev/static/refs/heads/main/rslife/banner/banner_light.svg" alt="RSLife logo">
  </picture>
</a>

</h1>

<div align="center">

[![crates.io Latest Release](https://img.shields.io/crates/v/rslife.svg)](https://crates.io/crates/rslife)
[![Documentation](https://docs.rs/rslife/badge.svg)](https://docs.rs/rslife/latest/rslife/)
[![codecov](https://img.shields.io/codecov/c/github/hnlearndev/rslife?logo=codecov&label=coverage)](https://app.codecov.io/gh/hnlearndev/rslife)
[![Performance](https://img.shields.io/endpoint?url=https://codspeed.io/badge.json?org=hnlearndev&repo=rslife&label=performance)](https://codspeed.io/hnlearndev/rslife)

![Crates.io Total Downloads](https://img.shields.io/crates/d/rslife?style=social)

| Rust Version | Build Status | Test Status |
|:------------:|:------------:|:-----------:|
| **Stable**   | [![Build Status - Stable](https://img.shields.io/github/actions/workflow/status/hnlearndev/rslife/rust-stable.yml?branch=main&label=build&style=flat-square)](https://github.com/hnlearndev/rslife/actions/workflows/rust-stable.yml) | [![Test Status - Stable](https://img.shields.io/github/actions/workflow/status/hnlearndev/rslife/rust-stable.yml?branch=main&label=tests&style=flat-square)](https://github.com/hnlearndev/rslife/actions/workflows/rust-stable.yml) |
| **1.89.0**   | [![Build Status - 1.89.0](https://img.shields.io/github/actions/workflow/status/hnlearndev/rslife/rust-1.89.0.yml?branch=main&label=build&style=flat-square)](https://github.com/hnlearndev/rslife/actions/workflows/rust-1.89.0.yml) | [![Test Status - 1.89.0](https://img.shields.io/github/actions/workflow/status/hnlearndev/rslife/rust-1.89.0.yml?branch=main&label=tests&style=flat-square)](https://github.com/hnlearndev/rslife/actions/workflows/rust-1.89.0.yml) |
| **Nightly**  | [![Build Status - Nightly](https://img.shields.io/github/actions/workflow/status/hnlearndev/rslife/rust-nightly.yml?branch=main&label=build&style=flat-square)](https://github.com/hnlearndev/rslife/actions/workflows/rust-nightly.yml) | [![Test Status - Nightly](https://img.shields.io/github/actions/workflow/status/hnlearndev/rslife/rust-nightly.yml?branch=main&label=tests&style=flat-square)](https://github.com/hnlearndev/rslife/actions/workflows/rust-nightly.yml) |

</div>

---

A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics, featuring an elegant **builder pattern** that makes complex actuarial calculations intuitive and type-safe.

## Why RSLife?

**🚀 Performance & Memory Efficiency:**

- Leveraging Rust's zero-cost abstractions for maximum performance
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

Add the crate dependency

```bash
cargo add rslife
```

Or add this to your `Cargo.toml`:

```toml
[dependencies]
rslife = "0.2.8"
```

The crate is designed with three main layers to make actuarial computations convenient (more on [architecture from Wiki](https://github.com/hnlearndev/rslife/wiki/Architecture)), as illustrated below:

```rust
use rslife::prelude::*;

fn main() -> RSLifeResult<()> {
    // ========= FIRST LAYER - MORTALILITY DATA LOAD=========
    // Load mortality data
    // This seperation layer consists of multiple methods with flexibility at user hand to formulate the mortality or morbidity data
    let data = MortData::from_ifoa_url_id("AM92")?;

    // ========= SECOND LAYER - MORTALILITY TABLE CONFIGURATION =========
    // Construct Mortality Table Config
    // This layer is more rigid but still allows some configuration to mortality table
    let mt = MortTableConfig::builder()
      .data(data)
      .radix(100_000) // Radix of 100k instead of default 10k
      .pct(1.5) // 150% mortality rate instead of default 100%
      .assumption(AssumptionEnum::CFM) // CFM assumption instead of default UDD assumtpion
      .build()?;

    // ========= THIRD LAYER - CALCULATIONS =========
    // New builder pattern for actuarial calculations!
    // This is the layer to perform calculation. Variables are only declared when needed - Consistent with actuarial notation principle.
    let fractional_age_time_survival_rate = tpx()
      .mt(&mt)
      .x(35.5)
      .t(5.8)
      .entry_age(33)
      .call()?;

    let life_annuity = aax()
      .mt(&mt)
      .i(0.03)
      .x(65)
      .m(12) // monthly payable m=12
      .call()?;

    let deferred_term = Ax1n()
      .mt(&mt)
      .i(0.03)
      .x(35)
      .n(15)
      .entry_age(34) // Entry age for selected effect - duration
      .t(5) // Deferred 5 years
      .call()?;

    Ok(())
}
```

## Data sources - Layer 1 in zoom

RSLife supports flexible mortality data input with automatic `qx`/`lx` detection.

Detail guide can be found on project [wiki](https://github.com/hnlearndev/rslife/wiki)

An example of parametric life table model

```rust
// Parametric life table model
let makeham_model_data = MortData::from_Makeham_law()
  .A(0.00022)
  .B(2.7e-6)
  .C(1.124)
  .start_age(20)
  .call()?;
```

Life table can also be formulated from dataframe

```rust
// DataFrames - mortality rates or survivor functions
// qx data
let df_qx = df! {
    "age" => [25_u32, 26, 27],
    "qx" => [0.001_f64, 0.0012, 0.0015],
}?;

let data_from_df_with_qx = MortData::from_df(df_qx)?;

// lx data
let df_lx = df! {
    "age" => [25_u32, 26.0, 27.0],
    "lx" => [100000.0_f64, 99900.0, 99780.0],
}?;

let data_from_df_with_lx = MortData::from_df(df_lx)?;

// Macro to directly form MortData
// This is equivalent to forming dataframe then using from_df method
let data_from_macro = mddf! {
    "age" => [25_u32, 26, 27],
    "qx" => [0.001_f64, 0.0012, 0.0015],
}
```

There are various other methods to formulate life table. For examples, from spreadsheets

```rust
// Custom data from spreadsheet XLSX
let data_from_xlsx = MortData::from_xlsx("data/mortality.xlsx", "select")?;

// Custom data from spreadsheet ODS
let data_from_ods = MortData::from_ods("data/mortality.ods", "select")?;
```

Direct ingestion from SOA, IFOA and Australian Government Actuary mortality and morbidity database

More direct API are coming in the next releases. Please feel free to suggest your favorite database.

```rust
// ELT No.15 Female
let data_from_soa = MortData::from_soa_url_id(1704)?;

// AM92 Selected Mortality Table
let data_from_ifoa = MortData::from_ifoa_url_id("AM92")?;

// Male mortality rate in 2020-2022
let data_from_aga = MortData::from_aus_gov_act("Male", "2020-22")?;
```

## The Builder Pattern Advantage - IMMERSE in C4 principles

RSLife is designed to deliver actuarial developer experience founded on 💥 C4 💥 pillars - _Clear_, _Concise_, _Coherent_ and _Comprehensive_,  letting you IMMERSE yourselves in what truly matter for the core actuarial computation.

- **🎯 Intentional**: Only specify parameters that matter for each calculation
- **🗒️ Manageable**: Avoid clutter from declaring all parameters
- **🔧 Maintainable**: Adding new parameters doesn't break existing code
- **⚡ Efficient**: Automatic cross-field validation catches errors early
- **📖 Readable**: Self-documenting code that's easy to understand
- **🔒 Safe**: Compile-time validation prevents parameter mistakes
- **🧁 Effortless**: Capable to construct complex calculations with minial code

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

## Actuarial Functions & Naming Convention

### Function Structure

**Systematic Modifiers**:

- **Immediate / In arrears**: Single letter → `Ax`, `Axn` (payments at end of year)
- **Due / In advance**: Double letter → `aax`, `aaxn` (payments at start of year)
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

This project is dual-licensed under the Apache License, Version 2.0 and the MIT License.

You may choose either license when using this code.

See [![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-yellow.svg)](./LICENSE-APACHE) and [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE-MIT) for details.

## Contact

**Willian Nguyen** - [hieunt(dot)hello(at)gmail(dot)com](mailto:)

**Project link** - [https://github.com/hnlearndev/rslife](https://github.com/hnlearndev/rslife)

## References

- [Actuarial Mathematics for Life Contingent Risks](https://www.goodreads.com/book/show/58306503-actuarial-mathematics-for-life-contingent-risks)
- [Actuarial Mathematics](https://www.goodreads.com/book/show/1715653.Actuarial_Mathematics)
- [Society of Actuaries Mortality and Morbidity Tables](https://mort.soa.org)
- [Institute and Faculty of Actuaries Mortality and Morbidity Tables](https://www.actuaries.org.uk/learn-and-develop/continuous-mortality-investigation/cmi-mortality-and-morbidity-tables)
- [Australian Government Actuary](https://aga.gov.au)
- Standard actuarial notation and practices

## Similar Projects

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

## Enjoy?

Let me know that you find the crate helpful. Thank you :D

[![GitHub Sponsors](https://img.shields.io/github/sponsors/hnlearndev?color=red&logo=github&style=for-the-badge&label=GitHub%20Sponsors)](https://github.com/sponsors/hnlearndev)

[![Buy me a coffee](https://img.buymeacoffee.com/button-api/?text=Buy%20me%20a%20coffee&emoji=☕&slug=hnlearndev&button_colour=b5835a&font_colour=000000&font_family=Lato&outline_colour=000000&coffee_colour=FFDD00)](https://www.buymeacoffee.com/hnlearndev)
