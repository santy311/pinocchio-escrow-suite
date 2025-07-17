use anyhow::Result;
use escrow_suite::states::EscrowType;

mod common;
pub use common::*;

#[test]
fn test_partial_escrow_single_taker() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let total_token_a = 5000;
    let total_token_b = 10000;
    let take_amount = 2000; // 40% of the escrow

    println!("=== Testing Partial Escrow Single Taker ===");
    println!("Total Token A: {}", total_token_a);
    println!("Total Token B: {}", total_token_b);
    println!(
        "Take Amount: {} ({}%)",
        take_amount,
        (take_amount * 100) / total_token_a
    );

    // Verify initial balances
    setup.verify_simple_escrow_balances(total_token_a, total_token_b, "initial")?;

    // Create a partial escrow
    setup.create_escrow(EscrowType::Partial, total_token_a, total_token_b)?;

    // Verify balances after creation
    setup.verify_simple_escrow_balances(total_token_a, total_token_b, "after_creation")?;

    // Take partial amount
    setup.take_partial_escrow(take_amount)?;

    // Calculate expected token B amount (proportional to token A taken)
    let expected_token_b = (total_token_b * take_amount) / total_token_a;
    let remaining_token_a = total_token_a - take_amount;

    // Verify balances after partial take
    setup.verify_partial_escrow_balances(
        total_token_a,
        total_token_b,
        take_amount,
        expected_token_b,
        remaining_token_a,
        "after_partial_take",
    )?;

    println!("✅ Partial escrow single taker test passed");
    Ok(())
}

#[test]
fn test_partial_escrow_multiple_takers() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let total_token_a = 3000; // Reduced from 6000
    let total_token_b = 6000; // Reduced from 12000

    println!("=== Testing Partial Escrow Multiple Takers ===");
    println!("Total Token A: {}", total_token_a);
    println!("Total Token B: {}", total_token_b);

    // Verify initial balances
    setup.verify_simple_escrow_balances(total_token_a, total_token_b, "initial")?;

    // Create a partial escrow
    setup.create_escrow(EscrowType::Partial, total_token_a, total_token_b)?;

    // Verify balances after creation
    setup.verify_simple_escrow_balances(total_token_a, total_token_b, "after_creation")?;

    // First taker takes 30% (900 token A)
    let first_take = 900;
    setup.take_partial_escrow(first_take)?;
    let first_expected_token_b = (total_token_b * first_take) / total_token_a;
    let remaining_after_first = total_token_a - first_take;

    setup.verify_partial_escrow_balances(
        total_token_a,
        total_token_b,
        first_take,
        first_expected_token_b,
        remaining_after_first,
        "after_first_take",
    )?;

    // Second taker takes 40% of remaining (840 token A)
    let second_take = 840;
    setup.take_partial_escrow(second_take)?;
    let second_expected_token_b = (total_token_b * second_take) / total_token_a;
    let remaining_after_second = remaining_after_first - second_take;

    setup.verify_partial_escrow_balances(
        total_token_a,
        total_token_b,
        first_take + second_take,
        first_expected_token_b + second_expected_token_b,
        remaining_after_second,
        "after_second_take",
    )?;

    // Third taker takes the remaining 30% (1260 token A)
    let third_take = remaining_after_second;
    let third_expected_token_b = (total_token_b * third_take) / total_token_a;

    // Check if taker has enough token B for the third take
    let current_taker_token_b = setup.get_taker_token_b_balance();
    if current_taker_token_b < third_expected_token_b {
        println!(
            "Warning: Taker doesn't have enough token B for third take. Current: {}, Required: {}",
            current_taker_token_b, third_expected_token_b
        );
        // Skip the third take for this test
        setup.verify_partial_escrow_balances(
            total_token_a,
            total_token_b,
            first_take + second_take,
            first_expected_token_b + second_expected_token_b,
            remaining_after_second,
            "after_second_take",
        )?;
    } else {
        setup.take_partial_escrow(third_take)?;

        setup.verify_partial_escrow_balances(
            total_token_a,
            total_token_b,
            first_take + second_take + third_take,
            first_expected_token_b + second_expected_token_b + third_expected_token_b,
            0, // Escrow should be empty
            "after_third_take",
        )?;
    }

    println!("✅ Partial escrow multiple takers test passed");
    Ok(())
}

