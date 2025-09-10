use codspeed_criterion_compat::{Criterion, criterion_group, criterion_main};
use rslife::prelude::*;

fn setup_am92_table() -> MortTableConfig {
    let mort_data = MortData::from_ifoa_url_id("AM92").expect("Failed to load AM92");
    MortTableConfig::builder()
        .data(mort_data)
        .radix(100_000)
        .build()
        .expect("Failed to create AM92 MortTableConfig")
}

fn setup_pma92c20_table() -> MortTableConfig {
    let mort_data = MortData::from_ifoa_custom("PMA92C20").expect("Failed to load PMA92C20");
    MortTableConfig::builder()
        .data(mort_data)
        .radix(100_000)
        .build()
        .expect("Failed to create PMA92C20 MortTableConfig")
}

fn bench_life_annuities_due(c: &mut Criterion) {
    let mt = setup_am92_table();
    let interest_rate = 0.04;
 
    // Life annuity-due (aax)
    c.bench_function("aax_life_annuity_due_age_30", |b| {
        b.iter(|| aax().mt(&mt).i(interest_rate).x(30).call().unwrap())
    });

    c.bench_function("aax_life_annuity_due_age_50", |b| {
        b.iter(|| aax().mt(&mt).i(interest_rate).x(50).call().unwrap())
    });

    c.bench_function("aax_life_annuity_due_with_entry_age", |b| {
        b.iter(|| {
            aax()
                .mt(&mt)
                .i(interest_rate)
                .x(50)
                .entry_age(50)
                .call()
                .unwrap()
        })
    });

    // Temporary annuity-due (aaxn)
    c.bench_function("aaxn_temporary_annuity_due_20_years", |b| {
        b.iter(|| aaxn().mt(&mt).i(interest_rate).x(45).n(20).call().unwrap())
    });

    // Deferred life annuity-due (aax with deferral)
    c.bench_function("aax_deferred_life_annuity_due_5_years", |b| {
        b.iter(|| aax().mt(&mt).i(interest_rate).x(45).t(5).call().unwrap())
    });
}

fn bench_life_annuities_immediate(c: &mut Criterion) {
    let mt = setup_am92_table();
    let interest_rate = 0.04;

    // Life annuity-immediate (ax)
    c.bench_function("ax_life_annuity_immediate_age_50", |b| {
        b.iter(|| ax().mt(&mt).i(interest_rate).x(50).call().unwrap())
    });

    // Temporary annuity-immediate (axn)
    c.bench_function("axn_temporary_annuity_immediate_20_years", |b| {
        b.iter(|| axn().mt(&mt).i(interest_rate).x(45).n(20).call().unwrap())
    });

    // Test different payment frequencies (monthly payments)
    c.bench_function("aax_monthly_payments_m12", |b| {
        b.iter(|| aax().mt(&mt).i(interest_rate).x(45).m(12).call().unwrap())
    });

    c.bench_function("aaxn_quarterly_payments_m4", |b| {
        b.iter(|| {
            aaxn()
                .mt(&mt)
                .i(interest_rate)
                .x(45)
                .n(20)
                .m(4)
                .call()
                .unwrap()
        })
    });
}

fn bench_increasing_annuities(c: &mut Criterion) {
    let mt = setup_am92_table();
    let interest_rate = 0.04;

    // Increasing life annuity-due (Iaax)
    c.bench_function("Iaax_increasing_life_annuity_due", |b| {
        b.iter(|| Iaax().mt(&mt).i(interest_rate).x(50).call().unwrap())
    });

    // Increasing temporary life annuity-due (Iaaxn)
    c.bench_function("Iaaxn_increasing_temporary_annuity_due", |b| {
        b.iter(|| Iaaxn().mt(&mt).i(interest_rate).x(45).n(10).call().unwrap())
    });

    // Increasing life annuity-immediate (Iax)
    c.bench_function("Iax_increasing_life_annuity_immediate", |b| {
        b.iter(|| Iax().mt(&mt).i(interest_rate).x(45).call().unwrap())
    });

    // Increasing temporary life annuity-immediate (Iaxn)
    c.bench_function("Iaxn_increasing_temporary_annuity_immediate", |b| {
        b.iter(|| Iaxn().mt(&mt).i(interest_rate).x(45).n(12).call().unwrap())
    });
}

