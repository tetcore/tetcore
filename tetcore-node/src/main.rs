use clap::{Parser, Subcommand};
use tetcore_node::{NodeMode, TetcoreNode};
use tetcore_primitives::{Address, Hash32};

#[derive(Parser)]
#[command(name = "tetcore")]
#[command(version = "0.1.0")]
#[command(about = "Tetcore - The Sovereign Execution Framework for Intelligence Infrastructure", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(long, default_value = "client")]
        mode: String,
    },
    Keygen,
    Chain {
        #[command(subcommand)]
        command: ChainCommands,
    },
    Block {
        #[command(subcommand)]
        command: BlockCommands,
    },
    Run {
        #[arg(long)]
        validator: bool,
    },
}

#[derive(Subcommand)]
enum ChainCommands {
    Height,
    StateRoot,
}

#[derive(Subcommand)]
enum BlockCommands {
    Produce,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { mode } => {
            let node_mode = match mode.as_str() {
                "validator" => NodeMode::Validator,
                "client" => NodeMode::Client,
                "operator" => NodeMode::InferenceOperator,
                "relay" => NodeMode::Relay,
                _ => NodeMode::Client,
            };
            println!("Initializing Tetcore node in {} mode", mode);
            let mut node = TetcoreNode::new(node_mode);

            let alice = Address::from_bytes([0u8; 32]);
            let bob = Address::from_bytes([1u8; 32]);
            node.initialize_genesis(vec![(alice, 1_000_000_000), (bob, 500_000_000)]);

            println!("Genesis initialized successfully");
            println!("Alice address: {}", alice);
            println!("Bob address: {}", bob);
        }

        Commands::Keygen => {
            use ed25519_dalek::SigningKey;
            use rand::rngs::OsRng;
            use tetcore_primitives::PrivateKey;

            let signing_key = SigningKey::generate(&mut OsRng);
            let private_key = PrivateKey::from_signing_key(signing_key);
            let public_key = private_key.public_key();
            let address = Address::from_public_key(&public_key);

            println!("Private Key: {}", hex::encode(private_key.as_bytes()));
            println!("Public Key: {}", hex::encode(public_key.as_bytes()));
            println!("Address: {}", address);
        }

        Commands::Chain { command } => match command {
            ChainCommands::Height => {
                println!("Chain height: 0");
            }
            ChainCommands::StateRoot => {
                println!("Current state root: {}", Hash32::empty());
            }
        },

        Commands::Block { command } => match command {
            BlockCommands::Produce => {
                println!("Producing new block");
            }
        },

        Commands::Run { validator } => {
            let mode = if validator { "validator" } else { "client" };
            println!("Starting Tetcore node in {} mode...", mode);
            println!("Tetcore v0.1.0 - The Sovereign Execution Framework for Intelligence Infrastructure");
            println!();
            println!("Node running. Press Ctrl+C to stop.");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }
    }
}
