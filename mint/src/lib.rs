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
    spl_associated_token_account::instruction as associated_token_account_instruction,
};

entrypoint!(process_instruction);

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let mint = next_account_info(accounts_iter)?; // Create a new mint (token-identifier)
    let token_account = next_account_info(accounts_iter)?; // Create a token account for the mint
    let mint_authority = next_account_info(accounts_iter)?; // Our wallet 
    let rent = next_account_info(accounts_iter)?; // Sysvar (like an .env var) but still an account
    let system_program = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?; // SPL Token Program
    let associated_token_program = next_account_info(accounts_iter)?; // SPL ATA Program


    msg!("1. Creating Mint account for the actual mint (token)...");
    msg!("Mint Address: {}", mint.key);
    // Invoke a Cross-program Invocation: 
    // NOTE Hits another program by sending accounts and doing stuff
    // Q: Is this the spl-token create-account <TOKEN_ADDRESS> command?
    // A: Yes and No! This invokes a CPI that passes some accounts and other args
    // to the SystemProgram, which will create an account that HOUSES THE MINT (mint account)
    invoke(
        // Instruction
        &system_instruction::create_account(
            &mint_authority.key, // Our wallet. We're the signer and payer for the tx
            &mint.key,
            LAMPORTS_PER_SOL,
            82, // Standard mint space size
            &token_program.key // Owner. SystemProgram makes SPL Token Program the owner
        ),
        // AccountInfo
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
    msg!("Mint Address: {}", mint.key);
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
    // A: Yes! This creates ATA based on keypair of the Mint account
    // NOTE When running this CLI command, the owner of account is our local keypair account
    // NOTE This create-account command literally adds the token account (ATA) inside owner's wallet!
    // Q: Is this the Token Metadata Program creating the Metadata Account for the token?
    // A: NO! Not sure where we create the metadata account, but this isn't it just yet!
    msg!("3. Creating associated token account for the mint/token and the wallet...");
    msg!("Token Account Address: {}", token_account.key);
    invoke(
        // Instruction
        &associated_token_account_instruction::create_associated_token_account(
            &mint_authority.key,
            &mint_authority.key,
            &mint.key, // Token Identifier/Address
        ),
        // AccountInfo
        &[
            mint.clone(),
            token_account.clone(),
            mint_authority.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ]
    )?;

    // Q: Is this spl-token mint <TOKEN_ADDRESS> <AMOUNT> <RECIPIENT_ADDRESS>?
    // A: Yes! This mints (increases supply of Token) and transfers new tokens
    // to owner's token account (default recipient token address) balance
    msg!("4. Minting token to the token account (i.e. give it 1 for NFT)...");
    msg!("Mint Address: {}", mint.key);
    msg!("Token Account Address: {}", token_account.key);
    invoke(
        // Instruction
        &token_instruction::mint_to(
            &token_program.key,
            &mint.key,
            &token_account.key, // The account to mint tokens to
            &mint_authority.key, // Mint's minting authority
            &[&mint_authority.key],
            1, // Amount of new tokens to mint
        )?,
        // AccountInfo
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
    // NOTE Cookbook TS: https://solanacookbook.com/references/token.html#how-to-set-authority-on-token-accounts-or-mints
    // msg!("5. Disable Mint authority (i.e., set/update token's mint authority type to 'null')");
    // msg!("Mint Address: {}", mint.key);
    // // Q: Is the Mint owner === authority?
    // msg!("Current Mint owner: {}", mint.owner);
    // invoke(
    //     // Instruction
    //     &token_instruction::set_authority(
    //         &token_program.key, // token_program_id, 
    //         owned_pubkey, // Current authority???
    //         None, // new_authority_pubkey Option<&Pubkey>
    //         token_instruction::AuthorityType::MintTokens, // AuthorityType.{MintTokens||FreezeAccount}
    //         &mint.owner, // owner_pubkey -- Should be the original wallet Pubkey???
    //         signer_pubkeys
    //     )?,
    //     // AccountInfo
    //     &[
    //         mint.clone(),
    //         mint_authority.clone(),
    //         token_program.clone(),
    //     ]

    // )


    msg!("5. Token mint process completed successfully.");
    // msg!("Final Mint Account: {:?}", mint);
    // msg!("Final Token Account: {:?}", token_account);
    
    Ok(())
}
