use warp::Filter;
use docker_container_log_streamer::{WatchQueryParams, send_message, Logger};
use clap::Parser;
use std::net::Ipv4Addr;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = "Parameters when running the log streamer app.")]
struct Args {
    /// Stream Key used for Authentication
    #[arg(long)]
    stream_key: String,
    
    /// Host in IPV4 IP Address format.
    #[arg(long)]
    host: String,
    
    /// The port number to use.
    #[arg(long)]
    port: u16,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let args: Args  = Args::parse();
    let port: u16 = args.port;
    let host: Ipv4Addr = args.host.parse().expect("Unable to parse host as IPv4 Address");  
    let logs_dir_route = warp::path("logs")
    .and(warp::fs::dir(
        format!("{}",Logger::log_dir())
    ));
    let ws_route = warp::path("watch")    
    .map(move || args.clone() )
    .and(warp::ws())
    .and(warp::query::<WatchQueryParams>())
    .map(|args: Args, ws: warp::ws::Ws, q: WatchQueryParams|{
        ws.on_upgrade(move |socket|
            send_message(
                socket,
                q.container_name.clone(),
                q.since_in_minutes.clone(),
                q.session_id.clone(),
                q.stream_key.clone(),
                args.stream_key.clone(),
                q.timeout,
                q.save_logs,
            )
        )
    });
    let routes = ws_route.or(logs_dir_route);
    warp::serve(routes).run((host.octets(),port)).await;
}