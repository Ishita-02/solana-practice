import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MyEscrowProject } from "../target/types/my_escrow_project";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL, Connection } from "@solana/web3.js";
import { assert } from "chai";

describe("my_escrow_project", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.myEscrowProject as Program<MyEscrowProject>;

  let seller: Keypair;
  let buyer: Keypair;

  let escrowPda: PublicKey;
  let bump: number;

  before(async() => {
    seller = Keypair.generate();
    buyer = Keypair.generate();

    escrowPda = Keypair.generate().publicKey;

    await airdrop(provider.connection, seller.publicKey, 1 * LAMPORTS_PER_SOL);
    await airdrop(provider.connection, buyer.publicKey, 1 * LAMPORTS_PER_SOL);

    console.log("Seller: ", seller.publicKey.toString());
    console.log("Buyer: ", buyer.publicKey.toString());
  });

  it("Initialize an escrow", async () => {
    [escrowPda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), seller.publicKey.toBuffer()],
      program.programId
    );

    console.log("Escrow PDA: ", escrowPda.toString());

    const amount = 1 * LAMPORTS_PER_SOL;
    const item_details = "Pen";
    const tx = await program.methods
    .initializeEscrow(new anchor.BN(amount), item_details)
    .accountsPartial({
      escrow: escrowPda,
      seller: seller.publicKey,
      systemProgram: SystemProgram.programId
    })
    .signers([seller])
    .rpc();

    console.log("Initialize transaction signature: ", tx);

    const escrowAccount = await program.account.escrow.fetch(escrowPda);

    assert.equal(escrowAccount.amount.toNumber(), amount);
    assert.equal(escrowAccount.itemDetails, item_details);
    assert.equal(escrowAccount.isActive, false);

    console.log("Escrow created successfully");

  });
});

async function airdrop(connection: Connection, creator: PublicKey, amount: number) {
  const signature = await connection.requestAirdrop(creator, amount * LAMPORTS_PER_SOL);
  const latestBlockHash = await connection.getLatestBlockhash();

  const tx = await connection.confirmTransaction({
    signature,
    ...latestBlockHash
  });
}
