# CodSpeed Integration Setup Guide

## Overview

This document explains how to complete the CodSpeed.io integration for continuous performance monitoring of the `rslife` actuarial library.

## What We've Set Up

✅ **Criterion.rs with CodSpeed compatibility** - Added to `Cargo.toml`  
✅ **Comprehensive benchmark suite** - Created 3 benchmark files:
- `benches/commutations.rs` - Commutation functions (Dx, Nx, Mx, Cx, Rx, Sx)
- `benches/mortality_functions.rs` - Basic mortality functions (qx, lx, dx, px)
- `benches/annuities.rs` - Annuity calculations and variations

✅ **GitHub Actions workflow** - `.github/workflows/codspeed.yml`

## Manual Steps Required

### 1. Configure CodSpeed.io Account

1. **Visit CodSpeed.io**
   ```
   https://codspeed.io/
   ```

2. **Sign up with GitHub**
   - Use your GitHub account to sign up
   - Grant necessary permissions

3. **Connect your repository**
   - Add the `rslife` repository to CodSpeed
   - Follow the integration wizard

4. **Generate API token**
   - Go to your CodSpeed dashboard
   - Navigate to Settings → Tokens
   - Generate a new token for `rslife`

### 2. Add GitHub Secret

1. Go to your GitHub repository: `https://github.com/hnlearndev/rslife`
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Name: `CODSPEED_TOKEN`
5. Value: Paste the token from CodSpeed.io
6. Click **Add secret**

### 3. Test the Setup

Run benchmarks locally to verify everything works:

```bash
# Install dependencies
cargo build

# Run individual benchmark suites
cargo bench --bench commutations
cargo bench --bench mortality_functions  
cargo bench --bench annuities

# Run all benchmarks
cargo bench
```

### 4. Verify CI Integration

1. Create a test branch and make a small change
2. Open a PR to main branch
3. Check that CodSpeed workflow runs successfully
4. Verify CodSpeed comments appear on the PR

## Benchmark Coverage

### Core Mathematical Functions
- **Commutation Functions**: Dx, Nx, Mx, Cx, Rx, Sx with various scenarios
- **Mortality Functions**: qx, lx, dx, px lookups and calculations  
- **Life Annuities**: Whole life, temporary, deferred variations
- **Certain Annuities**: Fixed-period calculations
- **Increasing Annuities**: Growing payment streams

### Performance Scenarios
- Single value calculations
- Bulk operations (age ranges)
- 1D vs 2D table operations
- Different mortality tables (AM92, SULT)
- Various interest rates and payment frequencies
- Edge cases (young/old ages)

### Data Loading Benchmarks
- Mortality table loading from different sources
- Configuration setup performance

## Using CodSpeed Results

### Interpreting Results
- **Green indicators**: Performance maintained or improved
- **Red indicators**: Performance regression detected
- **Statistical significance**: CodSpeed uses rigorous statistical analysis

### Performance Monitoring
- Track trends over time in CodSpeed dashboard
- Set up alerts for significant regressions
- Compare performance across different actuarial scenarios

### Optimization Workflow
1. Make code changes
2. Run benchmarks locally: `cargo bench`
3. Check CodSpeed PR comments for performance impact
4. Investigate any regressions before merging

## Maintenance

### Adding New Benchmarks
When adding new actuarial functions:

1. Add benchmark to appropriate file in `benches/`
2. Follow existing patterns for setup and measurement
3. Include realistic scenarios and edge cases
4. Test locally before committing

### Benchmark Best Practices
- Use representative data (real mortality tables)
- Cover common usage patterns
- Include performance-critical code paths
- Avoid I/O operations in benchmark loops
- Use consistent test data across benchmarks

## Troubleshooting

### Common Issues

**Benchmark compilation errors:**
- Check that all imports in benchmark files are correct
- Ensure CodSpeed dependencies are properly added

**CodSpeed workflow failing:**
- Verify `CODSPEED_TOKEN` secret is set correctly
- Check GitHub Actions logs for specific errors
- Ensure network access for mortality table downloads

**Inconsistent results:**
- CodSpeed uses statistical analysis to filter out noise
- Local variations are normal; focus on CI results
- Consistent regressions indicate real performance issues

### Getting Help
- CodSpeed documentation: https://docs.codspeed.io/
- Criterion.rs guide: https://bheisler.github.io/criterion.rs/
- GitHub Issues for this repository

## Benefits for RSLife

### Continuous Performance Monitoring
- Automatic detection of performance regressions
- Historical performance tracking
- Statistical analysis to distinguish real changes from noise

### Actuarial-Specific Value
- **Mathematical Functions**: Monitor computational performance of complex actuarial calculations
- **Data Processing**: Track efficiency of mortality table operations
- **Scalability**: Ensure library performs well with various table sizes and calculation scenarios

### Development Workflow
- Performance feedback on every PR
- Early detection of expensive changes
- Data-driven optimization decisions

## Next Steps

1. Complete CodSpeed account setup
2. Add `CODSPEED_TOKEN` to GitHub secrets
3. Test with a sample PR
4. Monitor performance trends
5. Expand benchmark coverage as needed

For questions about actuarial calculations or benchmark interpretation, refer to the main `README.md` and documentation.
