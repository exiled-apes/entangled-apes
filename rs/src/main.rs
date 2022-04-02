use borsh::de::BorshDeserialize;
use gumdrop::Options;
use mpl_token_metadata::instruction::update_metadata_accounts;
use mpl_token_metadata::state::{Creator, Data, Metadata};
use rusqlite::Connection;
use solana_client::rpc_client::RpcClient;
use solana_sdk::account::ReadableAccount;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::read_keypair_file;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse_args_default_or_exit();
    match args.clone().command {
        None => todo!(),
        Some(command) => match command {
            Command::UpdateCreatorsAndRoyalties(opts) => {
                update_creators_and_royalties(args, opts).await
            }
        },
    }
}

async fn update_creators_and_royalties(
    args: Args,
    opts: UpdateCreatorsAndRoyalties,
) -> Result<(), Box<dyn Error>> {
    let db = Connection::open(args.db)?;
    let rpc = RpcClient::new(args.rpc);
    let keypair = read_keypair_file(opts.keypair)?;

    let entanglements = &"SELECT mirc_mint_address, mono_mint_address FROM entanglements";
    let mut entanglements = db.prepare(entanglements)?;
    let entanglements = entanglements.query_map([], |row| {
        Ok(Entanglement {
            mirc_mint_address: row.get(0)?,
            mono_mint_address: row.get(0)?,
        })
    })?;

    for entanglement in entanglements {
        let entanglement = entanglement?;
        let mint = entanglement.mirc_mint_address.parse()?;
        let metadata_address = find_metadata_address(mint);
        let metadata = rpc.get_account(&metadata_address)?;
        let metadata = Metadata::deserialize(&mut metadata.data())?;

        let data = metadata.data;
        let creators = data.creators.unwrap();

        if creators[1].address == "4PC3jH8txRfFfa2n9AtuQMY3CZ2e3Bq8sbHEG784Zp6C".parse()? {
            let new_creators = Some(vec![
                creators[0].clone(),
                Creator {
                    address: "Hg5KGxWCwFWCsS5uTbKdjQv6pv21nG5kNwBch3zPKTFq".parse()?,
                    verified: false,
                    share: 65,
                },
                creators[2].clone(),
                creators[3].clone(),
                creators[4].clone(),
            ]);

            let new_data = Data {
                creators: new_creators,
                ..data
            };

            let instructions = vec![update_metadata_accounts(
                mpl_token_metadata::id(),
                metadata_address,
                keypair.pubkey(),
                None,
                Some(new_data),
                None,
            )];

            let signing_keypairs = &[&keypair];
            let recent_blockhash = rpc.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(
                &instructions,
                Some(&keypair.pubkey()),
                signing_keypairs,
                recent_blockhash,
            );

            let sig = rpc.send_transaction(&tx)?;
            eprintln!("{} {}", mint.to_string(), sig);
        }
        break;
    }
    Ok(())
}

#[derive(Clone, Debug, Options)]
struct Args {
    #[options(help = "slite db path")]
    db: String,
    #[options(help = "rpc server", default_expr = "default_rpc_url()", meta = "r")]
    rpc: String,
    #[options(command)]
    command: Option<Command>,
}

fn default_rpc_url() -> String {
    "https://api.mainnet-beta.solana.com".to_owned()
}

#[derive(Clone, Debug, Options)]
enum Command {
    UpdateCreatorsAndRoyalties(UpdateCreatorsAndRoyalties),
}

#[derive(Clone, Debug, Options)]
struct UpdateCreatorsAndRoyalties {
    #[options(help = "keypair", meta = "k")]
    keypair: String,
}

#[derive(Clone, Debug, Options)]
struct Entanglement {
    mirc_mint_address: String,
    mono_mint_address: String,
}

fn find_metadata_address(mint: Pubkey) -> Pubkey {
    let (address, _bump) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            mint.as_ref(),
        ],
        &mpl_token_metadata::id(),
    );
    address
}
