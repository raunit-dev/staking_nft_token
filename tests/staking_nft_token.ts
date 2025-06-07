import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StakingNftToken } from "../target/types/staking_nft_token";
import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { BN } from "bn.js";
import { randomBytes } from "crypto"

describe("staking_nft_token", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const connection = provider.connection;
  const program = anchor.workspace.StakingNftToken as Program<StakingNftToken>;
  const programId = program.programId;

  let admin: Keypair;
  let user: Keypair;
  let rewardMint: PublicKey;
  let userRewardAta: PublicKey;
  let userAccount: PublicKey;
  let stakeAccount: PublicKey;
  let vault: PublicKey;
  let rewardMintPda: PublicKey;
  let splMint: PublicKey;
  let splStakeAccount: PublicKey;
  let vaultAta: PublicKey;

  let seed = new BN(randomBytes(8));

  const [config] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    programId
  );

  async function confirm(signature: string): Promise<string> {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  }

  async function log(signature: string): Promise<string> {
    console.log(
      ` Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  }

  before(async () => {
    admin = Keypair.generate();
    const transfer = SystemProgram.transfer({
      fromPubkey: provider.publicKey,
      toPubkey: admin.publicKey,
      lamports: 10 * LAMPORTS_PER_SOL,
    });
    const tx = new Transaction().add(transfer);
    await provider.sendAndConfirm(tx);

    user = Keypair.generate();
    const transfer2 = SystemProgram.transfer({
      fromPubkey: provider.publicKey,
      toPubkey: user.publicKey,
      lamports: 5 * LAMPORTS_PER_SOL,
    });
    const tx2 = new Transaction().add(transfer2);
    await provider.sendAndConfirm(tx2);

    [userAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("user"), user.publicKey.toBuffer()],
      programId
    );

    [stakeAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("stake"),
        config.toBuffer(),
        user.publicKey.toBuffer(),
        seed.toArrayLike(Buffer, "le", 8)
      ],
      programId
    );

    [vault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), stakeAccount.toBuffer()],
      programId
    );

    [rewardMintPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("rewards"), config.toBuffer()],
      programId
    );
  });

  it("initialized config", async () => {
    try {
      const tx = await program.methods
        .initializeConfig(8, 4, 2, 86400)
        .accountsPartial({
          admin: admin.publicKey,
          config: config,
          rewardMint: rewardMintPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      await confirm(tx);
      await log(tx);

      console.log("Config initialized successfully!");
      const configAccount = await program.account.stakeConfigAccount.fetch(
        config
      );
      console.log("Config account data:", configAccount);
    } catch (error) {
      console.error("Error initializing config:", error);
      throw error;
    }
  });

  it("initialize user", async () => {
    const txSig = await program.methods
      .initializeUser()
      .accountsPartial({
        user: user.publicKey,
        userAccount,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    await confirm(txSig);
    await log(txSig);
  });

  it("stake sol", async () => {
    const userRewardAtaAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      rewardMintPda,
      user.publicKey
    );

    const stakeAmount = 1 * LAMPORTS_PER_SOL;
    const txSig = await program.methods
      .stakeSol(seed,new anchor.BN(stakeAmount))
      .accountsPartial({
        user: user.publicKey,
        rewardMint: rewardMintPda,
        config: config,
        stakeAccount,
        userAccount,
        userRewardAta: userRewardAtaAccount.address,
        vault,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();
    await confirm(txSig);
    await log(txSig);
  });

  it("stake spl", async () => {
    // Create a new SPL mint
    splMint = await createMint(
      connection,
      admin,
      admin.publicKey,
      null,
      6 // decimals
    );

    // Create user's ATA for the SPL mint
    const userSplAta = await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      splMint,
      user.publicKey
    );

    // Mint SPL tokens to user
    await mintTo(
      connection,
      admin,
      splMint,
      userSplAta.address,
      admin,
      1_000_000 // 1 token (with 6 decimals)
    );

    // Derive stake_account PDA for SPL
    [splStakeAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("stake"),
        config.toBuffer(),
        user.publicKey.toBuffer(),
        splMint.toBuffer(),
      ],
      programId
    );

    // Derive vault_ata PDA for SPL
    vaultAta = getAssociatedTokenAddressSync(
      splMint,
      splStakeAccount,
      true
    );

    // User's reward ATA for reward mint
    const userRewardAtaAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      rewardMintPda,
      user.publicKey
    );

    const stakeAmount = 500_000; // 0.5 SPL token
    const txSig = await program.methods
      .stakeSpl(new anchor.BN(12345), new anchor.BN(stakeAmount))
      .accountsPartial({
        user: user.publicKey,
        mint: splMint,
        mintAta: userSplAta.address,
        rewardMint: rewardMintPda,
        userRewardAta: userRewardAtaAccount.address,
        stakeAccount: splStakeAccount,
        config: config,
        vaultAta: vaultAta,
        userAccount: userAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    await confirm(txSig);
    await log(txSig);
    console.log("SPL staked successfully!");
    const userAcc = await program.account.userAccount.fetch(userAccount);
    console.log("User account after SPL stake:", userAcc);
  });

  it("unstake sol", async () => {
    // Simulate freeze period passed by manipulating the clock (or by direct state update in local/test env)
    // For now, just try the call (may need to update the stake_account's staked_at if running locally)
    // Derive the stake_account PDA for SOL
    // (already derived as stakeAccount in before hook)
    // User's reward ATA for reward mint
    const userRewardAtaAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      rewardMintPda,
      user.publicKey
    );

    // const splMint = await createMint(
    //   connection,
    //   admin,
    //   admin.publicKey,
    //   null,
    //   6 // decimals
    // );

    // const [splStakeAccount] = PublicKey.findProgramAddressSync(
    //   [
    //     Buffer.from("stake"),
    //     config.toBuffer(),
    //     user.publicKey.toBuffer(),
    //     splMint.toBuffer(),
    //   ],
    //   programId
    // );


    // Derive vault PDA for SOL
    // const vaultAta = getAssociatedTokenAddressSync(
    //   splMint,
    //   splStakeAccount,
    //   true
    // )
    const txSig = await program.methods
      .unstakeSol()
      .accountsPartial({
        user: user.publicKey,
        mint: rewardMintPda, // using rewardMint as placeholder, adjust if needed
        rewardMint: rewardMintPda,
        userRewardAta: userRewardAtaAccount.address,
        stakeAccount: stakeAccount,
        config: config,
        vault,
        userAccount: userAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    await confirm(txSig);
    await log(txSig);
    console.log("SOL unstaked successfully!");
    const userAcc = await program.account.userAccount.fetch(userAccount);
    console.log("User account after SOL unstake:", userAcc);
  });

  it("unstake spl", async () => {
    // Use the stored SPL mint and stake account from the stake_spl test
    const userRewardAtaAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      rewardMintPda,
      user.publicKey
    );

    splMint = await createMint(
      connection,
      admin,
      admin.publicKey,
      null,
      6 // decimals
    );

    // Create user's ATA for the SPL mint
    const userSplAta = await getOrCreateAssociatedTokenAccount(
      connection,
      user,
      splMint,
      user.publicKey
    );


    const txSig = await program.methods
      .unstakeSpl()
      .accountsPartial({
        user: user.publicKey,
        mint: splMint, // Using the stored splMint
        mintAta: userSplAta.address, // Using the stored userSplAta
        rewardMint: rewardMintPda,
        userRewardAta: userRewardAtaAccount.address,
        stakeAccount: splStakeAccount, // Using the stored splStakeAccount
        config: config,
        vaultAta: vaultAta, // Using the stored vaultAta
        userAccount: userAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();
    await confirm(txSig);
    await log(txSig);
    console.log("SPL unstaked successfully!");
    const userAcc = await program.account.userAccount.fetch(userAccount);
    console.log("User account after SPL unstake:", userAcc);
  });
});