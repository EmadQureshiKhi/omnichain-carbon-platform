import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CarbonCertificateOapp } from "../target/types/carbon_certificate_oapp";
import { expect } from "chai";

describe("carbon-certificate-oapp", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.CarbonCertificateOapp as Program<CarbonCertificateOapp>;
  const provider = anchor.getProvider();

  let oappConfig: anchor.web3.PublicKey;
  let certificate: anchor.web3.PublicKey;

  const certificateId = "CERT-2024-001";
  const totalEmissions = new anchor.BN(2450); // 2450 kg CO2e
  const dataHash = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
  const validUntil = new anchor.BN(Math.floor(Date.now() / 1000) + 365 * 24 * 60 * 60); // 1 year from now

  before(async () => {
    // Derive PDAs
    [oappConfig] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("oapp_config")],
      program.programId
    );

    [certificate] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("certificate"), Buffer.from(certificateId)],
      program.programId
    );
  });

  it("Initializes the OApp", async () => {
    // Mock LayerZero endpoint program ID
    const endpointProgram = anchor.web3.Keypair.generate().publicKey;

    const tx = await program.methods
      .initialize({
        endpointProgram,
      })
      .accounts({
        admin: provider.wallet.publicKey,
        oappConfig,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Initialize transaction signature", tx);

    // Verify the OApp config was created
    const oappConfigAccount = await program.account.oAppConfig.fetch(oappConfig);
    expect(oappConfigAccount.admin.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(oappConfigAccount.endpointProgram.toString()).to.equal(endpointProgram.toString());
  });

  it("Issues a carbon certificate", async () => {
    const tx = await program.methods
      .issueCertificate({
        certificateId,
        totalEmissions,
        dataHash,
        validUntil,
      })
      .accounts({
        issuer: provider.wallet.publicKey,
        certificate,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Issue certificate transaction signature", tx);

    // Verify the certificate was created
    const certificateAccount = await program.account.carbonCertificate.fetch(certificate);
    expect(certificateAccount.id).to.equal(certificateId);
    expect(certificateAccount.issuer.toString()).to.equal(provider.wallet.publicKey.toString());
    expect(certificateAccount.totalEmissions.toString()).to.equal(totalEmissions.toString());
    expect(certificateAccount.dataHash).to.equal(dataHash);
    expect(certificateAccount.status).to.deep.equal({ active: {} });
  });

  it("Verifies a certificate", async () => {
    const result = await program.methods
      .verifyCertificate(certificateId)
      .accounts({
        certificate,
      })
      .view();

    expect(result.exists).to.be.true;
    expect(result.isValid).to.be.true;
    expect(result.totalEmissions.toString()).to.equal(totalEmissions.toString());
  });

  it("Handles certificate verification data", async () => {
    // Test the certificate verification data structure
    const verificationData = {
      certificateId,
      issuer: provider.wallet.publicKey,
      totalEmissions,
      dataHash,
      issueTimestamp: new anchor.BN(Math.floor(Date.now() / 1000)),
      validUntil,
    };

    // This would normally be tested with actual LayerZero message passing
    // For now, we verify the data structure can be serialized
    const serialized = anchor.utils.bytes.utf8.encode(JSON.stringify(verificationData));
    expect(serialized.length).to.be.greaterThan(0);
  });
});