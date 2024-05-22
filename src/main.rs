use warp::Filter;
use docker_container_log_streamer::{WatchQueryParams, send_message};
use clap::Parser;
use std::net::Ipv4Addr;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = "This is long about.")]
struct Args {
    #[arg(long)]
    stream_key: String,
    #[arg(long)]
    host: String,
    #[arg(long)]
    port: u16,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let args: Args  = Args::parse();
    let port: u16 = args.port;
    let host: Ipv4Addr = args.host.parse().expect("Unable to parse host as an IPv4 Address");  
    let routes = warp::path("watch")
    .map(move || args.stream_key.clone() )
    .and(warp::ws())

    .and(warp::query::<WatchQueryParams>())
    .map(|stream_key: String, ws: warp::ws::Ws, q: WatchQueryParams|{
        ws.on_upgrade(move |socket|
            send_message(
                socket,
                q.container_name.clone(),
                q.session_id.clone(),
                q.stream_key.clone(),
                stream_key.clone(),
                q.timeout
            )
        )
    });
    warp::serve(routes).run((host.octets(),port)).await;
}