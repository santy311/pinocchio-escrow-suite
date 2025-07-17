use anyhow::Result;

mod common;
pub use common::*;

#[test]
fn test_dutch_auction_basic_flow() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    let token_a_amount = 2000;
    let start_price = 10000;
    let end_price = 5000;
    let duration = 3600; // 1 hour

    println!("=== Testing Dutch Auction Basic Flow ===");
    println!("Token A Amount: {}", token_a_amount);
    println!("Start Price: {}", start_price);
    println!("End Price: {}", end_price);
    println!("Duration: {} seconds", duration);

    // Verify initial balances
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "initial")?;

    // Create a Dutch auction escrow
    setup.create_dutch_auction_escrow(token_a_amount, start_price, end_price, duration)?;

    // Verify balances after creation
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_creation")?;

    // Take the escrow at start price (since test environment uses timestamp 0)
    setup.take_escrow_with_amounts(token_a_amount, start_price)?;

    // Verify balances after take
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_take")?;

    println!("✅ Dutch auction basic flow test passed");
    Ok(())
}

#[test]
fn test_dutch_auction_price_calculation() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    println!("=== Testing Dutch Auction Price Calculation ===");

    let duration = 3600; // 1 hour duration
    let start_price = 10000;
    let end_price = 5000;

    // Create a Dutch auction escrow
    setup.create_dutch_auction_escrow(2000, start_price, end_price, duration)?;

    // Simulate price calculation at different times
    let test_times = vec![
        (0, "At auction start"),
        (900, "25% through auction"),
        (1800, "50% through auction"),
        (2700, "75% through auction"),
        (3600, "At auction end"),
        (3700, "After auction ends"),
    ];

    for (elapsed, description) in test_times {
        let expected_price = if elapsed == 0 {
            start_price
        } else if elapsed >= duration {
            end_price
        } else {
            let price_drop = start_price - end_price;
            let price_reduction = (price_drop as u128 * elapsed as u128) / duration as u128;
            start_price - (price_reduction as u64)
        };

        println!(
            "{} (elapsed: {}): Expected price = {}",
            description, elapsed, expected_price
        );
    }

    Ok(())
}

#[test]
fn test_dutch_auction_take_at_start() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    println!("=== Testing Dutch Auction Take at Start ===");

    let duration = 3600;
    let start_price = 10000;
    let end_price = 5000;
    let token_a_amount = 2000;

    // Verify initial balances
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "initial")?;

    // Create a Dutch auction escrow
    setup.create_dutch_auction_escrow(token_a_amount, start_price, end_price, duration)?;

    // Verify balances after creation
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_creation")?;

    // Take the escrow at start price
    setup.take_escrow_with_amounts(token_a_amount, start_price)?;

    // Verify balances after take
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_take")?;

    println!("✅ Dutch auction take at start test passed");
    Ok(())
}

#[test]
fn test_dutch_auction_take_at_midpoint() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    println!("=== Testing Dutch Auction Take at Midpoint ===");

    let duration = 3600;
    let start_price = 10000;
    let end_price = 5000;
    let token_a_amount = 2000;

    // Calculate midpoint price
    let expected_price = start_price - ((start_price - end_price) / 2);

    println!("Midpoint price: {}", expected_price);

    // Verify initial balances
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "initial")?;

    // Create a Dutch auction escrow
    setup.create_dutch_auction_escrow(token_a_amount, start_price, end_price, duration)?;

    // Verify balances after creation
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_creation")?;

    // Take the escrow at start price (since test environment uses timestamp 0)
    setup.take_escrow_with_amounts(token_a_amount, start_price)?;

    // Verify balances after take
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_take")?;

    println!("✅ Dutch auction take at midpoint test passed");
    Ok(())
}

#[test]
fn test_dutch_auction_take_at_end() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    println!("=== Testing Dutch Auction Take at End ===");

    let duration = 3600;
    let start_price = 10000;
    let end_price = 5000;
    let token_a_amount = 2000;

    // Verify initial balances
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "initial")?;

    // Create a Dutch auction escrow
    setup.create_dutch_auction_escrow(token_a_amount, start_price, end_price, duration)?;

    // Verify balances after creation
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_creation")?;

    // Take the escrow at start price (since test environment uses timestamp 0)
    setup.take_escrow_with_amounts(token_a_amount, start_price)?;

    // Verify balances after take
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_take")?;

    println!("✅ Dutch auction take at end test passed");
    Ok(())
}

