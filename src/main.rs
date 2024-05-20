use warp::Filter;
use docker_container_log_streamer::{WatchQueryParams, send_message};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let routes = warp::path("watch").and(
        warp::ws()
    )
    .and(warp::query::<WatchQueryParams>())
    .map(|ws: warp::ws::Ws, q: WatchQueryParams|{
        ws.on_upgrade(move |socket| send_message(
                socket,
                q.container_name.clone(),
                q.session_id.clone(),
                q.timeout
            )
        )
    });
    warp::serve(routes).run(([127,0,0,1],3090)).await;
}