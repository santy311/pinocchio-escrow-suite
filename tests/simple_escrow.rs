use anyhow::Result;
use escrow_suite::states::EscrowType;

mod common;
pub use common::*;

#[test]
fn test_simple_escrow_basic_flow() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let token_a_amount = 5000;
    let token_b_amount = 10000;

    println!("=== Testing Simple Escrow Basic Flow ===");
    println!("Token A Amount: {}", token_a_amount);
    println!("Token B Amount: {}", token_b_amount);

    // Verify initial balances
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "initial")?;

    // Create a simple escrow
    setup.create_escrow(EscrowType::Simple, token_a_amount, token_b_amount)?;

    // Verify balances after creation
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_creation")?;

    // Take the escrow
    setup.take_escrow()?;

    // Verify balances after take
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_take")?;

    println!("✅ Simple escrow basic flow test passed");
    Ok(())
}

#[test]
fn test_simple_escrow_different_amounts() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let token_a_amount = 1000;
    let token_b_amount = 5000;

    println!("=== Testing Simple Escrow with Different Amounts ===");
    println!("Token A Amount: {}", token_a_amount);
    println!("Token B Amount: {}", token_b_amount);

    // Verify initial balances
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "initial")?;

    // Create escrow with different amounts
    setup.create_escrow(EscrowType::Simple, token_a_amount, token_b_amount)?;

    // Verify balances after creation
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_creation")?;

    // Take the escrow
    setup.take_escrow()?;

    // Verify balances after take
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_take")?;

    println!("✅ Simple escrow with different amounts test passed");
    Ok(())
}

#[test]
fn test_simple_escrow_small_amounts() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let token_a_amount = 100;
    let token_b_amount = 250;

    println!("=== Testing Simple Escrow with Small Amounts ===");
    println!("Token A Amount: {}", token_a_amount);
    println!("Token B Amount: {}", token_b_amount);

    // Verify initial balances
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "initial")?;

    // Create escrow with small amounts
    setup.create_escrow(EscrowType::Simple, token_a_amount, token_b_amount)?;

    // Verify balances after creation
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_creation")?;

    // Take the escrow
    setup.take_escrow()?;

    // Verify balances after take
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_take")?;

    println!("✅ Simple escrow with small amounts test passed");
    Ok(())
}

#[test]
fn test_simple_escrow_large_amounts() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let token_a_amount = 8000;
    let token_b_amount = 9500;

    println!("=== Testing Simple Escrow with Large Amounts ===");
    println!("Token A Amount: {}", token_a_amount);
    println!("Token B Amount: {}", token_b_amount);

    // Verify initial balances
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "initial")?;

    // Create escrow with large amounts
    setup.create_escrow(EscrowType::Simple, token_a_amount, token_b_amount)?;

    // Verify balances after creation
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_creation")?;

    // Take the escrow
    setup.take_escrow()?;

    // Verify balances after take
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_take")?;

    println!("✅ Simple escrow with large amounts test passed");
    Ok(())
}

#[test]
fn test_simple_escrow_equal_amounts() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let token_a_amount = 3000;
    let token_b_amount = 3000;

    println!("=== Testing Simple Escrow with Equal Amounts ===");
    println!("Token A Amount: {}", token_a_amount);
    println!("Token B Amount: {}", token_b_amount);

    // Verify initial balances
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "initial")?;

    // Create escrow with equal amounts
    setup.create_escrow(EscrowType::Simple, token_a_amount, token_b_amount)?;

    // Verify balances after creation
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_creation")?;

    // Take the escrow
    setup.take_escrow()?;

    // Verify balances after take
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_take")?;

    println!("✅ Simple escrow with equal amounts test passed");
    Ok(())
}

#[test]
fn test_simple_escrow_maximum_amounts() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let token_a_amount = 10000; // Maximum available
    let token_b_amount = 10000; // Maximum available

    println!("=== Testing Simple Escrow with Maximum Amounts ===");
    println!("Token A Amount: {}", token_a_amount);
    println!("Token B Amount: {}", token_b_amount);

    // Verify initial balances
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "initial")?;

    // Create escrow with maximum amounts
    setup.create_escrow(EscrowType::Simple, token_a_amount, token_b_amount)?;

    // Verify balances after creation
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_creation")?;

    // Take the escrow
    setup.take_escrow()?;

    // Verify balances after take
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_take")?;

    println!("✅ Simple escrow with maximum amounts test passed");
    Ok(())
}

#[test]
fn test_simple_escrow_multiple_escrows() -> Result<()> {
    println!("=== Testing Multiple Simple Escrows ===");

    // First escrow
    let mut setup1 = EscrowTestSetup::new()?;
    let token_a_amount_1 = 2000;
    let token_b_amount_1 = 4000;

    println!(
        "First escrow - Token A: {}, Token B: {}",
        token_a_amount_1, token_b_amount_1
    );

    setup1.verify_simple_escrow_balances(token_a_amount_1, token_b_amount_1, "initial")?;
    setup1.create_escrow(EscrowType::Simple, token_a_amount_1, token_b_amount_1)?;
    setup1.verify_simple_escrow_balances(token_a_amount_1, token_b_amount_1, "after_creation")?;
    setup1.take_escrow()?;
    setup1.verify_simple_escrow_balances(token_a_amount_1, token_b_amount_1, "after_take")?;

    // Second escrow (with new setup to avoid seed conflicts)
    let mut setup2 = EscrowTestSetup::new()?;
    let token_a_amount_2 = 1500;
    let token_b_amount_2 = 3000;

    println!(
        "Second escrow - Token A: {}, Token B: {}",
        token_a_amount_2, token_b_amount_2
    );

    setup2.verify_simple_escrow_balances(token_a_amount_2, token_b_amount_2, "initial")?;
    setup2.create_escrow(EscrowType::Simple, token_a_amount_2, token_b_amount_2)?;
    setup2.verify_simple_escrow_balances(token_a_amount_2, token_b_amount_2, "after_creation")?;
    setup2.take_escrow()?;
    setup2.verify_simple_escrow_balances(token_a_amount_2, token_b_amount_2, "after_take")?;

    println!("✅ Multiple simple escrows test passed");
    Ok(())
}
