use gumdrop::Options;
use rusqlite::{params, Connection};
use std::{fmt::Debug, error::Error};

#[derive(Clone, Debug, Options)]
struct Args {
    #[options(command)]
    command: Option<Command>,
}

#[derive(Clone, Debug, Options)]
enum Command {
    #[options(help = "load the mint files into sqlite")]
    LoadMints(LoadMints),
    #[options(help = "load metadata for all mints into sqlite")]
    MineMetas(MineMetas),
}

#[derive(Clone, Debug, Options)]
struct LoadMints {
    #[options(help = "slite db path")]
    db_path: String,
    #[options(help = "mirc mints file")]
    mirc_mint_file: String,
    #[options(help = "mono mints file")]
    mono_mint_file: String,
}

#[derive(Clone, Debug, Options)]
struct MineMetas {}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse_args_default_or_exit();
    match args.clone().command {
        None => todo!(),
        Some(command) => match command {
            Command::LoadMints(opts) => load_mints(args, opts),
            Command::MineMetas(opts) => mine_metas(args, opts),
        },
    }
}

fn load_mints(_args: Args, opts: LoadMints) -> Result<(), Box<dyn Error>> {
    let db = Connection::open(opts.db_path)?;
    db.execute(
        "create table if not exists mirc_mints (
             token_address text primary key,
             metadata_address text unique
         )",
        params![],
    )?;
    db.execute(
        "create table if not exists mono_mints (
             token_address text primary key,
             metadata_address text unique
         )",
        params![],
    )?;

    Ok(())
}

fn mine_metas(_args: Args, _opts: MineMetas) -> Result<(), Box<dyn Error>> {
    println!("{}", "mine_metas");
    // let client = RpcClient::new(app_options.rpc_url);

    Ok(())
}