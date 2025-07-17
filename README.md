# Escrow Suite

A comprehensive Solana program built with Pinocchio framework that implements various types of escrow mechanisms for secure token trading.

## Overview

Escrow Suite is a decentralized escrow system that supports multiple escrow types:

- **Simple Escrow**: Traditional fixed-price token exchange
- **Partial Escrow**: Allows partial fulfillment of escrow orders
- **Dutch Auction**: Time-based declining price mechanism
- **Oracle Escrow**: Price-feed based escrow (planned feature)

## Features

### 🔒 Simple Escrow

- Fixed exchange rate between two tokens
- All-or-nothing execution
- Secure token transfer using PDA (Program Derived Address)

### 📊 Partial Escrow

- Partial fulfillment of escrow orders
- Proportional token exchange based on requested amount
- Flexible trading for large orders

### ⏰ Dutch Auction

- Time-based declining price mechanism
- Linear price decay over specified duration
- Automatic price calculation based on current time
- Configurable start price, end price, and auction duration

## Program Architecture

The program uses Pinocchio framework and consists of:

- **Entry Point**: `process_instruction` handles all program calls
- **Instructions**:
  - `make_escrow` (0x01): Creates new escrow orders
  - `take_escrow` (0x02): Executes escrow trades
- **States**: `Escrow` struct manages escrow data and logic
- **Error Handling**: Comprehensive error codes for validation

## Building and Testing

### Prerequisites

- Rust toolchain
- Cargo
- Pinocchio framework dependencies

### Build the Program

```bash
# Build for Solana BPF
cargo build-sbf
```

### Run Tests

```bash
# Run all tests with output
cargo test -- --nocapture
```

### Test Coverage

The test suite includes comprehensive tests for:

- **Simple Escrow Tests** (`tests/simple_escrow.rs`)

  - Basic flow validation
  - Different token amounts
  - Edge cases and error conditions
  - Multiple escrow scenarios

- **Partial Escrow Tests** (`tests/partial_escrow.rs`)

  - Partial fulfillment logic
  - Proportional calculations
  - Balance verification

- **Dutch Auction Tests** (`tests/dutch_auction.rs`)

  - Time-based price calculation
  - Auction duration validation
  - Price decay mechanisms

- **Unit Tests** (`tests/unit.rs`)
  - Individual component testing
  - Data structure validation

## Program ID

```
N9BuK6SmDXHr2jpca1C4WzMhok2wki8sx2osK1sTobc
```

## Usage Examples

### Creating a Simple Escrow

```rust
// Create instruction data for simple escrow
let make_ix = MakeEscrowIx::new(
    EscrowType::Simple,
    token_a_amount,  // Amount of token A to offer
    token_b_amount,  // Amount of token B requested
    bump,           // PDA bump
    seed,           // Unique seed
);
```

### Creating a Dutch Auction

```rust
// Create instruction data for Dutch auction
let make_ix = MakeEscrowIx::new_dutch_auction(
    token_a_amount,  // Amount of token A to offer
    start_price,     // Initial price in token B
    end_price,       // Final price in token B
    start_time,      // Auction start time
    end_time,        // Auction end time
    bump,           // PDA bump
    seed,           // Unique seed
);
```

### Taking an Escrow

```rust
// Create instruction data for taking escrow
let take_ix = TakeEscrowIx::new(
    escrow_type,     // Type of escrow being taken
    token_a_amount,  // Amount of token A to receive
    token_b_amount,  // Amount of token B to pay
);
```

## Security Features

- **PDA Validation**: All escrow accounts use Program Derived Addresses
- **Signer Verification**: Ensures only authorized parties can execute trades
- **Token Ownership Checks**: Validates token account ownership
- **Balance Verification**: Prevents insufficient fund transfers
- **Time-based Validation**: Dutch auctions respect time constraints

## Error Handling

The program includes comprehensive error codes:

- `InvalidMaker`: Unauthorized account attempting operation
- `EscrowAlreadyExists`: Duplicate escrow creation attempt
- `InvalidTokenOwner`: Incorrect token account ownership
- `InsufficientFunds`: Insufficient token balance for operation
- `PdaMismatch`: Program Derived Address validation failure
- `InvalidEscrowType`: Unsupported escrow type

## Development

### Project Structure

```
src/
├── lib.rs              # Program entry point and main logic
├── error.rs            # Error definitions
├── instructions/       # Instruction handlers
│   ├── make.rs        # Escrow creation logic
│   ├── take.rs        # Escrow execution logic
│   └── mod.rs         # Module exports
└── states/            # Data structures
    ├── escrows.rs     # Escrow state and logic
    ├── utils.rs       # Utility functions
    └── mod.rs         # Module exports
```

### Adding New Features

1. Define new escrow type in `EscrowType` enum
2. Implement logic in `make_escrow` and `take_escrow` functions
3. Add corresponding test cases
4. Update documentation

## License

This project is licensed under the MIT License.
