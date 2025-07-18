# RSLife

A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics, following standard actuarial principles and notation.

**Built on Polars** - Leveraging the latest high-performance DataFrame technology for lightning-fast actuarial computations with memory efficiency and parallel processing capabilities that outperform traditional pandas-based solutions.

[![Crates.io](https://img.shields.io/crates/v/rslife.svg)](https://crates.io/crates/rslife)
[![Documentation](https://docs.rs/rslife/badge.svg)](https://docs.rs/rslife)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **XML Parsing**: Load mortality data from Society of Actuaries (SOA) XML sources using the ACORD XTbML standard ([mort.soa.org](https://mort.soa.org/About.aspx))
- **Multiple Mortality Assumptions**: UDD, CFM, and HPB methods for fractional age calculations
- **Comprehensive Actuarial Functions**: Life insurance, annuities, and demographic calculations
- **Standard Notation**: Follows traditional actuarial notation (Ax, axn, etc.)
- **Polars Integration**: Built on top of Polars DataFrames for efficient data processing
- **Well-Documented**: Extensive documentation with mathematical formulations

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rslife = "0.1.0"
```

### Using the Prelude (Recommended)

The easiest way to get started is using the prelude module:

```rust
use rslife::prelude::*;

fn main() -> PolarsResult<()> {
    // Load mortality data from XML
    let elt_15_female_xml = MortXML::from_url_id(1704)?;

    // Configure mortality table
    let config = MortTableConfig {
        xml: elt_15_female_xml,
        l_x_init: 100_000,
        pct: Some(1.0),
        int_rate: Some(0.03),
        assumption: Some(AssumptionEnum::UDD),
    };

    // Generate mortality table
    let mortality_table = config.gen_mort_table()?;
    println!("Generated mortality table with {} rows", mortality_table.height());

    // Calculate actuarial values
    let whole_life_35 = Ax(&config, 35)?;
    let term_insurance_35_20 = Axn(&config, 35, 20)?;
    let life_annuity_35_12 = axn_due(&config, 35, 20, 12)?;

    println!("Whole life insurance (age 35): {:.6}", whole_life_35);
    println!("20-year term insurance (age 35): {:.6}", term_insurance_35_20);
    println!("20-year life annuity due (age 35): {:.6}", life_annuity_35_12);

    Ok(())
}
```

### Manual Imports

If you prefer explicit imports:

```rust
use rslife::xml::MortXML;
use rslife::actuarial::{MortTableConfig, AssumptionEnum, Ax, Axn, axn_due};
use polars::prelude::PolarsResult;
```

## Mortality Assumptions

The library supports three standard actuarial assumptions for fractional age calculations:

**Note**: The codebase is organized into `wholes.rs` (integer ages only) and `fractionals.rs` (fractional ages with UDD/CFM/HPB assumptions). Currently, most functions are in `wholes.rs` for compatibility, but future updates will migrate calculations to `fractionals.rs` to properly support all three mortality assumptions.

### UDD (Uniform Distribution of Deaths)
Linear interpolation between integer ages:
$${}_{t}p_x = 1 - t \cdot q_x$$

### CFM (Constant Force of Mortality)
Exponential survival model:
$${}_{t}p_x = (1 - q_x)^t$$

### HPB (Hyperbolic/Balmer)
Hyperbolic interpolation:
$${}_{t}p_x = \frac{1 - q_x}{1 - (1-t) \cdot q_x}$$

## Available Functions

### Life Insurance Benefits

| Function | Description | Notation |
|----------|-------------|----------------|
| `Ax(config, x)` | Whole life insurance | $A_x$ |
| `Axn(config, x, n)` | n-year term insurance | $A^1_{x:\overline{n\|}}$ |
| `Exn(config, x, n)` | n-year pure endowment | $E_{x:n}$ |
| `AExn(config, x, n)` | n-year endowment | $A_{x:\overline{n\|}}$ |
| `IAx(config, x)` | Increasing whole life | $(IA)_x$ |
| `IAxn(config, x, n)` | Increasing term | $(IA)^1_{x:\overline{n\|}}$ |
| `tAx(config, x, t)` | t-year deferred whole life | ${}_{t\|}A_x$ |
| `tAxn(config, x, n, t)` | t-year deferred term | ${}_{t\|}A^1_{x:\overline{n\|}}$ |
| `tExn(config, x, n, t)` | t-year deferred pure endowment | ${}_{t\|}E_{x:n}$ |
| `tAExn(config, x, n, t)` | t-year deferred endowment | ${}_{t\|}A_{x:\overline{n\|}}$ |
| `gAx(config, x, g)` | Geometric increasing whole life | $A_x^{(g)}$ |
| `gAxn(config, x, n, g)` | Geometric increasing term | $(A^{(g)})^1_{x:\overline{n\|}}$ |

### Annuities

| Function | Description | Notation |
|----------|-------------|----------------|
| `axn_due(config, x, n, m)` | n-year life annuity due (m-payable) | $\ddot{a}_{x:\overline{n\|}}^{(m)}$ |
| `tax_due(config, x, t, m)` | t-year deferred annuity (m-payable) | ${}_{t\|}\ddot{a}_x^{(m)}$ |
| `taxn_due(config, x, n, t, m)` | t-year deferred term annuity (m-payable) | ${}_{t\|}\ddot{a}_{x:\overline{n\|}}^{(m)}$ |
| `Iax_due(config, x, n, m)` | Increasing whole life annuity due (m-payable) | $(I\ddot{a})_x^{(m)}$ |
| `Iaxn_due(config, x, n, m)` | Increasing n-year annuity due (m-payable) | $(I\ddot{a})_{x:\overline{n\|}}^{(m)}$ |
| `tIax_due(config, x, n, t, m)` | t-year deferred increasing annuity (m-payable) | ${}_{t\|}(I\ddot{a})_x^{(m)}$ |
| `tIaxn_due(config, x, n, t, m)` | t-year deferred increasing term annuity (m-payable) | ${}_{t\|}(I\ddot{a})_{x:\overline{n\|}}^{(m)}$ |
| `gIax_due(config, x, n, m, g)` | Geometric increasing annuity due (m-payable) | $(I\ddot{a})_x^{(g,m)}$ |
| `gIaxn_due(config, x, n, m, g)` | Geometric increasing term annuity due (m-payable) | $(I\ddot{a})_{x:\overline{n\|}}^{(g,m)}$ |

### Fractional Age Functions

| Function | Description | Notation |
|----------|-------------|----------------|
| `tpx(config, t, x)` | Survival probability for t years | ${}_{t}p_x$ |
| `tqx(config, t, x)` | Death probability within t years | ${}_{t}q_x$ |
| `conditional_tqx(config, t, x, s)` | Conditional death probability | ${}_{t\mid s}q_x$ |

## XML Data Sources

Load mortality data from various sources:

```rust
use rslife::prelude::*;

// From SOA table ID (downloads from mort.soa.org)
let xml1 = MortXML::from_url_id(1704)?;

// From local table ID (loads from src/table_xml/ folder)
let xml2 = MortXML::from_id(912)?;

// From direct URL
let xml3 = MortXML::from_url("https://mort.soa.org/data/t1704.xml")?;

// From local file
let xml4 = MortXML::from_path(Path::new("t1704.xml"))?;

// From XML string
let xml_string = r#"<xml>...</xml>"#;
let xml5 = MortXML::from_string(xml_string)?;
```

**Note**: The `from_id()` method loads XML files from the default `src/table_xml/` folder, while `from_url_id()` downloads directly from the SOA website.

**Table IDs**: You can find mortality table IDs at [mort.soa.org](https://mort.soa.org/Default.aspx) - the first column with title "#" contains the ID numbers to use with `from_id()` and `from_url_id()` functions.


## Examples

Check out the `examples/` directory for more comprehensive examples:

- `prelude_demo.rs` - Basic usage with the prelude
- `mortality_calculations.rs` - Detailed actuarial calculations
- `xml_loading.rs` - Various ways to load mortality data

## Mathematical Documentation

All functions include comprehensive mathematical documentation with LaTeX formulas. View the full documentation at [docs.rs/rslife](https://docs.rs/rslife).

**Note**: Function names follow traditional actuarial notation (e.g., `Ax`, `Axn`) rather than Rust's snake_case convention to maintain consistency with mathematical literature and industry standards. The compiler warnings about snake_case naming can be safely ignored for this domain-specific library.

**Math Rendering**: The notation in this README uses GitHub's limited math rendering. For complete actuarial notation with proper symbols (including annuity bars, life tables, and complex subscripts), refer to the generated rustdoc documentation which supports full LaTeX rendering with actuarial packages.

## Roadmap

### Version 0.2.0 (Q4 2025)

- **Enhanced Fractional Age Support**: Migrate all calculations to `fractionals.rs` module for full UDD/CFM/HPB assumption support
- **Selection with Duration Tables**: Add support for selection with duration table XML parsing and calculations ($q_{[x]+t}$ notation)
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

Project Link: [https://github.com/hnlearndev/Basic-Term-Model-Rust-lifelibBasicTermSM](https://github.com/hnlearndev/Basic-Term-Model-Rust-lifelibBasicTermSM)

## References
- [Actuarial Mathematics (Bowers et al.)](https://www.soa.org/shop/actuarial-mathematics/)
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
