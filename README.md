# RSLife

A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics, following standard actuarial principles and notation.

**Built on Polars** - Leveraging high-performance DataFrame technology for fast actuarial computations with memory efficiency and parallel processing capabilities.

[![Crates.io](https://img.shields.io/crates/v/rslife.svg)](https://crates.io/crates/rslife)
[![Documentation](https://docs.rs/rslife/badge.svg)](https://docs.rs/rslife)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Performance Optimized**: 4-level detail system automatically optimizes calculations for your needs
- **XML Parsing**: Load mortality data from Society of Actuaries (SOA) XML sources using ACORD XTbML standard
- **Multiple Mortality Assumptions**: UDD, CFM, and HPB methods for fractional age calculations
- **Comprehensive Functions**: Life insurance, annuities, and demographic calculations
- **Standard Notation**: Follows traditional actuarial notation (Ax, äx, etc.)
- **Polars Integration**: Built on Polars DataFrames for efficient data processing
- **Well-Documented**: Extensive documentation with mathematical formulations

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rslife = "0.1.1"
```

### Basic Example

```rust
use rslife::prelude::*;

fn main() -> PolarsResult<()> {
    // Load mortality data from SOA
    let xml = MortXML::from_url_id(1704)?;

    // Configure mortality table
    let config = MortTableConfig {
        xml,
        l_x_init: 100_000,
        pct: Some(1.0),
        int_rate: Some(0.03),
        assumption: Some(AssumptionEnum::UDD),
    };

    // Calculate actuarial values
    let whole_life_35 = Ax(&config, 35)?;
    let life_annuity_35 = aaxn(&config, 35, 1)?;

    // Fractional age survival
    let survival_5_years = tpx(&config, 5.0, 30.0)?;

    println!("Whole life insurance (age 35): {:.6}", whole_life_35);
    println!("Life annuity due (age 35): {:.6}", life_annuity_35);
    println!("5-year survival from age 30: {:.6}", survival_5_years);

    Ok(())
}
```

## Performance Optimization

RSLife automatically optimizes performance with a 4-level detail system:

- **Level 1** (~10x faster): Basic mortality rates (`qx`, `px`)
- **Level 2** (~5x faster): Demographics (`lx`, `dx`)
- **Level 3** (~2x faster): Commutation functions (`Dx`, `Nx`, etc.)
- **Level 4** (complete): All actuarial functions (`Ax`, `äx`, etc.)

Functions automatically use the minimum required level for optimal performance.

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

## Key Functions

### Life Insurance

- `Ax(config, x)` - Whole life insurance
- `Axn(config, x, n)` - Term insurance
- `Exn(config, x, n)` - Pure endowment
- `AExn(config, x, n)` - Endowment
- `IAx(config, x)` - Increasing whole life
- `IAxn(config, x, n)` - Increasing term insurance
- `gAx(config, x, g)` - Geometric increasing whole life
- `gAxn(config, x, n, g)` - Geometric increasing term insurance

### Deferred Insurance

- `tAx(config, x, t)` - Deferred whole life insurance
- `tAxn(config, x, t, n)` - Deferred term insurance
- `tExn(config, x, t, n)` - Deferred pure endowment
- `tAExn(config, x, t, n)` - Deferred endowment

### Annuities

- `aax(config, x, m)` - Life annuity due with m payable
- `aaxn(config, x, n, m)` - Life temporary annuity due with m payable
- `Iaax(config, x, n, m)` - Increasing annuity with m payable
- `Iaaxn(config, x, n, m)` - Increasing temporary annuity with m payable
- `gIaax(config, x, n, m, g)` - Geometric increasing annuity with m payable
- `gIaaxn(config, x, n, m, g)` - Geometric increasing temporary annuity with m payable

### Deferred Annuities

- `taax(config, x, t, m)` - Deferred annuity with m payable
- `taaxn(config, x, t, n, m)` - Deferred temporary annuity with m payable
- `tIaax(config, x, n, t, m)` - Deferred increasing annuity with m payable
- `tIaaxn(config, x, n, t, m)` - Deferred increasing temporary annuity with m payable

  ### Survival Functions

- `tpx(config, t, x)` - Survival probability for t years
- `tqx(config, t, x)` - Death probability within t years

## Data Sources

Load mortality data from various sources:

```rust
// From SOA website (by table ID)
let xml = MortXML::from_url_id(1704)?;****

// From local file
let xml = MortXML::from_path("mortality_table.xml")?;

// From URL
let xml = MortXML::from_url("https://mort.soa.org/data/t1704.xml")?;

// From XML string
let xml_string = r#"<MortalityTable>...</MortalityTable>"#;
let xml = MortXML::from_string(xml_string)?;
```

**Table IDs**: You can find mortality table IDs at [mort.soa.org](https://mort.soa.org/Default.aspx) - the first column with title "#" contains the ID numbers.

## Examples

Check out the `examples/` directory for more comprehensive examples:

- `prelude_demo.rs` - Basic usage with the prelude
- `mortality_calculations.rs` - Detailed actuarial calculations
- `xml_loading.rs` - Various ways to load mortality data

## Mathematical Documentation

All functions include comprehensive mathematical documentation with Unicode formulas. View the full documentation at [docs.rs/rslife](https://docs.rs/rslife).

**Note**: Function names follow traditional actuarial notation (e.g., `Ax`, `Axn`) rather than Rust's snake_case convention to maintain consistency with mathematical literature and industry standards. The compiler warnings about snake_case naming can be safely ignored for this domain-specific library.

**Math Rendering**: The notation in this README and documentation uses Unicode characters for optimal rendering on both GitHub and crates.io, ensuring mathematical formulas display correctly across all platforms without requiring LaTeX rendering support.

## Roadmap

### Version 0.2.0 (Q4 2025)

- **Enhanced Fractional Age Support**: Migrate all calculations to `fractional.rs` module for full UDD/CFM/HPB assumption support
- **Selection with Duration Tables**: Add support for selection with duration table XML parsing and calculations (qₓ₊ₜ notation)
- **Additional Mortality Functions**: Add `lx`, `dx`, `qx` series functions for demographic analysis
- **Performance Optimizations**: Implement caching for commutation function calculations
- **Extended XML Support**: Add support for additional mortality table formats and international standards

### Version 0.3.0 (Q1 2026)

- **Multi-Life Functions**: Joint life, last survivor, and contingent insurance calculations
- **Pension Mathematics**: Add pension actuarial functions and retirement calculations
- **Stochastic Models**: Implement Lee-Carter and other stochastic mortality models
- **Parallel Processing**: Add optional parallel computation for large-scale calculations

### Version 1.0.0 (Q1 2026)

- **API Stabilization**: Finalize public API with semantic versioning guarantees
- **Advanced Features**: Health insurance, disability models, and multi-state transitions
- **Integration Tools**: Export capabilities for Excel, R, and Python interoperability
- **Regulatory Compliance**: Support for Solvency II, IFRS 17, and other regulatory frameworks

### Long-term Vision

- **Machine Learning Integration**: Mortality forecasting and risk modeling
- **Real-time Data Sources**: Live mortality data feeds and automatic updates
- **Web Assembly Support**: Browser-based actuarial calculations
- **Educational Tools**: Interactive tutorials and learning modules

Contributions and feedback on the roadmap are welcome! Please open an issue to discuss priority features or suggest new directions.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

**Trung-Hieu Nguyen** - [hieunt.hello@gmail.com](mailto:hieunt.hello@gmail.com)

Project Link: [https://github.com/hnlearndev/Basic-Term-Model-Rust-lifelibBasicTermSM](https://github.com/hnlearndev//rslife)

## References

- [Actuarial Mathematics (Bowers et al.)](https://www.soa.org/shop/actuarial-mathematics)
- [Society of Actuaries Mortality Tables](https://mort.soa.org/Default.aspx)
- Standard actuarial notation and practices

### Similar Projects

**Python:**

- [pylife](https://github.com/actuarialopensource/pylife) - Python library for actuarial calculations and life insurance mathematics
- [pymort](https://github.com/actuarialopensource/pymort) - Python mortality table library with XML parsing capabilities

**R:**

- [lifecontingencies](https://github.com/spedygiorgio/lifecontingencies) - R package for actuarial life contingencies calculations
- [MortalityTables](https://github.com/kainhofer/r-mortality-tables) - R package for working with life and pension tables
- [demography](https://github.com/robjhyndman/demography) - R package for demographic analysis and mortality forecasting

**Julia:**

- [MortalityTables.jl](https://github.com/JuliaActuary/MortalityTables.jl) - Julia package for mortality table calculations and life contingencies
- [ActuaryUtilities.jl](https://github.com/JuliaActuary/ActuaryUtilities.jl) - Julia utilities for actuarial modeling and analysis

**Note**: Mojo is a relatively new language and doesn't yet have established actuarial libraries, but its performance characteristics make it promising for computational actuarial work.
