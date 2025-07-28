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
- **Intuitive Builder Pattern**: Only specify parameters you need, no confusing parameter lists
- **Type Safety**: Compile-time validation prevents common actuarial calculation errors
- **Auto-Completion**: IDEs provide intelligent suggestions for all parameters
- **Self-Documenting**: Parameter names make code intent crystal clear
- **Cross-Field Validation**: Parameter combinations validated automatically

**📊 Intelligent Data Processing:**
- **Universal Input**: DataFrames, XLSX/ODS files, and SOA XML with automatic format detection
- **Format Agnostic**: Seamlessly detects `qx` rates or `lx` survivor functions without manual specification
- **Smart Table Recognition**: Automatically determines ultimate vs select mortality tables
- **Validation Built-In**: Comprehensive data integrity checks prevent runtime errors before calculations
- **Select & Ultimate**: Full support for both table types with automatic recognition

**🔧 Production Ready:**
- **Complete Actuarial Coverage**: Life insurance, annuities, and survival functions with standard notation
- **Multiple Assumptions**: UDD, CFM, and HPB methods for fractional age calculations
- **Consistent API**: All functions use the same parameter structure with builder pattern
- **Battle-Tested**: Validated against standard actuarial references and SOA mortality tables
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load mortality data from SOA (Society of Actuaries)
    let data = MortData::from_soa_url_id(1704)?;

    // Method 1: Struct literal (specify all fields)
    let mt = MortTableConfig {
        data: data.clone(),
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    // Method 2: Builder pattern (only specify needed fields)
    let mt_builder = MortTableConfig::builder()
        .data(data)
        .radix(100_000)
        .pct(1.0)
        .assumption(AssumptionEnum::UDD)
        .build()?;

    // New builder pattern for actuarial calculations!
    let life_annuity = aax()
        .mt(&mt)  // or &mt_builder
        .i(0.03)
        .x(65)
        .call()?;

    let deferred_term = Ax1n()
        .mt(&mt_builder)
        .i(0.03)
        .x(35)
        .n(15)
        .t(5)
        .call()?;

    let results = [
        ("Life Annuity", life_annuity),
        ("Deferred Term", deferred_term),
    ];

    for (name, value) in results {
        println!("{}: {:.6}", name, value);
    }

    // Survival probabilities with new builder pattern
    let survival_5_years = tpx()
        .mt(&mt)
        .x(35.0)
        .t(5.0)
        .k(0.0)
        .call()?;

    println!("5-year survival from age 35: {:.4}", survival_5_years);
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
let result = Ax()
    .mt(&config)
    .i(0.03)
    .x(35)
    .call()?;
```

### Custom Data Sources

RSLife supports flexible mortality data input with automatic `qx`/`lx` detection:

```rust
use polars::prelude::*;
use rslife::prelude::*;

// DataFrames - mortality rates or survivor functions
let df_qx = df! {
    "age" => [25.0, 26.0, 27.0],
    "qx" => [0.001, 0.0012, 0.0015],
}?;

let df_lx = df! {
    "age" => [25.0, 26.0, 27.0],
    "lx" => [100000.0, 99900.0, 99780.0],
}?;

// Load data from various sources
let data_from_df = MortData::from_df(df_qx.clone())?;
let data_from_file = MortData::from_ods("data/mortality.ods", "select")?;
let data_from_soa = MortData::from_soa_url_id(1704)?; // 2017 CSO table

// Both patterns work seamlessly with any data source

// Option A: Struct literal
let mt_struct = MortTableConfig {
    data: data_from_df,
    radix: Some(100_000),
    pct: Some(1.0),
    assumption: Some(AssumptionEnum::UDD),
};

// Option B: Builder pattern - shorter form with defaults
// UDD assumption, 100k radix, and 1.0 pct are defaults
let mt_builder = MortTableConfig::builder()
    .data(data_from_soa)
    .build()?;

// New actuarial function builder pattern
let result = Ax()
    .mt(&mt_builder)  // or &mt_struct
    .i(0.05)
    .x(25)
    .call()?;
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
let whole_life = Ax()
    .mt(&config)
    .i(0.03)
    .x(35)
    .call()?;

let term_life = Ax1n()
    .mt(&config)
    .i(0.03)
    .x(35)
    .n(20)
    .call()?;
```

**Builder Pattern Advantages:**
- **🎯 Intentional**: Only specify parameters that matter for each calculation
- **🔒 Safe**: Compile-time validation prevents parameter mistakes
- **📖 Readable**: Self-documenting code that's easy to understand
- **🔧 Maintainable**: Adding new parameters doesn't break existing code
- **⚡ Efficient**: Automatic cross-field validation catches errors early

**Architecture Overview:**

- **`MortTableConfig`**: Mortality table settings (data source, assumptions, adjustments)
- **`SingleLifeParams`**: Single life calculation parameters with automatic validation
- **`SurvivalFunctionParams`**: Survival function parameters with automatic validation
- **Functions**: Standard actuarial notation (`Ax`, `aax`, `tpx`, etc.) with builder pattern

## Data Format Requirements

### Supported Input Formats

**Data Sources:**

- **DataFrames**: Polars DataFrames with `qx` (rates 0.0-1.0) or `lx` (survivors from 100,000)
- **XLSX/ODS**: Excel or LibreOffice Calc spreadsheet files
- **SOA XML**: Direct access to Society of Actuaries mortality tables

### Data Structure Templates

RSLife supports three primary data structures for mortality tables. Each can use either mortality rates (`qx`) or survivor functions (`lx`).

#### 1. Ultimate Tables (age + qx)

Single mortality rate per age - the most common format:

| age | qx      |
|-----|--------|
| 25  | 0.00120 |
| 26  | 0.00135 |
| 27  | 0.00150 |

```rust
let df_ultimate = df! {
    "age" => [25u32, 26, 27, 28, 29],
    "qx" => [0.00120f64, 0.00135, 0.00150, 0.00168, 0.00188],
}?;
```

#### 2. Ultimate Tables (age + lx)

Survivor function format - same data expressed as remaining lives:

| age | lx       |
|-----|----------|
| 25  | 100000.0 |
| 26  | 99880.0  |
| 27  | 99745.1  |
| 28  | 99595.4  |
| 29  | 99428.2  |

```rust
// Load survivor function data from Excel/LibreOffice Calc file
let data_survivors = MortData::from_xlsx("mortality_lx.xlsx", "survivors")?;
let mt = MortTableConfig::builder()
    .data(data_survivors)
    .build()?;
```

#### 3. Select Tables (age + qx + duration)

Mortality varies by years since policy issue - used for medically underwritten policies:

| age | qx      | duration |
|-----|---------|----------|
| 35  | 0.00080 | 1        |
| 35  | 0.00095 | 2        |
| 35  | 0.00110 | 3        |
| 36  | 0.00085 | 1        |
| 36  | 0.00102 | 2        |
| 36  | 0.00118 | 3        |

```rust
// Load select table data from LibreOffice Calc file
let data_select = MortData::from_ods("select_mortality.ods", "select_table")?;
let mt = MortTableConfig::builder()
    .data(data_select)
    .radix(100_000)
    .build()?;
```

**Key Features:**

- **Automatic Detection**: RSLife automatically detects whether data contains `qx` rates or `lx` survivor functions
- **Select vs Ultimate**: Presence of `duration` column automatically identifies select tables
- **Format Flexibility**: All three formats work seamlessly with the same API
- **Data Validation**: Built-in checks ensure data integrity before calculations

### Data Validation Rules

- **Age**: `u32` type, sequential integer ages with no gaps
- **Mortality Data**: `f64` type, choose either `qx` (0.0-1.0) or `lx` (positive numbers)
- **Duration**: `u32` type, required only for select tables
- **Completeness**: No missing values within data ranges

### Practical Usage Example

```rust
use polars::prelude::*;
use rslife::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Method 1: DataFrame with mortality rates
    let df = df! {
        "age" => [25.0, 26.0, 27.0, 28.0],
        "qx" => [0.001, 0.0012, 0.0015, 0.0018],
    }?;

    // Method 2: XLSX file (same format as above table)
    let data_file = MortData::from_xlsx("mortality.xlsx", "sheet1")?;

    // Method 3: Official SOA mortality table
    let data_soa = MortData::from_soa_url_id(1704)?; // 2017 CSO

    let mt = MortTableConfig {
        data: MortData::from_df(df)?,  // Use any of the three methods
        radix: Some(100_000),
        pct: Some(1.0),
        assumption: Some(AssumptionEnum::UDD),
    };

    // New builder pattern for actuarial calculations
    let insurance = Ax()
        .mt(&mt)
        .i(0.03)
        .x(25)
        .call()?;

    println!("Whole life insurance value: {:.6}", insurance);
    Ok(())
}
```

> **💡 Spreadsheet Tips:**
> Use first row for headers • No empty cells in data range • Ages as integers • Mortality rates as decimals • Save as `.xlsx` or `.ods`


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

All functions now use the builder pattern with `SingleLifeParams` and `SurvivalFunctionParams` for consistent parameter passing and automatic validation.

### Full list of actuarial functions available via `rslife::prelude::*`

**Cetain annuities:**

- `aan`, `an`

**Annuities:**

- `aax`, `aaxn`
- `Iaax`, `Iaaxn`
- `Daaxn`
- `gaax`, `gaaxn`

**Benefits and Life Insurance:**

- `Ax`, `Ax1n`, `Exn`, `Axn`
- `IAx`, `IAx1n`, `IAxn`
- `DAx1n`, `DAxn`
- `gAx`, `gAx1n`, `gExn`, `gAxn`

**Survival Probabilities:**

- `tpx`, `tqx`

For more details, please visit the [full documentation](https://docs.rs/rslife)

## Examples

Check out the `examples/` directory for comprehensive examples:

- **`basic_usage.rs`** - Demonstrates both UDD and CFM mortality assumptions with builder pattern
- **`soa_data_demo.rs`** - Loading real SOA mortality tables (2017 CSO, 1980 CSO) with calculations
- **`custom_data_demo.rs`** - Working with XLSX files and custom DataFrames, including fallback examples

### Running Examples

```bash
# Basic usage example
cargo run --example basic_usage
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
