# Submission Deliverables

## 1. Rust Program Code

**Location:** `programs/mancer-submission/src/`

| File                 | Description                                                               |
| -------------------- | ------------------------------------------------------------------------- |
| `lib.rs`             | Main program - 4 functions: create_campaign, contribute, withdraw, refund |
| `errors.rs`          | Custom error codes (8 errors)                                             |
| `campaign.rs`        | Campaign account struct                                                   |
| `contribution.rs`    | Contribution tracking struct                                              |
| `accounts_struct.rs` | Account validation contexts                                               |

---

## 2. Deployed Program

### Devnet (DEPLOYED ✅)

| Item             | Value                                                                                      |
| ---------------- | ------------------------------------------------------------------------------------------ |
| Program ID       | `2bvT3M5bLbJgk8dcm3jsDkSEn8B2ntk2Eokmt2UKb7pU`                                             |
| Deployer Account | `7mCXNFj4N3X7Hax9FMWYBCeGpN89Qs5NrtU82dtm7Ye5`                                             |
| Deploy Signature | `2huRwu5g3CYvVbhSGyARyKoZBWrhXCfpHEDKDHBCjvECYo7Z1hFvKPXTHr2moBCebZxbDAuNZxPLo29KAA9o4tHe` |
| IDL Account      | `9pvvyc1HubSQtjwQ3xGGV451zjtvP23bexawjUTrWQ4b`                                             |
| Cluster          | Devnet                                                                                     |

### Localnet (Testing)

- **Program ID:** `2bvT3M5bLbJgk8dcm3jsDkSEn8B2ntk2Eokmt2UKb7pU`

Run these commands:

```bash
# Build
anchor build

# Deploy
anchor deploy --provider.cluster devnet
```

---

## 3. Test Transaction Signatures

Run tests and save the transaction hashes:

```bash
anchor test
```

### Expected Tests (6 total):

| #   | Test Name                                          | Expected Result |
| --- | -------------------------------------------------- | --------------- |
| 1   | Create campaign (goal=1000 SOL, deadline=tomorrow) | ✅ Pass         |
| 2   | Contribute 600 SOL                                 | ✅ Pass         |
| 3   | Contribute 500 SOL (total=1100)                    | ✅ Pass         |
| 4   | Withdraw before deadline                           | ✅ Rejected     |
| 5   | Withdraw after deadline                            | ✅ Pass         |
| 6   | Double withdrawal                                  | ✅ Rejected     |

**Transaction signatures will appear in test output like:**

```
Create campaign transaction: 58CHDChV4VK7MVLYP2SL1MkpccVB33bSQRrxnh2GgZHR...
Contribute transaction: 5mep8JECwUGDVHXtvwqA9B2gNzhHDD9his8pJE5mDFQTm...
```

---

## Quick Checklist

- [ ] Rust code - ready in `programs/mancer-submission/src/`
- [ ] Deploy to devnet - run `anchor deploy --provider.cluster devnet`
- [ ] Save new Program ID
- [ ] Run `anchor test` and save transaction signatures
- [ ] Update Program ID in Anchor.toml if needed

---

## Notes

- Amounts in lamports: 1 SOL = 1,000,000,000 lamports
- Program uses PDAs for vault and contribution tracking
- All 6 tests pass on localnet
