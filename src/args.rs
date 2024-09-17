use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long)]
    pub configuration: Option<String>,
    #[arg(short, long, default_value_t = true)]
    pub immediatly_start_enabled_plugins: bool,
    #[arg(short, long)]
    pub show: Option<String>,
}
