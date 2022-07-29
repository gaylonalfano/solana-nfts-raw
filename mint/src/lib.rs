use {
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint,
        entrypoint::ProgramResult,
        msg,
        native_token::LAMPORTS_PER_SOL,
        program::invoke,
        pubkey::Pubkey,
        system_instruction
    },
    spl_token::instruction as token_instruction,
    spl_associated_token_account::instruction as token_account_instruction,
};

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let mint = next_account_info(accounts_iter)?; // Create a new mint (token)
    let token_account = next_account_info(accounts_iter)?; // Create a token account for the mint
    let mint_authority = next_account_info(accounts_iter)?; // Our wallet
    let rent = next_account_info(accounts_iter)?; // Sysvar but still an account
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;
    let associated_token_program = next_account_info(accounts_iter)?;


    msg!("1. Creating account for the actual mint (token)...");
    msg!("Mint: {}", mint.key);
    // Invoke a Cross-program Invocation: 
    // NOTE Hits another program by sending accounts and doing stuff
    // Q: Is this the spl-token create-account <TOKEN_ADDRESS> command?
    invoke(
        &system_instruction::create_account(
            &mint_authority.key, // Our wallet. We're the signer and payer for the tx
            &mint.key,
            LAMPORTS_PER_SOL,
            82, // Standard mint space size
            &token_program.key // Owner. SPL Token Program owns the mint
        ),
        &[
            mint.clone(), // Clone so ownership isn't moved into each tx
            mint_authority.clone(),
            token_program.clone(),
        ]
    )?;

    // Q: Is this the spl-token create-account <TOKEN_ADDRESS> command?
    // A: NO! This is spl-token create-token --decimals 0
    // NOTE --decimals 0 is the protocol for NFTs
    msg!("2. Initializing mint account as a mint...");
    msg!("Mint: {}", mint.key);
    invoke(
        &token_instruction::initialize_mint(
            &token_program.key, // Setting it up with Token Program so it's writable by Token Program
            &mint.key,
            &mint_authority.key, // Setting our wallet as authority
            Some(&mint_authority.key), // freeze_authority
            0, // decimals = 0
        )?,
        &[
            mint.clone(),
            mint_authority.clone(),
            token_program.clone(),
            rent.clone(),
        ]
    )?;

    // Q: Is this spl-token create-account <TOKEN_ADDRESS> <OWNER_ADDRESS>?
    // NOTE When running this CLI command, the owner of account is our local keypair account
    // NOTE This create-account command literally adds the token account (token holdings) inside owner's wallet!
    // Q: Is this the Token Metadata Program creating the Metadata Account for the token?
    msg!("3. Creating token account for the mint and the wallet...");
    msg!("Token Address: {}", token_account.key);
    invoke(
        &token_account_instruction::create_associated_token_account(
            &mint_authority.key,
            &mint_authority.key,
            &mint.key,
        ),
        &[
            mint.clone(),
            token_account.clone(),
            mint_authority.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ]
    )?;

     

    // Q: Is this spl-token mint <TOKEN_ADDRESS> <AMOUNT> <RECIPIENT_ADDRESS>?
    msg!("4. Minting token to the token account (i.e. give it 1 for NFT)...");
    msg!("Mint: {}", mint.key);
    msg!("Token Address: {}", token_account.key);
    invoke(
        &token_instruction::mint_to(
            &token_program.key,
            &mint.key,
            &token_account.key, // The account to mint tokens to
            &mint_authority.key, // Mint's minting authority
            &[&mint_authority.key],
            1, // Amount of new tokens to mint
        )?,
        &[
            mint.clone(),
            mint_authority.clone(),
            token_account.clone(),
            token_program.clone(),
            rent.clone(),
        ]
    )?;

    // Q: Where is the step for spl-token authorize <TOKEN_ADDRESS> mint --disable?
    // NOTE This updates the token's mint authority from the wallet to DISABLED!

    msg!("5. Token mint proces completed successfully.");
    
    Ok(())
}
