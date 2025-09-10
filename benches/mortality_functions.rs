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

fn setup_sult_table() -> MortTableConfig {
    let mort_data = MortData::from_soa_custom("SULT").expect("Failed to load SULT");
    MortTableConfig::builder()
        .data(mort_data)
        .radix(100_000)
        .build()
        .expect("Failed to create SULT MortTableConfig")
}

fn bench_basic_mortality_functions(c: &mut Criterion) {
    let mt = setup_mortality_table();

    c.bench_function("qx_single_lookup", |b| {
        b.iter(|| mt.qx().x(45).call().unwrap())
    });

    c.bench_function("lx_single_lookup", |b| {
        b.iter(|| mt.lx().x(45).call().unwrap())
    });

    c.bench_function("dx_single_calculation", |b| {
        b.iter(|| mt.dx().x(45).call().unwrap())
    });

    c.bench_function("px_single_calculation", |b| {
        b.iter(|| mt.px().x(45).call().unwrap())
    });
}

fn bench_mortality_with_entry_age(c: &mut Criterion) {
    let mt = setup_mortality_table();

    c.bench_function("qx_with_entry_age", |b| {
        b.iter(|| mt.qx().x(47).entry_age(45).call().unwrap())
    });

    c.bench_function("lx_with_entry_age", |b| {
        b.iter(|| mt.lx().x(47).entry_age(45).call().unwrap())
    });

    c.bench_function("dx_with_entry_age", |b| {
        b.iter(|| mt.dx().x(47).entry_age(45).call().unwrap())
    });
}

fn bench_mortality_table_operations(c: &mut Criterion) {
    let mt = setup_mortality_table();

    c.bench_function("mortality_table_min_max_age", |b| {
        b.iter(|| {
            let _min = mt.min_age().unwrap();
            let _max = mt.max_age().unwrap();
        })
    });

    c.bench_function("mortality_table_duration_bounds", |b| {
        b.iter(|| {
            let _min_dur = mt.min_duration().unwrap();
            let _max_dur = mt.max_duration().unwrap();
        })
    });
}

fn bench_bulk_mortality_calculations(c: &mut Criterion) {
    let mt = setup_mortality_table();

    c.bench_function("qx_age_range_25_65", |b| {
        b.iter(|| {
            for age in 25..=65 {
                mt.qx().x(age).call().unwrap();
            }
        })
    });

    c.bench_function("mortality_life_table_generation", |b| {
        b.iter(|| {
            let ages = [25, 30, 35, 40, 45, 50, 55, 60, 65, 70];
            for age in ages {
                mt.qx().x(age).call().unwrap();
                mt.lx().x(age).call().unwrap();
                mt.dx().x(age).call().unwrap();
                mt.px().x(age).call().unwrap();
            }
        })
    });
}

fn bench_different_mortality_tables(c: &mut Criterion) {
    let am92 = setup_mortality_table();
    let sult = setup_sult_table();

    c.bench_function("AM92_qx_lookup", |b| {
        b.iter(|| am92.qx().x(45).call().unwrap())
    });

    c.bench_function("SULT_qx_lookup", |b| {
        b.iter(|| sult.qx().x(45).call().unwrap())
    });
}

fn bench_mortality_data_loading(c: &mut Criterion) {
    c.bench_function("load_AM92_data", |b| {
        b.iter(|| {
            let mort_data = MortData::from_ifoa_url_id("AM92").unwrap();
            MortTableConfig::builder()
                .data(mort_data)
                .radix(100_000)
                .build()
                .unwrap()
        })
    });

    c.bench_function("load_SULT_data", |b| {
        b.iter(|| {
            let mort_data = MortData::from_soa_custom("SULT").unwrap();
            MortTableConfig::builder()
                .data(mort_data)
                .radix(100_000)
                .build()
                .unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_basic_mortality_functions,
    bench_mortality_with_entry_age,
    bench_mortality_table_operations,
    bench_bulk_mortality_calculations,
    bench_different_mortality_tables,
    bench_mortality_data_loading
);
criterion_main!(benches);
