# Deployment Guide - Solana Crowdfunding

This guide covers setting up your environment, deploying the program to devnet, and running tests.

---

## Prerequisites

### 1. Install Required Tools

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL "https://release.solana.com/v1.18.26/install")"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install latest
avm use latest

# Verify installations
solana --version
anchor --version
rustc --version
```

### 2. Setup Phantom Wallet

1. **Download Phantom Wallet** from [phantom.app](https://phantom.app)
2. **Create new wallet** or import existing
3. **Switch to Devnet**: Click network selector → Select "Devnet"
4. **Get wallet address**: Click to copy your public key

### 3. Configure Wallet Path in Anchor.toml

```toml
[provider]
cluster = "devnet"
wallet = "~/.config/solana/id.json"  # Update if needed
```

**Alternative:** Use Phantom's secret key:

```bash
# Import from Phantom (export private key -> base58 -> file)
solana config set --keypair /path/to/keypair.json
```

---

## Getting SOL on Devnet

### Option 1: CLI Airdrop (Recommended for testing)

```bash
# Get your wallet address
solana address

# Airdrop SOL (max 2 SOL per request on devnet)
solana airdrop 2 <YOUR_WALLET_ADDRESS> --url devnet

# Verify balance
solana balance <YOUR_WALLET_ADDRESS> --url devnet
```

**Note:** Each airdrop is limited to 2 SOL. You may need multiple airdrops for testing.

### Option 2: Phantom Faucet

1. Open Phantom Wallet
2. Go to Settings → Developer Settings
3. Enable "Show Testnet"
4. Click "Devnet" → "Request Airdrop"

---

## Building the Program

```bash
# Navigate to project directory
cd mancer-submission

# Build the program
anchor build

# Expected output:
# Building...
# Success
```

If you encounter errors:

```bash
# Clean and rebuild
anchor clean
anchor build
```

---

## Deploying to Devnet

```bash
# Deploy program
anchor deploy --provider.cluster devnet

# Expected output:
# Deploying program 'mancer_submission'...
# Program Id: <YOUR_PROGRAM_ID>
# Signature: <TRANSACTION_SIGNATURE>
# Successfully deployed program.
```

**Important:** Save your Program ID - you'll need it for:

- Updating Anchor.toml
- Tests
- Submission

---

## Verifying Deployment

```bash
# Check program on chain
solana program show <PROGRAM_ID> --url devnet

# Expected output shows:
# Program Id: <PROGRAM_ID>
# Owner: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss613VQ6
# ProgramData Address: ...
# Last Deployed Slot: ...
```

---

## Running Tests

### Local Tests (Using Validator)

```bash
# Start local validator and run tests
anchor test

# Or with specific options
anchor test --skip-local-validator
```

### Devnet Tests

```bash
# Run tests against devnet
anchor test --provider.cluster devnet
```

### Reading Test Output

```
mancer-submission
  ✔ Create campaign (400ms)
  ✔ Contribute to campaign (400ms)
  ✔ Withdraw after successful campaign (400ms)
  ✔ Refund after failed campaign (400ms)

  4 passing (2s)
```

---

## Common Issues & Solutions

### Issue 1: "Insufficient funds"

```bash
# Airdrop more SOL
solana airdrop 2 <ADDRESS> --url devnet
```

### Issue 2: "Program already deployed"

```bash
# You need to upgrade if Program ID same
# First, get current program
solana program show <PROGRAM_ID>

# Deploy with upgrade authority
anchor deploy --provider.cluster devnet
# Or force redeploy (creates new Program ID)
anchor deploy --provider.cluster devnet --program-name mancer_submission
```

### Issue 3: "Connection refused"

```bash
# Check cluster config
solana config get

# Set to devnet
solana config set --url devnet
```

### Issue 4: "Wallet not found"

```bash
# Check wallet path
solana config get

# Update config
solana config set --keypair ~/.config/solana/id.json
```

---

## Updating Program ID

After deploying, update `lib.rs`:

```rust
// Before
declare_id!("2bvT3M5bLbJgk8dcm3jsDkSEn8B2ntk2Eokmt2UKb7pU");

// After (use the new Program ID from deployment)
declare_id!("YourNewProgramIdHere...");
```

Also update `Anchor.toml`:

```toml
[programs.devnet]
mancer_submission = "YourNewProgramIdHere..."
```

---

## Test Transaction Signatures

After running tests, you'll get transaction signatures. Save these for submission:

```bash
# Run tests with verbose output
anchor test --provider.cluster devnet -v

# Look for:
# Your transaction signature: <SIGNATURE>
```

### Verifying Transactions

```bash
# View transaction details
solana confirm <TRANSACTION_SIGNATURE> --url devnet

# Get transaction info
solana transaction <TRANSACTION_SIGNATURE> --url devnet
```

---

## Checklist

- [ ] Rust installed
- [ ] Solana CLI installed
- [ ] Anchor installed
- [ ] Wallet configured
- [ ] SOL airdropped (devnet)
- [ ] Program builds successfully
- [ ] Program deployed to devnet
- [ ] Program ID recorded
- [ ] Tests pass
- [ ] Transaction signatures collected

---

## Production Checklist

Before submitting, ensure:

- [ ] All 4 instructions implemented
- [ ] Tests pass for all scenarios
- [ ] Program ID matches config
- [ ] Documentation complete
- [ ] Transaction signatures available

---

## Next Steps

Now that deployment is working:

1. Review `010_submission-spec.md` for requirements
2. Implement the crowdfunding program
3. Write comprehensive tests
4. Deploy and verify everything works
