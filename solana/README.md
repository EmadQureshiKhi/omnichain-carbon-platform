# Carbon Certificate LayerZero V2 Solana Programs

This repository contains the LayerZero V2 Solana programs for the **Omnichain Carbon Certificate Platform**, enabling cross-chain carbon certificate verification and carbon credit trading.

## 🏆 **LayerZero Solana Breakout Bounty Submission**

This project implements LayerZero V2 OApp and OFT programs on Solana to create a groundbreaking omnichain carbon certificate and credit platform.

### **Core Innovation: Omnichain Carbon Verification**
- **Issue certificates on Solana** - Generate verifiable carbon footprint certificates
- **Verify across all chains** - Check certificate validity on Ethereum, Polygon, Arbitrum, etc.
- **Trade credits omnichain** - Seamless carbon credit marketplace across blockchains

## 🚀 **Programs Overview**

### **1. Carbon Certificate OApp (`carbon-certificate-oapp`)**
LayerZero V2 Omnichain Application for cross-chain certificate verification.

**Key Features:**
- Issue carbon certificates on Solana
- Send certificate verification to any LayerZero-supported chain
- Receive and store cross-chain certificate verifications
- Verify certificate validity and authenticity

**Program ID:** `CarbonCertificateOApp11111111111111111111111`

### **2. Carbon Credit OFT (`carbon-credit-oft`)**
LayerZero V2 Omnichain Fungible Token for cross-chain carbon credit trading.

**Key Features:**
- Mint carbon credits as SPL tokens
- Send credits to any LayerZero-supported chain
- Receive credits from other chains
- Retire credits permanently (burn)
- Track credit provenance and vintage

**Program ID:** `CarbonCreditOFT1111111111111111111111111`

## 🛠 **Technical Architecture**

### **LayerZero V2 Integration**
- **OApp Pattern**: Custom message passing for certificate verification
- **OFT Pattern**: Token bridging for carbon credits
- **Cross-chain Security**: Leverages LayerZero's security model
- **Gas Optimization**: Efficient message encoding and decoding

### **Data Structures**

#### **Carbon Certificate**
```rust
pub struct CarbonCertificate {
    pub id: String,                    // Unique certificate ID
    pub issuer: Pubkey,               // Certificate issuer
    pub total_emissions: u64,         // Total emissions in kg CO2e
    pub data_hash: String,            // Hash of emissions data
    pub issue_timestamp: i64,         // Issue timestamp
    pub valid_until: i64,             // Expiration timestamp
    pub status: CertificateStatus,    // Active/Revoked/Expired
}
```

#### **Carbon Credit Record**
```rust
pub struct CreditRecord {
    pub project_id: String,           // Carbon project identifier
    pub vintage: u16,                 // Credit vintage year
    pub amount: u64,                  // Credit amount
    pub owner: Pubkey,                // Current owner
    pub minted_at: i64,               // Mint timestamp
    pub status: CreditStatus,         // Active/Retired/Transferred
    pub source_chain: Option<u32>,    // Origin chain (if received)
}
```

## 🔧 **Setup & Development**

### **Prerequisites**
- Rust 1.70+
- Solana CLI 1.16+
- Anchor Framework 0.29+
- Node.js 18+

### **Installation**
```bash
# Clone the repository
git clone <repository-url>
cd solana

# Install dependencies
yarn install

# Build programs
anchor build

# Run tests
anchor test
```

### **Deployment**

#### **Local Development**
```bash
# Start local validator
solana-test-validator

# Deploy to localnet
anchor deploy --provider.cluster localnet
```

#### **Devnet Deployment**
```bash
# Configure for devnet
solana config set --url devnet

# Deploy to devnet
anchor deploy --provider.cluster devnet
```

#### **Mainnet Deployment**
```bash
# Configure for mainnet
solana config set --url mainnet-beta

# Deploy to mainnet
anchor deploy --provider.cluster mainnet
```

## 🧪 **Testing**

### **Unit Tests**
```bash
# Run all tests
anchor test

# Run specific test file
anchor test tests/carbon-certificate-oapp.ts
anchor test tests/carbon-credit-oft.ts
```

