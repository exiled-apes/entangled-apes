use borsh::de::BorshDeserialize;
use gumdrop::Options;
use metaplex_token_metadata::{
    instruction::update_metadata_accounts,
    state::{Data, Metadata},
};
use rusqlite::{params, Connection};
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    account::ReadableAccount, pubkey::Pubkey, signature::read_keypair_file, signer::Signer,
    transaction::Transaction,
};
use std::{error::Error, fmt::Debug, fs::File, io::BufRead, io::BufReader};
use tokio::join;

#[derive(Clone, Debug, Options)]
struct Args {
    #[options(command)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Options)]
enum Command {
    #[options(help = "fix some busted ghosts")]
    FixGhosts(FixGhosts),
    #[options(help = "load the mint files into sqlite")]
    LoadBlanks(LoadBlanks),
    #[options(help = "load the mint files into sqlite")]
    LoadMints(LoadMints),
    #[options(help = "populate entanglements table from mints")]
    PlanEntanglements(PlanEntanglements),
}

#[derive(Clone, Debug, Options)]
struct FixGhosts {
    #[options(help = "rpc server")]
    rpc: String,
    #[options(help = "update authority")]
    update_authority: String,
}

#[derive(Clone, Debug, Options)]
struct LoadBlanks {
    #[options(help = "blanks csv file")]
    csv_file: String,
    #[options(help = "sqlite db path")]
    db: String,
}

#[derive(Clone, Debug, Options)]
struct LoadMints {
    #[options(help = "sqlite db path")]
    db: String,
    #[options(help = "mirc mints file")]
    mirc_file: String,
    #[options(help = "mono mints file")]
    mono_file: String,
    #[options(help = "rpc server")]
    rpc: String,
}

#[derive(Clone, Debug, Options)]
struct PlanEntanglements {
    #[options(help = "sqlite db path")]
    db: String,
}

#[derive(Clone, Debug, Deserialize)]
struct MircMeta {
    name: String,
    image: String,
    attributes: Vec<MircAttr>,
}

#[derive(Clone, Debug, Deserialize)]
struct MircAttr {
    value: String,
    trait_type: String,
}

#[derive(Clone, Debug, Deserialize)]
struct MonoMeta {
    name: String,
    image: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse_args_default_or_exit();
    match args.clone().command {
        None => todo!(),
        Some(command) => match command {
            Command::FixGhosts(opts) => fix_ghosts(opts).await,
            Command::LoadBlanks(opts) => load_blanks(opts).await,
            Command::PlanEntanglements(opts) => plan_entanglements(opts).await,
            Command::LoadMints(opts) => load_mints(opts).await,
        },
    }
}

async fn fix_ghosts(opts: FixGhosts) -> Result<(), Box<dyn Error>> {
    let rpc = RpcClient::new(opts.rpc);

    let fixes = [
        (
            "C1zuSLjxYW3gSekUAMeSVt74dn826EUHoP1Pfjm8sh5Y",
            "https://arweave.net/p7_PP3_b610qH7o0sM-n7twSpX94lF9TLwRHaSkKrYE",
        ),
        (
            "4Q1TGHyQBnYDUKuFwQqYDkiCEDkh4FSrSpUp9rGEg7Kk",
            "https://arweave.net/pMFdkuXKf9P6JrkHIeZuqNlcC-kIImc6y0NzZqRMj7s",
        ),
        (
            "4GFfjpBYVoHcmnGbhqSXkS7CHHg3dWGSGV2G66PgRxkr",
            "https://arweave.net/0erYimpankdX2yUCvkXEyoaxmCI1-dMAxIDtE-lhyjU",
        ),
        (
            "7qtG5sLk7Z3uU4Bfy8hwuTt9mUWqZW3y9eASQsJDBbtV",
            "https://arweave.net/1r_twlSBjrOZXq3lhEKYZvSiuKNugVcUNb1yIH5PaLU",
        ),
    ];

    for (mint_address, meta_uri) in fixes {
        let mint_address = mint_address.parse().unwrap();
        let meta_address = find_metadata_address(mint_address);
        let metadata = rpc.get_account(&meta_address)?;
        let metadata = Metadata::deserialize(&mut metadata.data())?;

        let update_authority = read_keypair_file(opts.update_authority.clone())?;

        if {
            let chain_uri = metadata.data.uri.trim_matches(char::from(0));
            chain_uri != meta_uri
        } {
            eprintln!(" got {}\nwant {}", metadata.data.uri, meta_uri);

            let (recent_blockhash, _) = rpc.get_recent_blockhash().unwrap();

            let data = Data {
                uri: meta_uri.to_string(),
                ..metadata.data
            };

            let instruction = update_metadata_accounts(
                metaplex_token_metadata::id(),
                meta_address,
                metadata.update_authority,
                None,
                Some(data),
                None,
            );

            let instructions = &[instruction];

            let signing_keypairs = &[&update_authority];

            let tx = Transaction::new_signed_with_payer(
                instructions,
                Some(&update_authority.pubkey()),
                signing_keypairs,
                recent_blockhash,
            );

            // let res = rpc.simulate_transaction(&tx);
            // let res = res.expect("could not simulate tx");
            // eprintln!("{:?}", res);

            let res = rpc.send_and_confirm_transaction(&tx);
            let sig = res.expect("could not confirm tx");
            eprintln!("{:?}", sig);
        }
    }

    Ok(())
}

async fn load_blanks(opts: LoadBlanks) -> Result<(), Box<dyn Error>> {
    let db = Connection::open(opts.db)?;
    db.execute("DROP TABLE IF EXISTS blanks", params![])?;
    db.execute(
        "CREATE TABLE blanks (
             mono_mint   text primary key,
             mirc_name   text unique,
             mirc_number numeric unique
        )",
        params![],
    )?;

