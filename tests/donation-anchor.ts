import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DonationAnchor } from "../target/types/donation_anchor";
import { assert } from "chai";

describe("donation-anchor", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.donationAnchor as Program<DonationAnchor>;
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const wallet = provider.wallet;

  const jarPDA = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("jar"), wallet.publicKey.toBuffer()],
    program.programId,
  )[0];

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initJar()
      .accounts({
        jar: jarPDA,
        creator: wallet.publicKey,
        system_program: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("Your transaction signature", tx);

    const state = await program.account.jar.fetch(jarPDA);
    console.log("Jar state:", state);
    assert.equal(
      state.totalRaised.toString(),
      "0",
      "Initial total donations should be 0",
    );
    assert.equal(
      state.creator.toString(),
      wallet.publicKey.toString(),
      "Creator should be the wallet that initialized the jar",
    );
    assert.equal(
      state.donationCount.toString(),
      "0",
      "Initial donation count should be 0",
    );
  });

  it("Allows donations!", async () => {
    const donationAmount = new anchor.BN(1_000_000); // 0.001 SOL

    const donor = anchor.web3.Keypair.generate();
    const airdropSig = await provider.connection.requestAirdrop(
      donor.publicKey,
      2_000_000, // Airdrop 0.002 SOL to cover donation and fees
    );
    await provider.connection.confirmTransaction(airdropSig);

    const tx = await program.methods
      .donate(donationAmount)
      .accounts({
        jar: jarPDA,
        donor: donor.publicKey,
        system_program: anchor.web3.SystemProgram.programId,
      })
      .signers([donor])
      .rpc();
    console.log("Donation transaction signature", tx);

    const state = await program.account.jar.fetch(jarPDA);
    console.log("Jar state after donation:", state);
    assert.equal(
      state.totalRaised.toString(),
      donationAmount.toString(),
      "Total donations should reflect the donated amount",
    );
    assert.equal(
      state.donationCount.toString(),
      "1",
      "Donation count should be 1 after one donation",
    );
  });

  it("Allows withdrawals!", async () => {
    const withdrawalAmount = new anchor.BN(500_000); // 0.0005 SOL

    const initialBalance = await provider.connection.getBalance(
      wallet.publicKey,
    );

    const tx = await program.methods
      .withdraw(withdrawalAmount)
      .accounts({
        jar: jarPDA,
        creator: wallet.publicKey,
        system_program: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("Withdrawal transaction signature", tx);

    const finalBalance = await provider.connection.getBalance(wallet.publicKey);
    const balanceDifference = finalBalance - initialBalance;
    console.log("Balance difference after withdrawal:", balanceDifference);
    assert.isAbove(finalBalance, initialBalance);

    const state = await program.account.jar.fetch(jarPDA);
    console.log("Jar state after withdrawal:", state);
  });
});
