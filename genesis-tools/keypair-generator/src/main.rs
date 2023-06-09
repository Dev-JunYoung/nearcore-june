use blockbuster::Utc;
use blockbuster::DepthGuard;
use blockbuster::DEPTH_COUNTER;
use blockbuster::TOTAL_COUNTER;
use blockbuster::PATH_COUNT_MAP;
use blockbuster::FN_COUNT_MAP;
use blockbuster::print_file_path_and_function_name;


use std::fs;
use std::path::{Path, PathBuf};

use clap::{Arg, Command};

use near_crypto::{InMemorySigner, KeyType, SecretKey, Signer};
use nearcore::get_default_home;

fn generate_key_to_file(account_id: &str, key: SecretKey, path: &PathBuf) -> std::io::Result<()> {
print_file_path_and_function_name!();

    let signer = InMemorySigner::from_secret_key(account_id.parse().unwrap(), key);
    signer.write_to_file(path.as_path())
}

fn main() {
print_file_path_and_function_name!();

    let default_home = get_default_home();
    let matches = Command::new("Key-pairs generator")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .about("Generates: access key-pairs, validation key-pairs, network key-pairs")
        .arg(
            Arg::new("home")
                .long("home")
                .default_value_os(default_home.as_os_str())
                .help("Directory for config and data (default \"~/.near\")")
                .takes_value(true),
        )
        .arg(
            Arg::new("account-id")
                .long("account-id")
                .takes_value(true),
        )
        .arg(
            Arg::new("generate-config")
                .long("generate-config")
                .help("Whether to generate a config file when generating keys. Requires account-id to be specified.")
                .takes_value(false),
        )
        .subcommand(
            Command::new("signer-keys").about("Generate signer keys.").arg(
                Arg::new("num-keys")
                    .long("num-keys")
                    .takes_value(true)
                    .help("Number of signer keys to generate. (default 3)"),
            ),
        )
        .subcommand(
            Command::new("node-key").about("Generate key for the node communication."),
        )
        .subcommand(Command::new("validator-key").about("Generate staking key."))
        .get_matches();

    let home_dir = matches.value_of("home").map(|dir| Path::new(dir)).unwrap();
    fs::create_dir_all(home_dir).expect("Failed to create directory");
    let account_id = matches.value_of("account-id");
    let generate_config = matches.is_present("generate-config");

    match matches.subcommand() {
        Some(("signer-keys", args)) => {
            let num_keys = args
                .value_of("num-keys")
                .map(|x| x.parse().expect("Failed to parse number keys."))
                .unwrap_or(3usize);
            let keys: Vec<SecretKey> =
                (0..num_keys).map(|_| SecretKey::from_random(KeyType::ED25519)).collect();
            let mut pks = vec![];
            for (i, key) in keys.into_iter().enumerate() {
                println!("Key#{}", i);
                println!("PK: {}", key.public_key());
                println!();
                if generate_config {
                    let account_id = account_id
                        .expect("Account id must be specified if --generate-config is used");
                    let key_file_name = format!("signer{}_key.json", i);
                    let mut path = home_dir.to_path_buf();
                    path.push(&key_file_name);
                    if let Err(e) = generate_key_to_file(account_id, key.clone(), &path) {
                        eprintln!("Error writing key to {}: {}", path.display(), e);
                        return;
                    }
                }

                pks.push(key.public_key());
            }
            let pks: Vec<_> = pks.into_iter().map(|pk| pk.to_string()).collect();
            println!("List of public keys:");
            println!("{}", pks.join(","));
        }
        Some(("validator-key", _)) => {
            let key = SecretKey::from_random(KeyType::ED25519);
            println!("PK: {}", key.public_key());
            if generate_config {
                let account_id =
                    account_id.expect("Account id must be specified if --generate-config is used");
                let mut path = home_dir.to_path_buf();
                path.push(nearcore::config::VALIDATOR_KEY_FILE);
                if let Err(e) = generate_key_to_file(account_id, key, &path) {
                    eprintln!("Error writing key to {}: {}", path.display(), e);
                    return;
                }
            }
        }
        Some(("node-key", _args)) => {
            let key = SecretKey::from_random(KeyType::ED25519);
            println!("PK: {}", key.public_key());
            if generate_config {
                let mut path = home_dir.to_path_buf();
                path.push(nearcore::config::NODE_KEY_FILE);
                if let Err(e) = generate_key_to_file("node", key, &path) {
                    eprintln!("Error writing key to {}: {}", path.display(), e);
                    return;
                }
            }
        }
        _ => unreachable!(),
    }
}
