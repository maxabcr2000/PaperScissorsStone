mod http_dispatcher;
mod model;
mod service;
use service::rollup;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(
        short,
        long,
        default_value = "http://127.0.0.1:5004",
        env = "ROLLUP_HTTP_SERVER_URL"
    )]
    http_dispatcher_url: String,
}

pub async fn run(opts: Opts) {
    let Opts {
        http_dispatcher_url,
    } = opts;

    rollup(&http_dispatcher_url).await
}
