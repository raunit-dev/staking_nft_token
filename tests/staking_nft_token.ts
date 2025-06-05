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
  let userStakeAccount: PublicKey;
  let solVault: PublicKey;
  let rewardMintPda: PublicKey;

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

    [userStakeAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("stake_account"),
        user.publicKey.toBuffer(),
        config.toBuffer(),
      ],
      programId
    );

    [solVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), userAccount.toBuffer()],
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
        .initConfig(8, 4, 2, 86400)
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
      .initUser()
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
      .stakeSol(new anchor.BN(stakeAmount))
      .accountsPartial({
        user: user.publicKey,
        rewardMint: rewardMintPda,
        stakeMint: rewardMintPda,
        stakeConfig: config,
        userStakeAccount,
        userAccount,
        userRewardAta: userRewardAtaAccount.address,
        solVault,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();
    await confirm(txSig);
    await log(txSig);
  });
});