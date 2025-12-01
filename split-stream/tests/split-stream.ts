
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SplitStream } from "../target/types/split_stream";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL, Connection } from "@solana/web3.js";
import { assert } from "chai";

describe("split-stream", () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.splitStream as Program<SplitStream>;

  let creator: Keypair;
  let recipeint1: Keypair;
  let recipeint2: Keypair;
  let depositor: Keypair;

  let nftMint: PublicKey;

  let royaltySplitPda: PublicKey;
  let newRoyaltySplitPda: PublicKey;
  let bump: number;

  before(async() => {
    creator = Keypair.generate();
    recipeint1 = Keypair.generate();
    recipeint2 = Keypair.generate();
    depositor = Keypair.generate();

    nftMint = Keypair.generate().publicKey;

    await airdrop(provider.connection, creator.publicKey, 2);
    await airdrop(provider.connection, depositor.publicKey, 2);

    console.log("Creator", creator.publicKey.toString());
    console.log("Recipient 1", recipeint1.publicKey.toString());
    console.log("Recipient 2", recipeint2.publicKey.toString());
    console.log("Depositor", depositor.publicKey.toString());
  })

  it("Initailized a royalty split with two recipeints", async () => {
    [royaltySplitPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("royalty_split"), nftMint.toBuffer()],
      program.programId
    );

    console.log("Royalty split PDA:", royaltySplitPda.toString());

    const recipeints = [
        {wallet: recipeint1.publicKey, percentage: 60},
        {wallet: recipeint2.publicKey, percentage: 40}
    ];

    const tx = await program.methods
    .initializeSplit(nftMint, recipeints)
    .accountsPartial({
      royaltySplit: royaltySplitPda,
      creator: creator.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([creator])
    .rpc();

    console.log("Initialize transaction signature:", tx);

    const royaltySplitAccount = await program.account.royaltySplit.fetch(royaltySplitPda);

    assert.equal(royaltySplitAccount.nftMint.toString(), nftMint.toString());
    assert.equal(royaltySplitAccount.creator.toString(), creator.publicKey.toString());
    assert.equal(royaltySplitAccount.recipients.length, 2);
    assert.equal(royaltySplitAccount.recipients[0].percentage, 60);
    assert.equal(royaltySplitAccount.recipients[1].percentage, 40);
    assert.equal(royaltySplitAccount.totalCollected.toNumber(), 0);
    assert.equal(royaltySplitAccount.isActive, true);

    console.log("Royalty split initialized successfully!");
  });

  it("Deposit royalty to the split", async() => {
    const depositAmount = 1 * LAMPORTS_PER_SOL;
    const balanceBefore = await provider.connection.getBalance(royaltySplitPda);

    const tx = await program.methods
    .depositRoyalty(new anchor.BN(depositAmount))
    .accountsPartial({
      royaltySplit: royaltySplitPda,
      depositor: depositor.publicKey,
      systemProgram: SystemProgram.programId
    })
    .signers([depositor])
    .rpc();

    console.log("Deposit ttransaction sucessfull");

    const balanceAfter = await provider.connection.getBalance(royaltySplitPda);

    assert.equal(balanceAfter - balanceBefore, depositAmount);

    const royaltySplitAccount = await program.account.royaltySplit.fetch(royaltySplitPda);
    assert.equal(royaltySplitAccount.totalCollected.toNumber(), depositAmount);

    console.log("Deposited 1 SOL successfully");
  });

  it("Prevents unauthorized recipient from claiming", async() => {
    try {
      await program.methods
      .claimShare(new anchor.BN(0))
      .accountsPartial({
        royaltySplit: royaltySplitPda,
        recipient: recipeint2.publicKey
      })
      .signers([recipeint2])
      .rpc();

      assert.fail("Should have failed");
    } catch(err) {
        assert.include(err.toString(), "UnauthorizedRecipient");
      }
  });

  it("Rejects split with percentge != 100", async() => {
    const newNftMint = Keypair.generate().publicKey;
    [newRoyaltySplitPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("royalty_split"), newNftMint.toBuffer()],
      program.programId
    );
    const badRecipients = [
      {wallet: recipeint1.publicKey, percentage: 60},
      {wallet: recipeint2.publicKey, percentage: 30}
    ];

    try {
      const tx = await program.methods
      .initializeSplit(newNftMint, badRecipients)
      .accountsPartial({
        royaltySplit: newRoyaltySplitPda,
        creator: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([creator])
      .rpc();
      assert.fail("Should have failed");
    } catch(err) {
      assert.include(err.toString(), "SumExceed100Error");
    }
  });

  it("Handles multiple deposits before claiming", async() => {
    await program.methods
      .depositRoyalty(new anchor.BN(0.5 * LAMPORTS_PER_SOL))
      .accountsPartial({
        royaltySplit: royaltySplitPda,
        depositor: depositor.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([depositor])
      .rpc();
    
    await program.methods
      .depositRoyalty(new anchor.BN(0.5 * LAMPORTS_PER_SOL))
      .accountsPartial({
        royaltySplit: royaltySplitPda,
        depositor: depositor.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([depositor])
      .rpc();

    const account = await program.account.royaltySplit.fetch(royaltySplitPda);
    assert.equal(account.totalCollected.toNumber(), 2 * LAMPORTS_PER_SOL);
    
    console.log("Multiple deposits tracked correctly!");
  });

  it("Recipeint 1 claims their 60% share", async() => {
    const totalDeposited = 2 * LAMPORTS_PER_SOL;
    const expectedShare = (totalDeposited * 60) /100;

    const balanceBefore = await provider.connection.getBalance(recipeint1.publicKey);

    const tx = await program.methods
    .claimShare(new anchor.BN(0))
    .accountsPartial({
      royaltySplit: royaltySplitPda,
      recipient: recipeint1.publicKey
    })
    .signers([recipeint1])
    .rpc();

    console.log("Claim transaction signature:", tx);

    const balanceAfter = await provider.connection.getBalance(recipeint1.publicKey);
    assert.approximately(balanceAfter - balanceBefore , expectedShare, 10000);

    const royaltySplitAccount = await program.account.royaltySplit.fetch(royaltySplitPda);
    assert.equal(royaltySplitAccount.recipients[0].claimed.toNumber(), expectedShare);

    console.log(`Recipient 1 claimed ${expectedShare / LAMPORTS_PER_SOL} SOL!`);
  });

  it("Recipeint 2 claims their 40% share", async() => {
    const totalDeposited = 2 * LAMPORTS_PER_SOL;
    const expectedShare = (totalDeposited * 40) /100;

    const balanceBefore = await provider.connection.getBalance(recipeint2.publicKey);

    const tx = await program.methods
    .claimShare(new anchor.BN(1))
    .accountsPartial({
      royaltySplit: royaltySplitPda,
      recipient: recipeint2.publicKey
    })
    .signers([recipeint2])
    .rpc();

    console.log("Claim transaction signature:", tx);

    const balanceAfter = await provider.connection.getBalance(recipeint2.publicKey);
    assert.approximately(balanceAfter - balanceBefore , expectedShare, 10000);

    const royaltySplitAccount = await program.account.royaltySplit.fetch(royaltySplitPda);
    assert.equal(royaltySplitAccount.recipients[1].claimed.toNumber(), expectedShare);
    
    console.log(`Recipient 2 claimed ${expectedShare / LAMPORTS_PER_SOL} SOL!`);
  });

  it("Cannot claim again when nothing left to claim", async() => {
    try {
      await program.methods
      .claimShare(new anchor.BN(0))
      .accountsPartial({
        royaltySplit: royaltySplitPda,
        recipient: recipeint1.publicKey
      })
      .signers([recipeint1])
      .rpc();

      assert.fail("Should have thrown an error");
    } catch(err) {
      assert.include(err.toString(), "NothingToClaim");
      console.log("Correctly prevented double claim!");
    }
  });

  it("Close the split after all funds are claimed", async() => {
    const tx = await program.methods
    .closeSplit()
    .accountsPartial({
      royaltySplit: royaltySplitPda,
      creator: creator.publicKey
    })
    .signers([creator])
    .rpc();

    console.log("Close transaction signature: ", tx);

    try {
      await program.account.royaltySplit.fetch(royaltySplitPda);
      assert.fail("Account should be closed");
    } catch(err) {
      console.log("Account closed successfully");
    }
  });
});

async function airdrop(connection: Connection, publicKey: PublicKey, amount: number) {
  const signature = await connection.requestAirdrop(
    publicKey,
    amount * LAMPORTS_PER_SOL
  );

  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    signature,
    ...latestBlockhash,
  });
}

