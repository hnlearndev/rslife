use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use rslife::prelude::*;

fn setup_mortality_table() -> MortTableConfig {
    let mort_data = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92");
    MortTableConfig::builder()
        .data(mort_data)
        .radix(100_000)
        .build()
        .expect("Failed to create MortTableConfig")
}

fn bench_commutation_functions(c: &mut Criterion) {
    let mt = setup_mortality_table();
    let interest_rate = 0.04;

    // Benchmark individual commutation functions
    c.bench_function("Dx_single_age", |b| {
        b.iter(|| Dx().mt(&mt).i(interest_rate).x(45).call().unwrap())
    });

    c.bench_function("Nx_single_age", |b| {
        b.iter(|| Nx().mt(&mt).i(interest_rate).x(45).call().unwrap())
    });

    c.bench_function("Cx_single_age", |b| {
        b.iter(|| Cx().mt(&mt).i(interest_rate).x(45).call().unwrap())
    });

    c.bench_function("Mx_single_age", |b| {
        b.iter(|| Mx().mt(&mt).i(interest_rate).x(45).call().unwrap())
    });

    // Benchmark with entry age (2D table)
    c.bench_function("Dx_with_entry_age", |b| {
        b.iter(|| {
            Dx().mt(&mt)
                .i(interest_rate)
                .x(47)
                .entry_age(45)
                .call()
                .unwrap()
        })
    });

    // Benchmark range calculations (computationally intensive)
    c.bench_function("Nx_early_age_intensive", |b| {
        b.iter(|| Nx().mt(&mt).i(interest_rate).x(25).call().unwrap())
    });

    c.bench_function("Mx_early_age_intensive", |b| {
        b.iter(|| Mx().mt(&mt).i(interest_rate).x(25).call().unwrap())
    });

    // Benchmark double summation functions (most intensive)
    c.bench_function("Rx_computation", |b| {
        b.iter(|| Rx().mt(&mt).i(interest_rate).x(30).call().unwrap())
    });

    c.bench_function("Sx_computation", |b| {
        b.iter(|| Sx().mt(&mt).i(interest_rate).x(30).call().unwrap())
    });
}

fn bench_commutation_bulk_operations(c: &mut Criterion) {
    let mt = setup_mortality_table();
    let interest_rate = 0.04;

    // Benchmark multiple age calculations
    c.bench_function("Dx_age_range_30_50", |b| {
        b.iter(|| {
            for age in 30..=50 {
                Dx().mt(&mt).i(interest_rate).x(age).call().unwrap();
            }
        })
    });

    c.bench_function("commutation_table_generation", |b| {
        b.iter(|| {
            let ages = [25, 35, 45, 55, 65];
            for age in ages {
                Dx().mt(&mt).i(interest_rate).x(age).call().unwrap();
                Nx().mt(&mt).i(interest_rate).x(age).call().unwrap();
                Cx().mt(&mt).i(interest_rate).x(age).call().unwrap();
                Mx().mt(&mt).i(interest_rate).x(age).call().unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    bench_commutation_functions,
    bench_commutation_bulk_operations
);
criterion_main!(benches);
