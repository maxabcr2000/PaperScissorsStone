use atb::includes::anyhow;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Commands {
    #[structopt(name = "mono")]
    Mono,

    #[structopt(name = "version")]
    Version,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "PaperScissorsStone",
    about = "Rust backend for PaperScissorsStone",
    no_version,
    global_settings = &[AppSettings::DisableVersion, AppSettings::ColoredHelp]
)]
struct PaperScissorsStone {
    #[structopt(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    let opts = PaperScissorsStone::from_args();

    match opts.command {
        Commands::Mono => Ok(()),
        Commands::Version => {
            print_version();
            Ok(())
        }
    }

    //#TODO: command line => setup server url, port, player hp, player damage
    //#TODO: HTTP server
    //#TODO: only match for two players, no need for database now, only one room only
    //#TODO: game flow
}

fn print_version() {
    println!(
        "Package: {:?}\nVersion: {:?}\nAuthor: {:?}\nDescription: {:?}\nSupport: {:?}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS"),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_REPOSITORY")
    );
}
