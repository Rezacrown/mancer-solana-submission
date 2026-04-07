# AGENTS.md - Solana Crowdfunding Project

## Project Overview

Solana crowdfunding smart contract built with Anchor framework.

## Documentation

All documentation in `docs/` folder - reference these files for details:

| File                           | Description         |
| ------------------------------ | ------------------- |
| `docs/000_project-plan.md`     | Project milestones  |
| `docs/010_submission-spec.md`  | Task requirements   |
| `docs/011_solana-basics.md`    | Solana fundamentals |
| `docs/012_anchor-primer.md`    | Anchor framework    |
| `docs/013_pda-explained.md`    | PDAs deep dive      |
| `docs/014_deployment-guide.md` | Deployment steps    |

## Quick Reference

- **Program ID:** `2bvT3M5bLbJgk8dcm3jsDkSEn8B2ntk2Eokmt2UKb7pU`
- **Program:** `programs/mancer-submission/src/lib.rs`
- **Tests:** `tests/mancer-submission.ts`

## Commands (Run in WSL)

```bash
# Install dependencies
cd /path/to/mancer-submission
npm install

# Build program
anchor build

# Run tests locally
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Check balance
solana balance <WALLET_ADDRESS> --url devnet

# Airdrop SOL
solana airdrop 2 <WALLET_ADDRESS> --url devnet
```

## General Rules

1. **Always read existing code before editing** - Never modify files without reading them first

2. **Ensure to using skill solana-dev** - Don't make ansumtion about the code use this skill for helping development from offical solana

3. **Verify changes compile** - Run `anchor build` before claiming work is complete

4. **Don't guess dependencies** - Check `Cargo.toml` and `package.json` for available libraries

5. **Use Anchor conventions** - Follow the patterns in `lib.rs` for new instructions

6. **Amounts are in lamports** - 1 SOL = 1,000,000,000 lamports

7. **PDAs use seeds + bump** - Always use `invoke_signed` when transferring from PDA

8. **Check errors match actual code** - Don't claim error messages that don't exist in the program

9. **Always include documentation** - When write the code make sure to include documentation or comment for the context what your build