fn bench_geometric_annuities(c: &mut Criterion) {
    let mt = setup_am92_table();
    let interest_rate = 0.04;
    let growth_rate = 0.02;

    // Geometric increasing life annuity-due (gaax)
    c.bench_function("gaax_geometric_life_annuity_due", |b| {
        b.iter(|| {
            gaax()
                .mt(&mt)
                .i(interest_rate)
                .x(40)
                .g(growth_rate)
                .call()
                .unwrap()
        })
    });

    // Geometric increasing temporary annuity-due (gaaxn)
    c.bench_function("gaaxn_geometric_temporary_annuity_due", |b| {
        b.iter(|| {
            gaaxn()
                .mt(&mt)
                .i(interest_rate)
                .x(40)
                .n(10)
                .g(growth_rate)
                .call()
                .unwrap()
        })
    });

    // Geometric increasing life annuity-immediate (gax)
    c.bench_function("gax_geometric_life_annuity_immediate", |b| {
        b.iter(|| {
            gax()
                .mt(&mt)
                .i(interest_rate)
                .x(40)
                .g(growth_rate)
                .call()
                .unwrap()
        })
    });

    // Geometric increasing temporary annuity-immediate (gaxn)
    c.bench_function("gaxn_geometric_temporary_annuity_immediate", |b| {
        b.iter(|| {
            gaxn()
                .mt(&mt)
                .i(interest_rate)
                .x(40)
                .n(10)
                .g(growth_rate)
                .call()
                .unwrap()
        })
    });
}

fn bench_decreasing_annuities(c: &mut Criterion) {
    let mt = setup_am92_table();
    let interest_rate = 0.04;

    // Decreasing temporary life annuity-due (Daaxn)
    c.bench_function("Daaxn_decreasing_temporary_annuity_due", |b| {
        b.iter(|| Daaxn().mt(&mt).i(interest_rate).x(40).n(10).call().unwrap())
    });

    // Decreasing temporary life annuity-immediate (Daxn)
    c.bench_function("Daxn_decreasing_temporary_annuity_immediate", |b| {
        b.iter(|| Daxn().mt(&mt).i(interest_rate).x(40).n(10).call().unwrap())
    });
}

fn bench_annuity_bulk_calculations(c: &mut Criterion) {
    let mt = setup_am92_table();
    let pma_mt = setup_pma92c20_table();
    let interest_rate = 0.04;

    // Test scenarios similar to unit tests
    c.bench_function("annuity_table_generation_multiple_ages", |b| {
        b.iter(|| {
            let ages = [25, 35, 45, 55, 65];
            for age in ages {
                aax().mt(&mt).i(interest_rate).x(age).call().unwrap();
                ax().mt(&mt).i(interest_rate).x(age).call().unwrap();
                aaxn().mt(&mt).i(interest_rate).x(age).n(20).call().unwrap();
            }
        })
    });

    c.bench_function("annuity_sensitivity_analysis_interest_rates", |b| {
        b.iter(|| {
            let rates = [0.02, 0.03, 0.04, 0.05, 0.06];
            for rate in rates {
                aax().mt(&mt).i(rate).x(45).call().unwrap();
            }
        })
    });

    // Compare different mortality tables (like unit tests do)
    c.bench_function("annuity_comparison_mortality_tables", |b| {
        b.iter(|| {
            aax().mt(&mt).i(0.04).x(30).call().unwrap();
            aax().mt(&pma_mt).i(0.04).x(75).call().unwrap();
        })
    });

    // Test relationship between advance and arrear payments
    c.bench_function("annuity_advance_vs_arrear_relationship", |b| {
        b.iter(|| {
            let advance = aax().mt(&mt).i(0.04).x(50).call().unwrap();
            let arrear = ax().mt(&mt).i(0.04).x(50).call().unwrap();
            let _diff = advance - arrear; // Should be approximately 1.0
        })
    });
}

criterion_group!(
    benches,
    bench_life_annuities_due,
    bench_life_annuities_immediate,
    bench_increasing_annuities,
    bench_geometric_annuities,
    bench_decreasing_annuities,
    bench_annuity_bulk_calculations
);
criterion_main!(benches);
