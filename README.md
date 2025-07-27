# RSLife

A comprehensive Rust library for actuarial mortality table calculations and life insurance mathematics, featuring an elegant **builder pattern** that makes complex actuarial calculations intuitive and type-safe.

[![Crates.io](https://img.shields.io/crates/v/rslife.svg)](https://crates.io/crates/rslife)
[![Documentation](https://docs.rs/rslife/badge.svg)](https://docs.rs/rslife)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Why RSLife?

**🚀 Performance & Memory Efficiency:**
- Built on Rust's zero-cost abstractions for maximum performance
- Polars integration for efficient DataFrame operations
- Minimal memory allocation with smart data reuse
- Compile-time optimizations eliminate runtime overhead

**🎯 Developer Experience:**
- **Intuitive Builder Pattern**: Only specify parameters you need, no confusing parameter lists
- **Type Safety**: Compile-time validation prevents common actuarial calculation errors
- **Auto-Completion**: IDEs provide intelligent suggestions for all parameters
- **Self-Documenting**: Parameter names make code intent crystal clear

**📊 Data Flexibility:**
- **Universal Input**: DataFrames, XLSX/ODS files, and SOA XML with automatic format detection
- **Format Agnostic**: Works with `qx` rates or `lx` survivor functions seamlessly
- **Validation Built-In**: Comprehensive data integrity checks before calculations
- **Select & Ultimate**: Full support for both table types with automatic recognition

**🔧 Production Ready:**
- **Complete Actuarial Coverage**: Life insurance, annuities, and survival functions with standard notation
- **Multiple Assumptions**: UDD, CFM, and HPB methods for fractional age calculations
- **Consistent API**: All functions use the same parameter structure
- **Extensible**: Easy to add new parameters without breaking existing code
- **Error Handling**: Clear, actionable error messages for debugging

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rslife = "0.1.3"
```

### The Builder Pattern Advantage

```rust
use rslife::prelude::*;

fn main() - Result(), Boxdyn std::error::Error {
    let xml = MortXML::from_url_id(1704)?;

    // Method 1: Struct literal (specify all fields)
    let mt = MortTableConfig {
        xml: xml.clone(),
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    // Method 2: Builder pattern (only specify needed fields)
    let mt_builder = MortTableConfig::builder()
        .xml(xml)
        .radix(100_000)
        .pct(1.0)
        .assumption(AssumptionEnum::UDD)
        .build()?;

    // Builder pattern works for both configs!
    let life_annuity = aax(ParamConfig::builder()
        .mt(mt.clone())  // or mt_builder.clone()
        .i(0.03)
        .x(65)
        .build())?;

    let deferred_term = Ax1n(ParamConfig::builder()
        .mt(mt_builder)
        .i(0.03)
        .x(35)
        .n(15)
        .t(5)
        .build())?;

    let results = [
        ("Life Annuity", life_annuity),
        ("Deferred Term", deferred_term),
    ];

    for (name, value) in results {
        println!("{}: {:.6}", name, value);
    }

    // Survival probabilities
    println!("5-year survival from age 35: {:.4}", tpx(&mt, 35.0, 5.0, 0.0, None)?);
    Ok(())
}
```

**vs. Traditional Approaches:**

```rust
// ❌ Other libraries: verbose structs, easy to mess up parameter order
let params = ComplexConfig {
    mt: config,
    i: 0.03,
    x: 35,
    n: None,
    t: None,
    m: Some(1),
    moment: Some(1),
    entry_age: None,
};

let result = some_function(&config, 35, 0.03, 1, 0, None, None, 1)?; // What does this mean?

// ✅ RSLife: crystal clear, only specify what matters
let result = Ax(&ParamConfig::builder()
    .mt(config)
    .i(0.03)
    .x(35)
    .build())?;
```

### Custom Data Sources

RSLife supports flexible mortality data input with automatic `qx`/`lx` detection:

```rust
use polars::prelude::*;
use rslife::prelude::*;

// DataFrames - mortality rates or survivor functions
let df_qx = df! {
    "age" = [25u32, 26, 27],
    "qx" = [0.001f64, 0.0012, 0.0015],
}?;

let df_lx = df! {
    "age" = [25u32, 26, 27],
    "lx" = [100000.0f64, 99900.0, 99780.0],
}?;

// XLSX/ODS files - Excel or LibreOffice Calc
let xml_xlsx = MortXML::from_xlsx("data/mortality.xlsx", "ultimate")?;
let xml_ods = MortXML::from_ods("data/mortality.ods", "select")?;

// SOA XML - Official mortality tables
let xml_soa = MortXML::from_url_id(1704)?; // 2017 CSO table

// Both patterns work seamlessly with any data source

// Option A: Struct literal
let mt_struct = MortTableConfig {
    xml: MortXML::from_df(df_qx.clone())?,
    radix: Some(100_000),
    pct: Some(1.0),
    assumption: Some(AssumptionEnum::UDD),
};

// Option B: Builder pattern - Shoerter form because
// UDD for fractional, radix are not neceesary
// pct will be default to 1.0 so we do not need to setup.
let mt_builder = MortTableConfig::builder()
    .xml(MortXML::from_df(df_qx)?)
    .build()?;

let result = Ax(ParamConfig::builder()
    .mt(mt_builder)  // or mt_struct
    .i(0.05)
    .x(25)
    .build())?;
```

## Why the Builder Pattern is Superior

RSLife's builder pattern transforms complex actuarial calculations into intuitive, readable code:

```rust
// ❌ Traditional approaches in other languages
Calculate_Ax(
    mortality_table,
    interest_rate=0.03,
    age=35,
    term=null,
    deferral=null,
    frequency=1,
    moment=1
)

whole_life = LifeInsurance(
    config, 35, 0.03, None, None, 1, 1, None
)  # What do these parameters mean?

// ✅ RSLife's builder pattern
let whole_life = Ax(&ParamConfig::builder()
    .mt(config)
    .i(0.03)
    .x(35)
    .build())?;

let term_life = Ax1n(&ParamConfig::builder()
    .mt(config)
    .i(0.03)
    .x(35)
    .n(20)
    .build())?;
```

**Builder Pattern Advantages:**
- **🎯 Intentional**: Only specify parameters that matter for each calculation
- **🔒 Safe**: Compile-time validation prevents parameter mistakes
- **📖 Readable**: Self-documenting code that's easy to understand
- **🔧 Maintainable**: Adding new parameters doesn't break existing code
- **⚡ Efficient**: Automatic cross-field validation catches errors early

**Architecture Overview:**
- **`MortTableConfig`**: Mortality table settings (data source, assumptions, adjustments)
- **`ParamConfig`**: Calculation parameters with automatic validation
- **Functions**: Standard actuarial notation (`Ax`, `aax`, `tpx`, etc.)

## Data Format Requirements

### Supported Input Formats

**Data Sources:**
- **DataFrames**: Polars DataFrames with `qx` (rates 0.0-1.0) or `lx` (survivors from 100,000)
- **XLSX/ODS**: Excel or LibreOffice Calc spreadsheet files
- **SOA XML**: Direct access to Society of Actuaries mortality tables

### Data Structure Templates

**Ultimate Tables** (single mortality rate per age):

| Format | age | qx/lx | Example |
|--------|-----|-------|--------|
| **Mortality Rates** | 0,1,2,... | 0.006,0.0004,... | `qx` between 0.0-1.0 |
| **Survivor Function** | 0,1,2,... | 100000,99368,... | `lx` positive numbers |

**Select Tables** (mortality varies by duration since selection):

| Format | age | duration | qx/lx | Example |
|--------|-----|----------|-------|--------|
| **Mortality Rates** | 25,25,26,... | 1,2,1,... | 0.008,0.009,... | Include `duration` column |
| **Survivor Function** | 25,25,26,... | 1,2,1,... | 99200,99116,... | Include `duration` column |

### Data Validation Rules

- **Age**: `u32` type, sequential integer ages with no gaps
- **Mortality Data**: `f64` type, choose either `qx` (0.0-1.0) or `lx` (positive numbers)
- **Duration**: `u32` type, required only for select tables
- **Completeness**: No missing values within data ranges

### Practical Usage Example

```rust
use polars::prelude::*;
use rslife::prelude::*;

fn main() - Result(), Boxdyn std::error::Error {
    // Method 1: DataFrame with mortality rates
    let df = df! {
        "age" = [25u32, 26, 27, 28],
        "qx" = [0.001f64, 0.0012, 0.0015, 0.0018],
    }?;

    // Method 2: XLSX file (same format as above table)
    let xml_file = MortXML::from_xlsx("mortality.xlsx", "sheet1")?;

    // Method 3: Official SOA mortality table
    let xml_soa = MortXML::from_url_id(1704)?; // 2017 CSO

    let mt = MortTableConfig {
        xml: MortXML::from_df(df)?,  // Use any of the three methods
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    // Builder pattern works with any data source
    let insurance = Ax(ParamConfig::builder()
        .mt(mt)
        .i(0.03)
        .x(25)
        .build())?;

    println!("Whole life insurance value: {:.6}", insurance);
    Ok(())
}
```

> **💡 Spreadsheet Tips:**
> Use first row for headers • No empty cells in data range • Ages as integers • Mortality rates as decimals • Save as `.xlsx` or `.ods`

## Core Features & Performance

### Intelligent Data Processing

- **🔍 Automatic Detection**: Seamlessly detects `qx` (mortality rates) or `lx` (survivor functions) format without manual specification
- **📊 Smart Table Recognition**: Automatically determines ultimate vs select mortality tables based on column structure
- **⚡ Zero-Copy Optimization**: Efficient data processing with minimal memory overhead using Polars DataFrames
- **🛡️ Pre-Calculation Validation**: Comprehensive data integrity checks prevent runtime errors before calculations begin
- **🎯 Lazy Evaluation**: Only computes necessary mortality table components for your specific calculations

### Built-in Safety & Reliability

- **🔒 Type Safety**: `u32` for ages prevents negative values and floating-point errors
- **✅ Cross-Field Validation**: Parameter combinations are validated automatically (e.g., term length vs age ranges)
- **📋 Comprehensive Error Handling**: Clear, actionable error messages for data format issues
- **🧪 Battle-Tested**: Validated against standard actuarial references and SOA mortality tables

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

All functions now use `ParamConfig` for consistent parameter passing and automatic validation.

### Full list of actuarial functions available via `rslife::prelude::*`

**Cetain annuities:**

- `aan`, `an`

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

For more details, please visit the [full documentation](https://docs.rs/rslife)

## Examples

Check out the `examples/` directory for comprehensive examples:

- **`basic_usage.rs`** - Demonstrates both UDD and CFM mortality assumptions with ParamConfig
- **`soa_data_demo.rs`** - Loading real SOA mortality tables (2017 CSO, 1980 CSO) with calculations
- **`custom_data_demo.rs`** - Working with XLSX files and custom DataFrames, including fallback examples

### Running Examples

```bash
# Basic usage example
cargo run --example basic_usage

# SOA data demonstration
cargo run --example soa_data_demo

# Custom data sources
cargo run --example custom_data_demo
```

### Key Features Demonstrated

- **Parameter validation** with automatic error detection
- **Multiple data sources** (SOA XML, XLSX, DataFrames)
- **Mortality assumptions** (UDD vs CFM comparisons)
- **Select vs ultimate** mortality tables
- **Comprehensive calculations** (life insurance, annuities, survival probabilities)


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
- [ActuaryUtilities.jl](https://github.com/JuliaActuary/ActuaryUtilities.jl) - Julia utilities for actuarial modeling and analysisk.