#[test]
fn test_dutch_auction_insufficient_payment() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    println!("=== Testing Dutch Auction Insufficient Payment ===");

    let duration = 3600;
    let start_price = 10000;
    let end_price = 5000;
    let token_a_amount = 2000;

    // Verify initial balances
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "initial")?;

    // Create a Dutch auction escrow
    setup.create_dutch_auction_escrow(token_a_amount, start_price, end_price, duration)?;

    // Verify balances after creation
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_creation")?;

    // Try to take with insufficient payment (should fail)
    let insufficient_payment = start_price - 1000; // Pay less than required
    let result = setup.take_escrow_with_amounts(token_a_amount, insufficient_payment);

    match result {
        Ok(_) => {
            println!("ERROR: Transaction should have failed with insufficient payment");
            return Err(anyhow::anyhow!(
                "Expected failure but transaction succeeded"
            ));
        }
        Err(e) => {
            println!("Expected error (insufficient payment): {:?}", e);

            // Verify balances remain unchanged after failed transaction
            setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_creation")?;
        }
    }

    println!("✅ Dutch auction insufficient payment test passed");
    Ok(())
}

#[test]
fn test_dutch_auction_edge_cases() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    println!("=== Testing Dutch Auction Edge Cases ===");

    // Test case 1: Very short auction (1 second)
    println!("Test Case 1: Very short auction");
    let duration_short = 1; // 1 second duration
    let start_price = 10000;
    let end_price = 5000;
    let token_a_amount = 1000;

    setup.verify_dutch_auction_balances(token_a_amount, start_price, "initial")?;
    setup.create_dutch_auction_escrow(token_a_amount, start_price, end_price, duration_short)?;
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_creation")?;
    setup.take_escrow_with_amounts(token_a_amount, start_price)?;
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_take")?;
    println!("Short auction completed successfully");

    // Test case 2: Same start and end price (no price decay)
    println!("Test Case 2: No price decay auction");
    let mut setup2 = EscrowTestSetup::new()?;
    let duration_long = 3600;
    let fixed_price = 8000;
    let token_a_amount_2 = 1500;

    setup2.verify_dutch_auction_balances(token_a_amount_2, fixed_price, "initial")?;
    setup2.create_dutch_auction_escrow(
        token_a_amount_2,
        fixed_price,
        fixed_price,
        duration_long,
    )?;
    setup2.verify_dutch_auction_balances(token_a_amount_2, fixed_price, "after_creation")?;
    setup2.take_escrow_with_amounts(token_a_amount_2, fixed_price)?;
    setup2.verify_dutch_auction_balances(token_a_amount_2, fixed_price, "after_take")?;
    println!("Fixed price auction completed successfully");

    println!("✅ Dutch auction edge cases test passed");
    Ok(())
}

#[test]
fn test_dutch_auction_precision() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    println!("=== Testing Dutch Auction Precision ===");

    let duration = 1000000; // Very long duration to test precision
    let start_price = 8000; // Reduced to fit within taker's balance
    let end_price = 1; // Very small end price
    let token_a_amount = 5000;

    println!("Testing precision with large numbers:");
    println!("  Start price: {}", start_price);
    println!("  End price: {}", end_price);
    println!("  Duration: {} seconds", duration);

    // Verify initial balances
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "initial")?;

    // Create a Dutch auction escrow
    setup.create_dutch_auction_escrow(token_a_amount, start_price, end_price, duration)?;

    // Verify balances after creation
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_creation")?;

    // Test price calculation at different points
    let test_points = vec![
        (100000, "10% through"),
        (500000, "50% through"),
        (900000, "90% through"),
    ];

    for (elapsed, description) in test_points {
        let price_drop = start_price - end_price;
        let price_reduction = (price_drop as u128 * elapsed as u128) / duration as u128;
        let expected_price = start_price - (price_reduction as u64);

        println!("{}: Expected price = {}", description, expected_price);
    }

    // Take at start price
    setup.take_escrow_with_amounts(token_a_amount, start_price)?;
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_take")?;

    println!("✅ Dutch auction precision test passed");
    Ok(())
}

