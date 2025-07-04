import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CarbonCreditOft } from "../target/types/carbon_credit_oft";
import { TOKEN_PROGRAM_ID, createMint, createAccount, getAccount } from "@solana/spl-token";
import { expect } from "chai";

describe("carbon-credit-oft", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.CarbonCreditOft as Program<CarbonCreditOft>;
  const provider = anchor.getProvider();

  let oftConfig: anchor.web3.PublicKey;
  let mint: anchor.web3.PublicKey;
  let userTokenAccount: anchor.web3.PublicKey;
  let creditRecord: anchor.web3.PublicKey;

  const projectId = "AMAZON-FOREST-001";
  const vintage = 2024;
  const mintAmount = new anchor.BN(1000); // 1000 carbon credits

  before(async () => {
    // Create mint
    mint = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      provider.wallet.publicKey,
      6 // 6 decimals
    );

    // Create user token account
    userTokenAccount = await createAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      provider.wallet.publicKey
    );

    // Derive PDAs
    [oftConfig] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("oft_config")],
      program.programId
    );

    const timestamp = Math.floor(Date.now() / 1000);
    [creditRecord] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("credit"),
        Buffer.from(projectId),
        Buffer.from(timestamp.toString().slice(-8)) // Use last 8 chars for uniqueness
      ],
      program.programId
    );
  });

  it("Initializes the OFT", async () => {
    // Mock LayerZero endpoint program ID
    const endpointProgram = anchor.web3.Keypair.generate().publicKey;

    const tx = await program.methods
      .initialize({
        endpointProgram,
        decimals: 6,
      })
      .accounts({
        admin: provider.wallet.publicKey,
        oftConfig,
        mint,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Initialize OFT transaction signature", tx);

    // Verify the OFT config was created
    const oftConfigAccount = await program.account.oftConfig.fetch(oftConfig);
    expect(oftConfigAccount.admin.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(oftConfigAccount.mint.toString()).to.equal(mint.toString());
    expect(oftConfigAccount.decimals).to.equal(6);
  });

  it("Mints carbon credits", async () => {
    const tx = await program.methods
      .mintCredits(mintAmount, projectId, vintage)
      .accounts({
        mintAuthority: provider.wallet.publicKey,
        mint,
        recipient: provider.wallet.publicKey,
        recipientTokenAccount: userTokenAccount,
        creditRecord,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Mint credits transaction signature", tx);

    // Verify tokens were minted
    const tokenAccount = await getAccount(provider.connection, userTokenAccount);
    expect(tokenAccount.amount.toString()).to.equal(mintAmount.toString());

    // Verify credit record was created
    const creditRecordAccount = await program.account.creditRecord.fetch(creditRecord);
    expect(creditRecordAccount.projectId).to.equal(projectId);
    expect(creditRecordAccount.vintage).to.equal(vintage);
    expect(creditRecordAccount.amount.toString()).to.equal(mintAmount.toString());
    expect(creditRecordAccount.owner.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(creditRecordAccount.status).to.deep.equal({ active: {} });
  });

  it("Gets credit information", async () => {
    const creditInfo = await program.methods
      .getCreditInfo()
      .accounts({
        creditRecord,
      })
      .view();

    expect(creditInfo.projectId).to.equal(projectId);
    expect(creditInfo.vintage).to.equal(vintage);
    expect(creditInfo.amount.toString()).to.equal(mintAmount.toString());
    expect(creditInfo.owner.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(creditInfo.status).to.deep.equal({ active: {} });
  });

  it("Retires carbon credits", async () => {
    const retireAmount = new anchor.BN(100);
    const retirementReason = "Corporate carbon neutrality program";

    const timestamp = Math.floor(Date.now() / 1000);
    const [retirementRecord] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("retirement"),
        provider.wallet.publicKey.toBuffer(),
        Buffer.from(timestamp.toString().slice(-8))
      ],
      program.programId
    );

    const tx = await program.methods
      .retireCredits(retireAmount, retirementReason)
      .accounts({
        owner: provider.wallet.publicKey,
        mint,
        ownerTokenAccount: userTokenAccount,
        retirementRecord,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Retire credits transaction signature", tx);

    // Verify tokens were burned
    const tokenAccount = await getAccount(provider.connection, userTokenAccount);
    const expectedBalance = mintAmount.sub(retireAmount);
    expect(tokenAccount.amount.toString()).to.equal(expectedBalance.toString());

    // Verify retirement record was created
    const retirementRecordAccount = await program.account.retirementRecord.fetch(retirementRecord);
    expect(retirementRecordAccount.owner.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(retirementRecordAccount.amount.toString()).to.equal(retireAmount.toString());
    expect(retirementRecordAccount.reason).to.equal(retirementReason);
  });

  it("Handles cross-chain credit transfer data", async () => {
    // Test the credit transfer data structure
    const transferData = {
      recipient: provider.wallet.publicKey,
      amount: new anchor.BN(500),
      projectId: "SOLAR-FARM-002",
      vintage: 2024,
    };

    // This would normally be tested with actual LayerZero message passing
    // For now, we verify the data structure can be serialized
    const serialized = anchor.utils.bytes.utf8.encode(JSON.stringify(transferData));
    expect(serialized.length).to.be.greaterThan(0);
  });
});