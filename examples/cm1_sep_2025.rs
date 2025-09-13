#![allow(non_snake_case)]

use approx::assert_abs_diff_eq;
use rslife::prelude::*;

fn main() -> RSLifeResult<()> {
    q1()?;
    q2()?;
    // q12()?;
    Ok(())
}

fn q1() -> RSLifeResult<()> {
    // September 2025 CM1 question 1
    // PV = 50500 * aₙ - 500 * (Ia)ₙ =  with t= n = 30, i = 4.5%
    let an = an().i(0.045).n(30).call()?;
    let Ian = Ian().i(0.045).n(30).call()?;
    let answer = 50500.0 * an - 500.0 * Ian;

    //------------------------------------------------------------------------------------
    // This is a simple assertion to check the result from examiner's report
    let expected = 722456.78;
    assert_abs_diff_eq!(answer, expected, epsilon = 1e-2);
    //------------------------------------------------------------------------------------

    println!("\n=== CM1 September 2025 Q1 Results ===");
    println!("The answer is {answer:.2}");
    Ok(())
}

fn q2() -> RSLifeResult<()> {
    // September 2025 CM1 question 2
    let pma92c20 = MortData::from_builtin("PMA92C20")?;
    let mt = MortTableConfig::builder()
        .data(pma92c20)
        .assumption(AssumptionEnum::CFM)
        .build()?;
    let answer = axn().mt(&mt).i(0.09).x(85.5).n(0.75).m(4).call()?;

    //------------------------------------------------------------------------------------
    // This is a simple assertion to check the result from examiner's report
    let expected = 0.684147;
    assert_abs_diff_eq!(answer, expected, epsilon = 1e-6);
    //------------------------------------------------------------------------------------

    println!("\n=== CM1 September 2025 Q1 Results ===");
    println!("The answer is {answer:.6}");
    Ok(())
}

// fn q12() -> RSLifeResult<()> {
//     // September 2025 CM1 question 2
//     let am92 = MortData::from_builtin("AM92")?;
//     let mt = MortTableConfig::builder().data(am92).build()?;

//     // Pure endowment
//     // Päₓ:ₙ̅=

//     //------------------------------------------------------------------------------------
//     // This is a simple assertion to check the result from examiner's report
//     let expected = 0.684147;
//     assert_abs_diff_eq!(answer, expected, epsilon = 1e-6);
//     //------------------------------------------------------------------------------------

//     println!("\n=== CM1 September 2025 Q1 Results ===");
//     println!("The answer is {answer:.6}");
//     Ok(())
// }