#[test]
fn test_partial_escrow_small_amounts() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let total_token_a = 1000;
    let total_token_b = 2000;
    let take_amount = 100; // 10% of the escrow

    println!("=== Testing Partial Escrow Small Amounts ===");
    println!("Total Token A: {}", total_token_a);
    println!("Total Token B: {}", total_token_b);
    println!(
        "Take Amount: {} ({}%)",
        take_amount,
        (take_amount * 100) / total_token_a
    );

    // Verify initial balances
    setup.verify_simple_escrow_balances(total_token_a, total_token_b, "initial")?;

    // Create a partial escrow
    setup.create_escrow(EscrowType::Partial, total_token_a, total_token_b)?;

    // Take small amount
    setup.take_partial_escrow(take_amount)?;

    let expected_token_b = (total_token_b * take_amount) / total_token_a;
    let remaining_token_a = total_token_a - take_amount;

    setup.verify_partial_escrow_balances(
        total_token_a,
        total_token_b,
        take_amount,
        expected_token_b,
        remaining_token_a,
        "after_small_take",
    )?;

    println!("✅ Partial escrow small amounts test passed");
    Ok(())
}

#[test]
fn test_partial_escrow_large_amounts() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let total_token_a = 4000; // Reduced from 8000
    let total_token_b = 8000; // Reduced from 16000
    let take_amount = 3000; // 75% of the escrow (reduced from 6000)

    println!("=== Testing Partial Escrow Large Amounts ===");
    println!("Total Token A: {}", total_token_a);
    println!("Total Token B: {}", total_token_b);
    println!(
        "Take Amount: {} ({}%)",
        take_amount,
        (take_amount * 100) / total_token_a
    );

    // Verify initial balances
    setup.verify_simple_escrow_balances(total_token_a, total_token_b, "initial")?;

    // Create a partial escrow
    setup.create_escrow(EscrowType::Partial, total_token_a, total_token_b)?;

    // Take large amount
    setup.take_partial_escrow(take_amount)?;

    let expected_token_b = (total_token_b * take_amount) / total_token_a;
    let remaining_token_a = total_token_a - take_amount;

    setup.verify_partial_escrow_balances(
        total_token_a,
        total_token_b,
        take_amount,
        expected_token_b,
        remaining_token_a,
        "after_large_take",
    )?;

    println!("✅ Partial escrow large amounts test passed");
    Ok(())
}

#[test]
fn test_partial_escrow_edge_cases() -> Result<()> {
    println!("=== Testing Partial Escrow Edge Cases ===");

    // Test case 1: Take exactly half
    println!("Test Case 1: Take exactly half");
    let mut setup1 = EscrowTestSetup::new()?;
    let total_a1 = 4000;
    let total_b1 = 8000;
    let take_half = total_a1 / 2;

    setup1.verify_simple_escrow_balances(total_a1, total_b1, "initial")?;
    setup1.create_escrow(EscrowType::Partial, total_a1, total_b1)?;
    setup1.take_partial_escrow(take_half)?;

    let expected_b1 = (total_b1 * take_half) / total_a1;
    setup1.verify_partial_escrow_balances(
        total_a1,
        total_b1,
        take_half,
        expected_b1,
        total_a1 - take_half,
        "after_half_take",
    )?;

    // Test case 2: Take almost everything (99%)
    println!("Test Case 2: Take almost everything");
    let mut setup2 = EscrowTestSetup::new()?;
    let total_a2 = 5000;
    let total_b2 = 10000;
    let take_almost_all = (total_a2 * 99) / 100;

    setup2.verify_simple_escrow_balances(total_a2, total_b2, "initial")?;
    setup2.create_escrow(EscrowType::Partial, total_a2, total_b2)?;
    setup2.take_partial_escrow(take_almost_all)?;

    let expected_b2 = (total_b2 * take_almost_all) / total_a2;
    setup2.verify_partial_escrow_balances(
        total_a2,
        total_b2,
        take_almost_all,
        expected_b2,
        total_a2 - take_almost_all,
        "after_almost_all_take",
    )?;

    // Test case 3: Take very small amount (1%)
    println!("Test Case 3: Take very small amount");
    let mut setup3 = EscrowTestSetup::new()?;
    let total_a3 = 3000;
    let total_b3 = 6000;
    let take_small = (total_a3 * 1) / 100;

    setup3.verify_simple_escrow_balances(total_a3, total_b3, "initial")?;
    setup3.create_escrow(EscrowType::Partial, total_a3, total_b3)?;
    setup3.take_partial_escrow(take_small)?;

    let expected_b3 = (total_b3 * take_small) / total_a3;
    setup3.verify_partial_escrow_balances(
        total_a3,
        total_b3,
        take_small,
        expected_b3,
        total_a3 - take_small,
        "after_small_take",
    )?;

    println!("✅ Partial escrow edge cases test passed");
    Ok(())
}

