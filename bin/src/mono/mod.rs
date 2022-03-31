mod service;
// use atb::includes::anyhow;
use actix_web::rt::System;
use service::build_http_service;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(short, long, default_value = "5003", env = "DAPP_PORT")]
    dapp_port: String,

    #[structopt(
        short,
        long,
        default_value = "http://127.0.0.1:5004",
        env = "HTTP_DISPATCHER_URL"
    )]
    http_dispatcher_url: String,
}

pub fn run(opts: Opts) -> std::io::Result<()> {
    let system = System::new();
    let Opts {
        dapp_port,
        http_dispatcher_url,
    } = opts;

    system
        .block_on(build_http_service(dapp_port, http_dispatcher_url))
        .map(|_| ())
        .map_err(Into::into)
}