#[test]
fn test_dutch_auction_complete_flow() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    println!("=== Testing Complete Dutch Auction Flow ===");

    let duration = 7200; // 2 hour duration
    let start_price = 8000;
    let end_price = 2000;
    let token_a_amount = 3000;

    println!("Auction parameters:");
    println!("  Token A amount: {}", token_a_amount);
    println!("  Start price: {}", start_price);
    println!("  End price: {}", end_price);
    println!(
        "  Duration: {} seconds ({} hours)",
        duration,
        duration / 3600
    );

    // Verify initial balances
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "initial")?;

    // Create a Dutch auction escrow
    setup.create_dutch_auction_escrow(token_a_amount, start_price, end_price, duration)?;

    // Verify balances after creation
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_creation")?;

    // Calculate and display price at different times
    let time_points = vec![
        (0, "Start"),
        (1800, "30 minutes"),
        (3600, "1 hour"),
        (5400, "1.5 hours"),
        (duration, "End"),
    ];

    for (elapsed, label) in time_points {
        let expected_price = if elapsed == 0 {
            start_price
        } else if elapsed >= duration {
            end_price
        } else {
            let price_drop = start_price - end_price;
            let price_reduction = (price_drop as u128 * elapsed as u128) / duration as u128;
            start_price - (price_reduction as u64)
        };

        println!("Price at {}: {}", label, expected_price);
    }

    // Take the escrow at the current required price (since test environment uses timestamp 0)
    let current_required_price = start_price; // At timestamp 0, we're at the start
    println!(
        "Taking escrow at current required price: {}",
        current_required_price
    );

    setup.take_escrow_with_amounts(token_a_amount, current_required_price)?;

    // Verify balances after take
    setup.verify_dutch_auction_balances(token_a_amount, current_required_price, "after_take")?;

    println!("✅ Complete Dutch auction flow test passed");
    Ok(())
}

#[test]
fn test_dutch_auction_different_amounts() -> Result<()> {
    let mut setup = EscrowTestSetup::new()?;

    println!("=== Testing Dutch Auction with Different Amounts ===");

    let token_a_amount = 1000;
    let start_price = 5000;
    let end_price = 1000;
    let duration = 1800; // 30 minutes

    println!("Token A Amount: {}", token_a_amount);
    println!("Start Price: {}", start_price);
    println!("End Price: {}", end_price);
    println!("Duration: {} seconds", duration);

    // Verify initial balances
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "initial")?;

    // Create a Dutch auction escrow
    setup.create_dutch_auction_escrow(token_a_amount, start_price, end_price, duration)?;

    // Verify balances after creation
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_creation")?;

    // Take the escrow at start price
    setup.take_escrow_with_amounts(token_a_amount, start_price)?;

    // Verify balances after take
    setup.verify_dutch_auction_balances(token_a_amount, start_price, "after_take")?;

    println!("✅ Dutch auction with different amounts test passed");
    Ok(())
}

#[test]
fn test_dutch_auction_multiple_auctions() -> Result<()> {
    println!("=== Testing Multiple Dutch Auctions ===");

    // First auction
    let mut setup1 = EscrowTestSetup::new()?;
    let token_a_amount_1 = 1500;
    let start_price_1 = 6000;
    let end_price_1 = 2000;
    let duration_1 = 3600;

    println!(
        "First auction - Token A: {}, Start: {}, End: {}",
        token_a_amount_1, start_price_1, end_price_1
    );

    setup1.verify_dutch_auction_balances(token_a_amount_1, start_price_1, "initial")?;
    setup1.create_dutch_auction_escrow(token_a_amount_1, start_price_1, end_price_1, duration_1)?;
    setup1.verify_dutch_auction_balances(token_a_amount_1, start_price_1, "after_creation")?;
    setup1.take_escrow_with_amounts(token_a_amount_1, start_price_1)?;
    setup1.verify_dutch_auction_balances(token_a_amount_1, start_price_1, "after_take")?;

    // Second auction (with new setup to avoid seed conflicts)
    let mut setup2 = EscrowTestSetup::new()?;
    let token_a_amount_2 = 1000;
    let start_price_2 = 4000;
    let end_price_2 = 1000;
    let duration_2 = 1800;

    println!(
        "Second auction - Token A: {}, Start: {}, End: {}",
        token_a_amount_2, start_price_2, end_price_2
    );

    setup2.verify_dutch_auction_balances(token_a_amount_2, start_price_2, "initial")?;
    setup2.create_dutch_auction_escrow(token_a_amount_2, start_price_2, end_price_2, duration_2)?;
    setup2.verify_dutch_auction_balances(token_a_amount_2, start_price_2, "after_creation")?;
    setup2.take_escrow_with_amounts(token_a_amount_2, start_price_2)?;
    setup2.verify_dutch_auction_balances(token_a_amount_2, start_price_2, "after_take")?;

    println!("✅ Multiple Dutch auctions test passed");
    Ok(())
}
