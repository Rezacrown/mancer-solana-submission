import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MancerSubmission } from "../target/types/mancer_submission";
import { assert } from "chai";

describe("mancer-submission", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .mancerSubmission as Program<MancerSubmission>;

  const creator = anchor.Wallet.local().payer;
  const creatorPublicKey = creator.publicKey;

  let mainCampaignAddress: anchor.web3.PublicKey;
  let mainVaultAddress: anchor.web3.PublicKey;

  // Generate a SECOND keypair for testing withdraw (to avoid address collision)
  const testCreator = anchor.web3.Keypair.generate();
  const testCreatorPublicKey = testCreator.publicKey;

  const GOAL = 1000 * 1_000_000_000; // 1000 SOL in lamports
  const CONTRIBUTION_1 = 600 * 1_000_000_000; // 600 SOL
  const CONTRIBUTION_2 = 500 * 1_000_000_000; // 500 SOL

  // =========================================================================
  // SETUP: Airdrop to test creator for withdraw tests
  // =========================================================================
  before(async () => {
    // Airdrop SOL to test creator for withdraw test (need enough for contribution + dummy txs)
    const airdropTx = await program.provider.connection.requestAirdrop(
      testCreatorPublicKey,
      2000 * 1_000_000_000, // 20 SOL (need for contribution + slot advance txs)
    );
    await program.provider.connection.confirmTransaction(airdropTx);
  });

  // =========================================================================
  // TEST 1: Create campaign with goal=1000 SOL, deadline=tomorrow
  // =========================================================================
  it("Create campaign with goal=1000 SOL, deadline=tomorrow", async () => {
    const deadline = Math.floor(Date.now() / 1000) + 86400; // Tomorrow

    const tx = await program.methods
      .createCampaign(new anchor.BN(GOAL), new anchor.BN(deadline))
      .accounts({
        creator: creatorPublicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Create campaign transaction:", tx);

    // Derive campaign address: [b"campaign", creator]
    const [campaignAddr] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("campaign"), creatorPublicKey.toBuffer()],
      program.programId,
    );
    mainCampaignAddress = campaignAddr;

    // Derive vault address: [b"vault", campaign]
    const [vaultAddr] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), campaignAddr.toBuffer()],
      program.programId,
    );
    mainVaultAddress = vaultAddr;

    // Verify campaign data
    const campaign = await program.account.campaign.fetch(mainCampaignAddress);

    assert.equal(campaign.creator.toString(), creatorPublicKey.toString());
    assert.equal(campaign.goal.toString(), GOAL.toString());
    assert.equal(campaign.raised.toString(), "0");
    assert.equal(campaign.deadline.toString(), deadline.toString());
    assert.equal(campaign.claimed, false);

    console.log("Campaign created successfully!");
    console.log("  Campaign address:", mainCampaignAddress.toString());
    console.log("  Vault address:", mainVaultAddress.toString());
    console.log("  Goal:", GOAL / 1_000_000_000, "SOL");
    console.log("  Deadline:", new Date(deadline * 1000).toLocaleString());
  });

  // =========================================================================
  // TEST 2: Contribute 600 SOL → should succeed, raised=600
  // =========================================================================
  it("Contribute 600 SOL → raised=600", async () => {
    const tx = await program.methods
      .contribute(new anchor.BN(CONTRIBUTION_1))
      .accounts({
        donor: creatorPublicKey,
        campaign: mainCampaignAddress,
        vault: mainVaultAddress,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Contribute transaction:", tx);

    const campaign = await program.account.campaign.fetch(mainCampaignAddress);
    assert.equal(campaign.raised.toString(), CONTRIBUTION_1.toString());

    console.log("First contribution successful!");
    console.log("  Amount:", CONTRIBUTION_1 / 1_000_000_000, "SOL");
    console.log(
      "  Total raised:",
      campaign.raised.toNumber() / 1_000_000_000,
      "SOL",
    );
  });

  // =========================================================================
  // TEST 3: Contribute 500 SOL → should succeed, raised=1100
  // =========================================================================
  it("Contribute 500 SOL → raised=1100", async () => {
    const tx = await program.methods
      .contribute(new anchor.BN(CONTRIBUTION_2))
      .accounts({
        donor: creatorPublicKey,
        campaign: mainCampaignAddress,
        vault: mainVaultAddress,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Second contribute transaction:", tx);

    const campaign = await program.account.campaign.fetch(mainCampaignAddress);
    const totalExpected = CONTRIBUTION_1 + CONTRIBUTION_2;
    assert.equal(campaign.raised.toString(), totalExpected.toString());

    console.log("Second contribution successful!");
    console.log("  Amount:", CONTRIBUTION_2 / 1_000_000_000, "SOL");
    console.log(
      "  Total raised:",
      campaign.raised.toNumber() / 1_000_000_000,
      "SOL",
    );
  });

  // =========================================================================
  // TEST 4: Try withdraw before deadline → should fail
  // =========================================================================
  it("Try withdraw before deadline → should fail", async () => {
    try {
      await program.methods
        .withdraw()
        .accounts({
          creator: creatorPublicKey,
          campaign: mainCampaignAddress,
          vault: mainVaultAddress,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      assert.fail("Should have thrown error");
    } catch (error) {
      console.log("Correctly rejected withdraw before deadline");
      assert.include(error.toString(), "Campaign deadline has not passed yet");
    }
  });

  // =========================================================================
  // Helper: Advance slots in localnet by creating dummy transfer transactions
  // This simulates time passing in the test validator
  // =========================================================================
  async function advanceClockBy(seconds: number) {
    const connection = program.provider.connection as any;
    const slotsToAdvance = Math.ceil(seconds * 2);
    for (let i = 0; i < slotsToAdvance; i++) {
      const tx = new anchor.web3.Transaction();
      tx.add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: testCreatorPublicKey,
          toPubkey: testCreatorPublicKey,
          lamports: 1,
        }),
      );
      await connection.sendTransaction(tx, [testCreator], {
        skipPreflight: true,
      });
    }
    console.log(`Advanced ${slotsToAdvance} slots`);
  }

  // =========================================================================
  // TEST 5: Wait until after deadline → withdraw should succeed
  // Use DIFFERENT creator (testCreator) to get DIFFERENT campaign address
  // =========================================================================
  it("Wait until after deadline → withdraw should succeed", async () => {
    // Derive campaign address using DIFFERENT creator: [b"campaign", testCreator]
    const [campaignAddr] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("campaign"), testCreatorPublicKey.toBuffer()],
      program.programId,
    );

    // Derive vault address
    const [vaultAddr] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), campaignAddr.toBuffer()],
      program.programId,
    );

    // Create campaign with deadline 1 second in the FUTURE (not past)
    // This ensures creation succeeds, then we advance clock before withdraw
    const nearDeadline = Math.floor(Date.now() / 1000) + 1;

    const createTx = await program.methods
      .createCampaign(new anchor.BN(GOAL), new anchor.BN(nearDeadline))
      .accounts({
        creator: testCreatorPublicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([testCreator])
      .rpc();

    console.log("Test campaign created:", createTx);

    // Contribute to meet goal (1100 SOL > 1000 SOL goal)
    const contributeTx = await program.methods
      .contribute(new anchor.BN(1100 * 1_000_000_000))
      .accounts({
        donor: testCreatorPublicKey,
        campaign: campaignAddr,
        vault: vaultAddr,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([testCreator])
      .rpc();

    console.log("Test contribute:", contributeTx);

    // Verify goal reached
    const campaignBefore = await program.account.campaign.fetch(campaignAddr);
    console.log(
      "Campaign raised:",
      campaignBefore.raised.toNumber() / 1_000_000_000,
      "SOL",
    );
    console.log(
      "Campaign goal:",
      campaignBefore.goal.toNumber() / 1_000_000_000,
      "SOL",
    );
    assert.isTrue(campaignBefore.raised.gte(campaignBefore.goal));

    // Advance clock past deadline (add 2 seconds to ensure deadline passed)
    await advanceClockBy(2);
    console.log("Clock advanced past deadline");

    // Now withdraw should succeed (deadline passed and goal reached)
    const withdrawTx = await program.methods
      .withdraw()
      .accounts({
        creator: testCreatorPublicKey,
        campaign: campaignAddr,
        vault: vaultAddr,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([testCreator])
      .rpc();

    console.log("Withdraw transaction:", withdrawTx);

    // Verify claimed = true
    const campaignAfter = await program.account.campaign.fetch(campaignAddr);
    assert.equal(campaignAfter.claimed, true);

    console.log("Withdraw successful!");
    console.log(" Campaign claimed:", campaignAfter.claimed);
  });

  // =========================================================================
  // TEST 6: Try withdraw again → should fail (already claimed)
  // Use SAME campaign from TEST 5 (same testCreator)
  // =========================================================================
  it("Try withdraw again → should fail (already claimed)", async () => {
    // Use the SAME campaign from test 5
    const [campaignAddr] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("campaign"), testCreatorPublicKey.toBuffer()],
      program.programId,
    );

    const [vaultAddr] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), campaignAddr.toBuffer()],
      program.programId,
    );

    try {
      await program.methods
        .withdraw()
        .accounts({
          creator: testCreatorPublicKey,
          campaign: campaignAddr,
          vault: vaultAddr,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([testCreator])
        .rpc();

      assert.fail("Should have thrown error");
    } catch (error) {
      console.log("Correctly rejected double withdrawal");
      assert.include(error.toString(), "Campaign funds already claimed");
    }
  });
});
