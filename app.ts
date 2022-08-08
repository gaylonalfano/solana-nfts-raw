// Our Client
import * as BufferLayout from "@solana/buffer-layout";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
  sendAndConfirmRawTransaction,
} from "@solana/web3.js";
import { createKeypairFromFile } from "./util";
import fs from "mz/fs";
import os from "os";
import path from "path";
import yaml from "yaml";

// 1. Get path to Solana config
const CONFIG_FILE_PATH = path.resolve(
  os.homedir(),
  ".config",
  "solana",
  "cli",
  "config.yml"
);

async function main() {
  const connection = new Connection(
    "https://api.devnet.solana.com",
    "confirmed"
  );
  console.log("Successfully connected to Solana dev net");

  const configYml = await fs.readFile(CONFIG_FILE_PATH, { encoding: "utf8" });
  const keypairPath = await yaml.parse(configYml).keypair_path;
  const wallet = await createKeypairFromFile(keypairPath);
  console.log("Local account (wallet) loaded successfully");

  const programKeypair = await createKeypairFromFile(
    path.join(
      path.resolve(__dirname, "./mint/target/deploy/"),
      "mint-keypair.json"
    )
  );
  const programId = programKeypair.publicKey;
  console.log(`Program ID: ${programId.toBase58()}`);

  // 1. Derive the mint address of NFT and associated token account address
  // IMPORTANT: We just derive the account addresses on the Client-side, and then
  // let our program take care of creating the actual accounts
  // NOTE: The Keypair.generate() is so we can pass the mintKeypair.publicKey
  // to the SystemProgram, which will create the account to house the Mint
  const mintKeypair: Keypair = Keypair.generate(); // A new unique keypair
  const tokenAddress = await getAssociatedTokenAddress(
    mintKeypair.publicKey,
    wallet.publicKey
  );
  console.log(`New token: ${mintKeypair.publicKey}`);

  // 2. Transact with the process_instruction() fn in our on-chain program
  // NOTE All accounts need to be in correct order!
  const instruction = new TransactionInstruction({
    keys: [
      // Mint account
      {
        pubkey: mintKeypair.publicKey,
        isSigner: true,
        isWritable: true,
      },
      // Token account
      {
        pubkey: tokenAddress,
        isSigner: false,
        isWritable: true,
      },
      // Mint authority
      {
        pubkey: wallet.publicKey,
        isSigner: true,
        isWritable: false,
      },
      // Rent
      {
        pubkey: SYSVAR_RENT_PUBKEY,
        isSigner: false,
        isWritable: false,
      },
      // System program
      {
        pubkey: SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
      // Token program
      {
        pubkey: TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      // Associated token program
      {
        pubkey: ASSOCIATED_TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
    ],
    programId: programId, // Our own Program
    data: Buffer.alloc(0), // Instruction Data
  });

  // 3. Send and confirm the transaction
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [wallet, mintKeypair]
  );
}

main().then(
  () => process.exit(),
  (err) => {
    console.error(err);
    process.exit(-1);
  }
);
