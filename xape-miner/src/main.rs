use borsh::de::BorshDeserialize;
use gumdrop::Options;
use metaplex_token_metadata::state::Metadata;
use rusqlite::{params, Connection};
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::ReadableAccount, pubkey::Pubkey};
use std::{error::Error, fmt::Debug, fs::File, io::BufRead, io::BufReader};

#[derive(Clone, Debug, Options)]
struct Args {
    #[options(command)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Options)]
enum Command {
    #[options(help = "load the mint files into sqlite")]
    LoadMints(LoadMints),
    #[options(help = "populate entanglements table from mints")]
    LoadEntanglements(LoadEntanglements),
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
struct LoadEntanglements {
    #[options(help = "sqlite db path")]
    db: String,
}

#[derive(Clone, Debug, Deserialize)]
struct JSONMeta {
    name: String,
    attributes: Vec<JSONAttr>,
}

#[derive(Clone, Debug, Deserialize)]
struct JSONAttr {
    value: String,
    trait_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse_args_default_or_exit();
    match args.clone().command {
        None => todo!(),
        Some(command) => match command {
            Command::LoadMints(opts) => load_mints(args, opts).await,
            Command::LoadEntanglements(opts) => load_entanglements(args, opts).await,
            // Command::MineMetas(opts) => mine_metas(args, opts),
        },
    }
}

async fn load_entanglements(_args: Args, opts: LoadEntanglements) -> Result<(), Box<dyn Error>> {
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
        "SELECT mint_address, meta_address, meta_name, meta_uri, inmate_number
             FROM mirc_mints
             ORDER BY mint_address",
    )?;

    let mirc_mint_iter = stmt.query_map([], |row| try_mint_row(row))?;
    for mirc_row in mirc_mint_iter {
        let mirc_row = mirc_row.unwrap();

        let mono_row = db.query_row(
            "SELECT mint_address, meta_address, meta_name, meta_uri, inmate_number
                FROM mono_mints
                WHERE inmate_number like ?1
                LIMIT 1
            ",
            params![mirc_row.inmate_number],
            |row| try_mint_row(row),
        );
        let mono_row = mono_row.unwrap();

        // normal entanglement
        db.execute(
            "INSERT INTO entanglements
                     (mirc_mint_address, mono_mint_address) values
                     (               ?1,                ?2)",
            params![mirc_row.mint_address, mono_row.mint_address,],
        )?;
    }

    let mut stmt = db.prepare(
        "SELECT mint_address, meta_address, meta_name, meta_uri, inmate_number
             FROM mirc_mints
             WHERE mint_address NOT IN (SELECT mirc_mint_address FROM entanglements)
             ORDER BY mint_address",
    )?;

    // TODO expect 518-30 ghost entanglements here

    let mirc_mint_iter = stmt.query_map([], |row| try_mint_row(row))?;
    for mirc_row in mirc_mint_iter {
        let mirc_row = mirc_row.unwrap();
        let mono_row = db.query_row(
            "SELECT mint_address, meta_address, meta_name, meta_uri, inmate_number
                FROM mono_mints
                WHERE mint_address NOT IN (SELECT mono_mint_address FROM entanglements)
                ORDER BY genesis_order
                LIMIT 1
            ",
            params![],
            |row| try_mint_row(row),
        );
        let mono_row = mono_row.unwrap();

        // ghost entanglement
        db.execute(
            "INSERT INTO entanglements
                     (mirc_mint_address, mono_mint_address) values
                     (               ?1,                ?2)",
            params![mirc_row.mint_address, mono_row.mint_address,],
        )?;
    }

    // TODO expect 518 entanglements here

    Ok(())
}

async fn load_mints(_args: Args, opts: LoadMints) -> Result<(), Box<dyn Error>> {
    let rpc = RpcClient::new(opts.rpc);
    let db = Connection::open(opts.db)?;

    db.execute("DROP TABLE IF EXISTS mirc_mints", params![])?;
    db.execute(
        "CREATE TABLE mirc_mints (
             mint_address text primary key,
             meta_address text unique,
             meta_name text,
             meta_uri text,
             inmate_number text
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
            .json::<JSONMeta>()
            .await?;

        let mut inmate_number = "".to_string();
        for attribute in jm.attributes {
            if attribute.trait_type == "Inmate number" {
                inmate_number = attribute.value;
            }
        }
        db.execute(
            "INSERT INTO mirc_mints
            (mint_address, meta_address, meta_name, meta_uri, inmate_number) values
            (          ?1,           ?2,        ?3,       ?4,            ?5)",
            params![
                mint_address.to_string(),
                meta_address.to_string(),
                metadata.data.name,
                metadata.data.uri,
                inmate_number,
            ],
        )?;
    }

    db.execute("DROP TABLE IF EXISTS mono_mints", params![])?;
    db.execute(
        "CREATE TABLE mono_mints (
             mint_address text primary key,
             meta_address text unique,
             meta_name text,
             meta_uri text,
             inmate_number text,
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

        db.execute(
            "INSERT INTO mono_mints
            (mint_address, meta_address, meta_name, meta_uri, inmate_number, genesis_order) values
            (          ?1,           ?2,        ?3,       ?4,            5?,            ?6)",
            params![
                mint_address.to_string(),
                meta_address.to_string(),
                metadata.data.name,
                metadata.data.uri,
                inmate_number,
                genesis_order,
            ],
        )?;
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
struct MintRow {
    mint_address: String,
    meta_address: String,
    meta_name: String,
    meta_uri: String,
    inmate_number: String,
}

fn try_mint_row(row: &rusqlite::Row) -> Result<MintRow, rusqlite::Error> {
    let mint_row = MintRow {
        mint_address: row.get(0)?,
        meta_address: row.get(1)?,
        meta_name: row.get(2)?,
        meta_uri: row.get(3)?,
        inmate_number: row.get(4)?,
    };
    Ok(MintRow {
        meta_name: mint_row.meta_name.trim_matches(char::from(0)).to_string(),
        meta_uri: mint_row.meta_uri.trim_matches(char::from(0)).to_string(),
        ..mint_row
    })
}
