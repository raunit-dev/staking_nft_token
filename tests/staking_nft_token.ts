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
  let rewardMint: PublicKey;
  
  const [config] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    programId
  );

  const [rewardMintPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("rewards"), config.toBuffer()],
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
      lamports: 10 * LAMPORTS_PER_SOL
    });
    const tx = new Transaction().add(transfer);
    await provider.sendAndConfirm(tx);
  });

  it("initialized config", async () => {
    try {
      const tx = await program.methods.
          initConfig(
          8,
          4,
          2,
          86400
        )
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
      const configAccount = await program.account.stakeConfigAccount.fetch(config);
      console.log("Config account data:", configAccount);
      
    } catch (error) {
      console.error("Error initializing config:", error);
      throw error;
    }
  });
});