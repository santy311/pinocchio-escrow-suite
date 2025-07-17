use anyhow::Result;
use escrow_suite::states::EscrowType;

mod common;
pub use common::*;

// ==================== ORACLE ESCROW TESTS ====================

#[test]
fn test_oracle_escrow() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let token_a_amount = 3000;
    let token_b_amount = 8000;

    println!("=== Testing Oracle Escrow ===");
    println!("Token A Amount: {}", token_a_amount);
    println!("Token B Amount: {}", token_b_amount);

    // Verify initial balances
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "initial")?;

    // Create an oracle escrow
    setup.create_escrow(EscrowType::Oracle, token_a_amount, token_b_amount)?;

    // Verify balances after creation
    setup.verify_simple_escrow_balances(token_a_amount, token_b_amount, "after_creation")?;

    // Note: Oracle escrow take logic would need to be implemented
    // For now, this test just verifies the escrow creation works
    println!("✅ Oracle escrow created successfully");

    Ok(())
}

#[test]
fn test_escrow_scenarios() -> Result<()> {
    println!("=== Testing Escrow Scenarios ===");

    // Test simple escrow with complete flow
    println!("Testing Simple Escrow Scenario");
    EscrowTestSetup::run_complete_escrow_test(EscrowType::Simple, 2500, 7500, true)?;

    // Test oracle escrow (creation only)
    println!("Testing Oracle Escrow Scenario");
    EscrowTestSetup::run_complete_escrow_test(EscrowType::Oracle, 1500, 4500, false)?;

    // Test Dutch auction escrow (creation only)
    println!("Testing Dutch Auction Escrow Scenario");
    EscrowTestSetup::run_complete_escrow_test(EscrowType::DutchAuction, 800, 3200, false)?;

    // Test partial escrow (creation only)
    println!("Testing Partial Escrow Scenario");
    EscrowTestSetup::run_complete_escrow_test(EscrowType::Partial, 1200, 3600, false)?;

    println!("✅ All escrow scenarios test passed");
    Ok(())
}
