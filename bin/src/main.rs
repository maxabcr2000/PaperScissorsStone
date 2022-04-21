mod mono;
// use atb::includes::anyhow;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Commands {
    #[structopt(name = "mono")]
    Mono(mono::Opts),

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

fn main() -> std::io::Result<()> {
    let opts = PaperScissorsStone::from_args();

    match opts.command {
        Commands::Mono(m_opts) => mono::run(m_opts),
        Commands::Version => {
            print_version();
            Ok(())
        }
    }
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
