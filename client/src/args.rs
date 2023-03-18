#[derive(clap::Parser)]
pub struct Args {
    #[arg(short = 'a', long = "address")]
    pub address: String,
    #[arg(short = 'p', long = "port")]
    pub port: u16,
}