    let file = File::open(opts.csv_file)?;
    let mut rdr = csv::Reader::from_reader(BufReader::new(file));
    for result in rdr.records() {
        let record = result?;
        let mirc_name = record.get(0).unwrap();
        let mono_mint = record.get(14).unwrap();
        let mirc_number = mirc_name.strip_prefix("Exiled Ape #").unwrap_or("");
        db.execute(
            "INSERT INTO blanks
            (mono_mint, mirc_name, mirc_number) values
            (       ?1,        ?2,          ?3)",
            params![mono_mint, mirc_name, mirc_number,],
        )?;
    }

    Ok(())
}

async fn plan_entanglements(opts: PlanEntanglements) -> Result<(), Box<dyn Error>> {
    let db = Connection::open(opts.db)?;
    db.execute("DROP TABLE IF EXISTS entanglements", params![])?;
    db.execute(
        "CREATE TABLE entanglements (
             mirc_mint_address text primary key,
             mono_mint_address text unique
        )",
        params![],
    )?;

    let mut stmt = db.prepare(
        "SELECT mint_address, meta_address, meta_name, meta_uri, inmate_number, image_uri
             FROM mirc_mints
             ORDER BY mint_address",
    )?;

    let mirc_mint_iter = stmt.query_map([], |row| try_mint_row(row))?;
    for mirc_row in mirc_mint_iter {
        let mirc_row = mirc_row.unwrap();

        let mono_row = db.query_row(
            "SELECT mint_address, meta_address, meta_name, meta_uri, inmate_number, image_uri
                 FROM mono_mints
                 WHERE inmate_number like ?1
                 LIMIT 1",
            params![mirc_row.inmate_number],
            |row| try_mint_row(row),
        );

        if let Ok(mono_row) = mono_row {
            // normal entanglement
            db.execute(
                "INSERT INTO entanglements
                     (mirc_mint_address, mono_mint_address) values
                     (               ?1,                ?2)",
                params![mirc_row.mint_address, mono_row.mint_address,],
            )?;
        }
    }

    let mut stmt = db.prepare(
        "SELECT mono_mint, mirc_name, mirc_number
             FROM blanks
             ORDER BY mirc_number",
    )?;

    let blanks_iter = stmt.query_map([], |row| try_blank_row(row))?;
    for blank_row in blanks_iter {
        let blank_row = blank_row.unwrap();
        let meta_name = format!("ExiledApe {}/518", blank_row.mirc_number);

        let mirc_row = db.query_row(
            "SELECT mint_address, meta_address, meta_name, meta_uri, inmate_number, image_uri
                 FROM mirc_mints
                 WHERE meta_name like ?1
                 LIMIT 1",
            params![meta_name],
            |row| try_mint_row(row),
        );
        let mirc_row = mirc_row.unwrap();

        // ghost entanglement
        db.execute(
            "INSERT INTO entanglements
            (mirc_mint_address, mono_mint_address) values
            (               ?1,                ?2)",
            params![mirc_row.mint_address, blank_row.mono_mint],
        )?;
    }

    Ok(())
}

async fn load_mints(opts: LoadMints) -> Result<(), Box<dyn Error>> {
    let (x, y) = join!(load_mono_mints(opts.clone()), load_mirc_mints(opts.clone()));

    if x.is_err() || y.is_err() {
        let mut msg: Vec<String> = vec![];
        if x.is_err() {
            msg.push(format!("load_mono_mints: {:?}", x));
        }
        if y.is_err() {
            msg.push(format!("load_mirc_mints: {:?}", y));
        }
        return Err(msg.join("\n").into());
    }

    Ok(())
}

fn find_metadata_address(mint: Pubkey) -> Pubkey {
    let (metadata, _bump) = Pubkey::find_program_address(
        &[
            metaplex_token_metadata::state::PREFIX.as_bytes(),
            metaplex_token_metadata::id().as_ref(),
            mint.as_ref(),
        ],
        &metaplex_token_metadata::id(),
    );
    metadata
}