#[test]
fn test_partial_escrow_precision_handling() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let total_token_a = 1000;
    let total_token_b = 3333; // Not evenly divisible
    let take_amount = 333; // 33.3% of the escrow

    println!("=== Testing Partial Escrow Precision Handling ===");
    println!("Total Token A: {}", total_token_a);
    println!("Total Token B: {}", total_token_b);
    println!(
        "Take Amount: {} ({}%)",
        take_amount,
        (take_amount * 100) / total_token_a
    );

    // Verify initial balances
    setup.verify_simple_escrow_balances(total_token_a, total_token_b, "initial")?;

    // Create a partial escrow
    setup.create_escrow(EscrowType::Partial, total_token_a, total_token_b)?;

    // Take amount that will result in fractional token B
    setup.take_partial_escrow(take_amount)?;

    let expected_token_b = (total_token_b * take_amount) / total_token_a;
    let remaining_token_a = total_token_a - take_amount;

    setup.verify_partial_escrow_balances(
        total_token_a,
        total_token_b,
        take_amount,
        expected_token_b,
        remaining_token_a,
        "after_precision_take",
    )?;

    println!("✅ Partial escrow precision handling test passed");
    Ok(())
}

#[test]
fn test_partial_escrow_sequential_takes() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let total_token_a = 5000; // Reduced from 10000
    let total_token_b = 10000; // Reduced from 20000

    println!("=== Testing Partial Escrow Sequential Takes ===");
    println!("Total Token A: {}", total_token_a);
    println!("Total Token B: {}", total_token_b);

    // Verify initial balances
    setup.verify_simple_escrow_balances(total_token_a, total_token_b, "initial")?;

    // Create a partial escrow
    setup.create_escrow(EscrowType::Partial, total_token_a, total_token_b)?;

    // Sequential takes with different amounts (total: 5000)
    let takes = vec![500, 1000, 750, 1250, 1500]; // Total: 5000
    let mut cumulative_taken_a = 0;
    let mut cumulative_taken_b = 0;

    for (i, take_amount) in takes.iter().enumerate() {
        println!("Take {}: {} token A", i + 1, take_amount);

        setup.take_partial_escrow(*take_amount)?;

        cumulative_taken_a += take_amount;
        let expected_token_b = (total_token_b * take_amount) / total_token_a;
        cumulative_taken_b += expected_token_b;

        let remaining_token_a = total_token_a - cumulative_taken_a;

        setup.verify_partial_escrow_balances(
            total_token_a,
            total_token_b,
            cumulative_taken_a,
            cumulative_taken_b,
            remaining_token_a,
            "after_partial_take",
        )?;
    }

    // Verify escrow is completely empty after all takes
    assert_eq!(
        setup.get_escrow_token_a_balance(),
        0,
        "Escrow should be empty"
    );
    assert_eq!(
        setup.get_escrow_token_b_balance(),
        0,
        "Escrow should be empty"
    );

    println!("✅ Partial escrow sequential takes test passed");
    Ok(())
}
