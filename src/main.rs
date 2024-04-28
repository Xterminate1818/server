#![feature(slice_split_once)]
mod forms;
mod get;
mod logging;
mod post;
mod util;

use std::{io::Write, sync::Arc};

use get::Router;
use post::PostHandler;
use tiny_http::{Request, Server};

// Exit process if its already running
fn graceful_cleanup() {
  const PID_PATH: &'static str = "./temp/pid";
  match {
    let s = std::fs::read_to_string(PID_PATH).unwrap_or("".to_string());
    s.parse::<u32>().ok()
  } {
    Some(last_pid) => {
      std::process::Command::new("pkill")
        .arg(format!("{}", last_pid))
        .output()
        .unwrap();
    },
    None => {
      log::warn!("Did not find previous PID");
    },
  };
  let pid = std::process::id();
  let mut file = std::fs::File::create(PID_PATH).unwrap();
  file.write(format!("{}", pid).as_bytes()).unwrap();
}

#[tokio::main]
async fn main() {
  // graceful_cleanup();
  let _h = logging::init();
  // let ssl = tiny_http::SslConfig {
  //   certificate:
  // include_bytes!("../lgatlin.dev.cert").to_vec(),
  //   private_key:
  // include_bytes!("../../lgatlin.dev.priv").to_vec(), };
  // let server = Server::https("0.0.0.0:443", ssl).unwrap();
  let server = Server::http("0.0.0.0:8080").unwrap();

  let router = Arc::new(Router::new());
  let post = Arc::new(PostHandler);
  router.create_redirect("/", "/home").await;
  router
    .create_redirect("/favicon.ico", "/resources/favicon.svg")
    .await;

  router.create_route("/home", "/hyper-build/home.html").await;
  router
    .create_route("/projects", "/hyper-build/projects.html")
    .await;
  router
    .create_route("/about", "hyper-build/about.html")
    .await;
  router
    .create_route(
      "/projects/http-server",
      "hyper-build/projects/http-server.html",
    )
    .await;
  router
    .create_route(
      "/projects/html-templating",
      "hyper-build/projects/html-templating.html",
    )
    .await;
  router
    .create_route("/projects/forte", "hyper-build/projects/forte.html")
    .await;
  router
    .create_route("/projects/fishbowl", "hyper-build/projects/fishbowl.html")
    .await;
  router
    .create_route(
      "/projects/math-interpreter",
      "hyper-build/projects/math-interpreter.html",
    )
    .await;
  router
    .create_route("/projects/nd-range", "hyper-build/projects/nd-range.html")
    .await;
  router
    .create_route(
      "/projects/fractal-explorer",
      "hyper-build/projects/fractal-explorer.html",
    )
    .await;
  router
    .create_route("/projects/pokedex", "hyper-build/projects/pokedex.html")
    .await;
  router
    .create_route(
      "/projects/stock-trading",
      "hyper-build/projects/stocktrading.html",
    )
    .await;

  for request in server.incoming_requests() {
    log::info!("{:?} AT {:?}", request.method(), request.url(),);
    let router = router.clone();
    let post = post.clone();
    tokio::spawn(async move {
      handle_connection(request, router, post).await;
    });
  }
}

async fn handle_connection(
  mut request: Request,
  route: Arc<Router>,
  post: Arc<PostHandler>,
) {
  use tiny_http::*;
  let response = match request.method() {
    Method::Get | Method::Head => route.construct_response(request.url()).await,
    Method::Post => match post.handle_post(&mut request).await {
      Some(response) => response,
      None => return,
    },
    _ => Response::from_string("Error occurred").with_status_code(405),
  };
  let _ = request.respond(response);
}