#[derive(Debug)]
struct BlankRow {
    mono_mint: String,
    mirc_name: String,
    mirc_number: u32,
}

fn try_blank_row(row: &rusqlite::Row) -> Result<BlankRow, rusqlite::Error> {
    Ok(BlankRow {
        mono_mint: row.get(0)?,
        mirc_name: row.get(1)?,
        mirc_number: row.get(2)?,
    })
}

#[derive(Debug)]
struct MintRow {
    mint_address: String,
    meta_address: String,
    meta_name: String,
    meta_uri: String,
    inmate_number: String,
    image_uri: String,
}

fn try_mint_row(row: &rusqlite::Row) -> Result<MintRow, rusqlite::Error> {
    let mint_row = MintRow {
        mint_address: row.get(0)?,
        meta_address: row.get(1)?,
        meta_name: row.get(2)?,
        meta_uri: row.get(3)?,
        inmate_number: row.get(4)?,
        image_uri: row.get(5)?,
    };
    Ok(MintRow {
        meta_name: mint_row.meta_name.trim_matches(char::from(0)).to_string(),
        meta_uri: mint_row.meta_uri.trim_matches(char::from(0)).to_string(),
        ..mint_row
    })
}

async fn load_mirc_mints(opts: LoadMints) -> Result<(), Box<dyn Error>> {
    let rpc = RpcClient::new(opts.rpc);
    let db = Connection::open(opts.db)?;

    db.execute("DROP TABLE IF EXISTS mirc_mints", params![])?;
    db.execute(
        "CREATE TABLE mirc_mints (
             mint_address text primary key,
             meta_address text unique,
             meta_name text,
             meta_uri text,
             inmate_number text,
             image_uri text
         )",
        params![],
    )?;
    let mirc_file = File::open(opts.mirc_file)?;
    let mirc_reader = BufReader::new(mirc_file);
    for line in mirc_reader.lines() {
        let mint_address = line.unwrap().parse()?;
        let meta_address = find_metadata_address(mint_address);
        let metadata = rpc.get_account(&meta_address)?;
        let metadata = Metadata::deserialize(&mut metadata.data())?;

        let jm = reqwest::get(metadata.data.clone().uri)
            .await?
            .json::<MircMeta>()
            .await?;

        let mut inmate_number = "".to_string();
        for attribute in jm.attributes {
            if attribute.trait_type == "Inmate number" {
                inmate_number = attribute.value;
            }
        }

        db.execute(
            "INSERT INTO mirc_mints
            (mint_address, meta_address, meta_name, meta_uri, inmate_number, image_uri) values
            (          ?1,           ?2,        ?3,       ?4,            ?5,        ?6)",
            params![
                mint_address.to_string(),
                meta_address.to_string(),
                metadata.data.name,
                metadata.data.uri,
                inmate_number,
                jm.image,
            ],
        )?;
    }

    Ok(())
}

async fn load_mono_mints(opts: LoadMints) -> Result<(), Box<dyn Error>> {
    let rpc = RpcClient::new(opts.rpc);
    let db = Connection::open(opts.db)?;

    db.execute("DROP TABLE IF EXISTS mono_mints", params![])?;
    db.execute(
        "CREATE TABLE mono_mints (
             mint_address text primary key,
             meta_address text unique,
             meta_name text,
             meta_uri text,
             inmate_number text,
             image_uri text,
             genesis_order integer
         )",
        params![],
    )?;
    let mono_file = File::open(opts.mono_file)?;
    let mono_reader = BufReader::new(mono_file);

    let mut genesis_order = 0;
    for line in mono_reader.lines() {
        genesis_order = genesis_order + 1;

        let mint_address = line.unwrap().parse()?;
        let meta_address = find_metadata_address(mint_address);
        let metadata = rpc.get_account(&meta_address)?;
        let metadata = Metadata::deserialize(&mut metadata.data())?;
        let inmate_number = metadata.data.name.strip_prefix("Degen Ape #").unwrap_or("");

        let mut image = "".to_string();
        match reqwest::get(metadata.data.clone().uri).await {
            Err(e) => eprintln!("a {:?}", e),
            Ok(res) => match res.json::<MonoMeta>().await {
                Ok(jm) => image = jm.image,
                Err(e) => eprintln!("b {:?}", e),
            },
        }

        db.execute(
            "INSERT INTO mono_mints
            (mint_address, meta_address, meta_name, meta_uri, inmate_number, image_uri, genesis_order) values
            (          ?1,           ?2,        ?3,       ?4,            ?5,        ?6,            ?7)",
            params![
                mint_address.to_string(),
                meta_address.to_string(),
                metadata.data.name,
                metadata.data.uri,
                inmate_number,
                image,
                genesis_order,
            ],
        )?;
    }

    Ok(())
}
