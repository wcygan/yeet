use common::DEFAULT_ADDRESS;

#[derive(clap::Parser)]
pub struct Args {
    /// An address, e.g., 127.0.0.1:8080
    #[arg(short = 'a', long = "address", default_value = DEFAULT_ADDRESS)]
    pub address: String,
}