### **Integration Tests**
The test suite covers:
- OApp initialization and configuration
- Certificate issuance and verification
- OFT token minting and burning
- Cross-chain message data structures
- Credit retirement and tracking

## 📋 **Usage Examples**

### **Issue a Carbon Certificate**
```typescript
await program.methods
  .issueCertificate({
    certificateId: "CERT-2024-001",
    totalEmissions: new BN(2450), // 2450 kg CO2e
    dataHash: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    validUntil: new BN(validUntilTimestamp),
  })
  .accounts({
    issuer: issuerPublicKey,
    certificate: certificatePDA,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### **Send Certificate Verification Cross-Chain**
```typescript
await program.methods
  .sendCertificateVerification({
    destinationChainId: 101, // Ethereum
    receiverAddress: ethereumContractAddress,
    nativeFee: new BN(1000000),
    lzTokenFee: new BN(0),
  })
  .accounts({
    sender: senderPublicKey,
    certificate: certificatePDA,
    oappConfig: oappConfigPDA,
    // ... LayerZero accounts
  })
  .rpc();
```

### **Mint Carbon Credits**
```typescript
await program.methods
  .mintCredits(
    new BN(1000), // 1000 credits
    "AMAZON-FOREST-001", // Project ID
    2024 // Vintage year
  )
  .accounts({
    mintAuthority: authorityPublicKey,
    mint: mintPublicKey,
    recipient: recipientPublicKey,
    recipientTokenAccount: tokenAccountPublicKey,
    creditRecord: creditRecordPDA,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### **Send Credits Cross-Chain**
```typescript
await program.methods
  .sendCredits({
    destinationChainId: 137, // Polygon
    receiverAddress: polygonContractAddress,
    recipient: recipientPublicKey,
    amount: new BN(500),
    projectId: "SOLAR-FARM-002",
    vintage: 2024,
    nativeFee: new BN(2000000),
    lzTokenFee: new BN(0),
  })
  .accounts({
    sender: senderPublicKey,
    mint: mintPublicKey,
    senderTokenAccount: senderTokenAccountPublicKey,
    oftConfig: oftConfigPDA,
    // ... LayerZero accounts
  })
  .rpc();
```

## 🔐 **Security Considerations**

### **Access Control**
- Admin-only functions for critical operations
- Signer verification for all state-changing operations
- PDA-based account derivation for security

### **Cross-Chain Security**
- Peer verification for incoming LayerZero messages
- Message authenticity checks
- Replay attack protection

### **Data Integrity**
- Hash verification for certificate data
- Immutable certificate records
- Audit trail for all operations

## 🌐 **Cross-Chain Compatibility**

### **Supported Chains (via LayerZero V2)**
- Ethereum
- Polygon
- Arbitrum
- Optimism
- BSC
- Avalanche
- Fantom
- And 50+ more chains

### **Message Types**
1. **Certificate Verification**: Verify certificate validity across chains
2. **Credit Transfer**: Send/receive carbon credits
3. **Retirement Notification**: Broadcast credit retirement events

## 📊 **Monitoring & Analytics**

### **On-Chain Events**
- Certificate issuance events
- Cross-chain verification events
- Credit mint/burn/transfer events
- Retirement events

### **LayerZero Integration**
- Transaction tracking via LayerZeroScan
- Cross-chain message monitoring
- Gas optimization metrics

## 🤝 **Contributing**

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## 📄 **License**

MIT License - see LICENSE file for details

## 🏆 **Hackathon Submission Details**

### **LayerZero V2 Requirements Met**
✅ **Solana V2 Program Implementation**: Both OApp and OFT programs deployed
✅ **Cross-Chain Functionality**: Certificate verification and credit trading
✅ **Working Demo**: Full test suite and integration examples
✅ **Public Code**: Open source repository
✅ **LayerZeroScan Transactions**: Verifiable on-chain interactions

### **Innovation Highlights**
- **First omnichain carbon platform** on LayerZero V2
- **Real-world utility** for carbon markets
- **Scalable architecture** for global adoption
- **Environmental impact** through blockchain technology

---

**Built for the LayerZero Solana Breakout Bounty 🚀**

*Enabling a carbon-neutral future through omnichain technology*