# RSLife

A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics, following standard actuarial principles and notation.

[![Crates.io](https://img.shields.io/crates/v/rslife.svg)](https://crates.io/crates/rslife)
[![Documentation](https://docs.rs/rslife/badge.svg)](https://docs.rs/rslife)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Unified API Interface**: Consistent and clear parameter requirements across all actuarial functions for enhanced usability
- **Performance Optimized**: 4-level detail system automatically optimizes calculations for your needs
- **XML Parsing**: Load mortality data from Society of Actuaries (SOA) XML sources using ACORD XTbML standard
- **Multiple Mortality Assumptions**: UDD, CFM, and HPB methods for fractional age calculations
- **Comprehensive Functions**: Life insurance, annuities, and demographic calculations
- **Standard Notation**: Follows traditional actuarial notation with modern Rust conventions
- **Polars Integration**: Built on Polars DataFrames for efficient data processing
- **Well-Documented**: Extensive documentation with mathematical formulations

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rslife = "0.1.3"
```

### Basic Example

```rust
use rslife::prelude::*;

fn main() -> PolarsResult<()> {
    // Load SOA mortality table
    let xml = MortXML::from_url_id(1704)?;
    let config = MortTableConfig {
        xml,
        radix: Some(100_000),
        int_rate: Some(0.03),
        pct: Some(0.01),
        assumption: AssumptionEnum::UDD
    };

    // Calculate actuarial values
    let whole_life = Ax(&config, 35, 0  None)?;
    let annuity = aaxn(&config, 35, 1, 1, 0, None)?;
    let survival = tpx(&config, 5.0, 30.0, None)?;

    println!("Whole life: {:.6}", whole_life);
    println!("Annuity: {:.6}", annuity);
    println!("5yr survival: {:.6}", survival);

    Ok(())
}
```

### Custom Data Example

```rust
use polars::prelude::*;
use rslife::prelude::*;

fn main() -> PolarsResult<()> {
    // Create custom mortality DataFrame
    let df = df! {
        "age" => [30, 31, 32, 33, 34],
        "qx" => [0.001, 0.0012, 0.0015, 0.0018, 0.002],
    }?;

    // Load from DataFrame
    let xml = MortXML::from_df(df)?;
    let config = MortTableConfig {
        xml,
        radix: Some(100_000),
        int_rate: Some(0.05),
        pct: Some(0.01),
        assumption: Some(AssumptionEnum::UDD)
    };

    let whole_life_benefit = Ax(&config, 30, 0, None)?;

    println!("Custom table insurance value: {:.6}", whole_life_benefit);

    // Load from Excel file
    let xml = MortXML::from_df(df)?;
    let config = MortTableConfig {
        xml,
        radix: Some(100_000),
        int_rate: Some(0.05),
        pct: Some(0.01),
        assumption: Some(AssumptionEnum::UDD)
    };

    let 5_year_deferred_increasing_endowment_benefit = IAxn(&config, 20, 10, 5, 0, None)?;

    println!("Custom table insurance value: {:.6}", 5_year_deferred_increasing_endowment_benefit);

    Ok(())
}
```

## Performance Optimization

### SOA Mortality Table Automatical Classification

- Only XML files with exactly 1 table are supported.
- The package automatically detects whether `qx` or `lx` is provided and generates a complete mortality table as needed.
- Selection functions automatically detect whether the appropriate SOA mortality table is used for calculation.

### Computation

RSLife automatically optimizes performance with a 4-level detail system:

- **Level 1** (~3x faster): Demographics only `age`, `qx`, `px`, `lx`, `dx`
- **Level 2** (standard): Level 1 + basic commutation `Cx`, `Dx`
- **Level 3** (extended): Level 2 + additional commutation `Mx`, `Nx`, `Px`
- **Level 4** (complete): Level 3 + additional `Rx`, `Sx`

Functions automatically select the minimum required level for optimal performance.

## Mortality Assumptions

The library supports three standard actuarial assumptions for fractional age calculations:

### UDD (Uniform Distribution of Deaths)

Linear interpolation between integer ages:

```text
ₜpₓ = 1 - t · qₓ
```

### CFM (Constant Force of Mortality)

Exponential survival model:

```text
ₜpₓ = (1 - qₓ)ᵗ
```

### HPB (Hyperbolic/Balmer)

Hyperbolic interpolation:

```text
ₜpₓ = (1 - qₓ) / (1 - (1-t) · qₓ)
```

## Actuarial Functions & Naming Convention

### Function Structure

**Systematic Modifiers**:

- **Imediate**: Single letter → `Ax`, `Axn` (payments at end)
- **Due**: Double letter → `aax`, `aaxn` (payments at start)
- **Increasing**: `I` prefix → `IAx`, `Iaax` (arithmetic growth)
- **Decreasing**: `D` prefix → `DAx1n`, `Daaxn` (arithmetic decrease)
- **Geometric**: `g` prefix → `gAx`, `gaax` (geometric growth)

These modifiers are applicable to most but not all functions.

Please take a look at [Full list](https//:google.com)

### Full list of actuarial functions available via `rslife::prelude::*`

**Annuities:**

- `aax`, `aaxn`
- `Iaax`, `Iaaxn`
- `Daaxn`
- `gaax`, `gaaxn`

**Benefits and Life Insurance:**

- `Ax`, `Ax1n`, `nEx`, `Axn`
- `IAx`, `IAx1n`, `IAxn`
- `DAx1n`, `DAxn`
- `gAx`, `gAx1n`, `gnEx`, `gAxn`

**Survival Probabilities:**

- `tpx`, `tqx`

For more details, please visit the [full documentation](google.com)

## Examples

Check out the `examples/` directory for more comprehensive examples:

- `prelude_demo.rs` - Basic usage with the prelude
- `mortality_calculations.rs` - Detailed actuarial calculations
- `xml_loading.rs` - Various ways to load mortality data

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

**Willian Nguyen** - [hieunt.hello@gmail.com](mailto:hieunt.hello@gmail.com)

Project Link: [https://github.com/hnlearndev/rslife](https://github.com/hnlearndev/rslife)

## References

- [Actuarial Mathematics for Life Contingent Risks](https://www.goodreads.com/book/show/58306503-actuarial-mathematics-for-life-contingent-risks)
- [Actuarial Mathematics](https://www.goodreads.com/book/show/1715653.Actuarial_Mathematics)
- [Society of Actuaries Mortality Tables](https://mort.soa.org)
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
- [ActuaryUtilities.jl](https://github.com/JuliaActuary/ActuaryUtilities.jl) - Julia utilities for actuarial modeling and analysis

**Note**:

Mojo is a relatively new language and doesn't yet have established actuarial libraries, but its performance characteristics make it promising for computational actuarial work.
