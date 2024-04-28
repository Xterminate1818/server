use std::{
  collections::HashMap,
  path::{Component, Path, PathBuf},
};

use tiny_http::Header;
use tokio::sync::RwLock;

use crate::util::*;

/// Makes the uri less spooky - removes leading /, rejects
/// ../ and ./
pub fn rewrite_url(url: &str) -> String {
  let (url, query) = url.split_once('?').unwrap_or((url, ""));
  let target_path = Path::new(url);
  let mut corrected_path = PathBuf::new();
  for c in target_path.components() {
    match c {
      Component::Normal(_) | Component::RootDir => {
        corrected_path.push(c);
      },
      Component::ParentDir => {
        corrected_path.pop();
      },
      _ => {},
    }
  }
  corrected_path.to_string_lossy().to_string()
}

pub struct Router {
  /// Reinterpret A as B server-side
  route_table: RwLock<HashMap<String, String>>,
  /// Send a 301 redirect from A to B
  routing_table: RwLock<HashMap<String, String>>,
}

impl Router {
  const NOT_FOUND: &'static [u8] = include_bytes!("../hyper-build/404.html");

  pub fn new() -> Self {
    Self {
      route_table: RwLock::new(Default::default()),
      routing_table: RwLock::new(Default::default()),
    }
  }

  fn redirect(to: &str) -> Response {
    Response::from_data(&[])
      .with_status_code(301)
      .with_header(header("Location", to))
  }

  fn not_found() -> Response {
    Response::from_data(Self::NOT_FOUND).with_status_code(404)
  }

  pub async fn construct_response(&self, request_url: &str) -> Response {
    let sanitized_url = rewrite_url(request_url);
    if sanitized_url != request_url {
      return Self::redirect(&sanitized_url);
    }

    let routed_url = self.perform_routing(&sanitized_url).await;
    if let Some(response) = self.perform_redirecting(&routed_url).await {
      return response;
    }

    let mime_type = mime_from_file_ext(&routed_url);

    match self.get_resource(&routed_url).await {
      Some(bytes) => Response::from_data(bytes)
        .with_header(header("Content-Type", &mime_type)),
      None => Self::not_found(),
    }
  }

  pub async fn create_route(&self, from: &str, to: &str) {
    let from = rewrite_url(from);
    let to = rewrite_url(to);
    let mut w = self.route_table.write().await;
    w.insert(from.into(), to.into());
  }

  async fn perform_routing(&self, uri: &str) -> String {
    let r = self.route_table.read().await;
    r.get(uri).cloned().unwrap_or(uri.to_string())
  }

  pub async fn create_redirect(&self, from: &str, to: &str) {
    let from = rewrite_url(from);
    let to = rewrite_url(to);
    let mut w = self.routing_table.write().await;
    w.insert(from.into(), to.into());
  }

  async fn perform_redirecting(&self, uri: &str) -> Option<Response> {
    let r = self.routing_table.read().await;
    r.get(uri).map(|url| Self::redirect(url))
  }

  pub async fn get_resource(&self, uri: &str) -> Option<Vec<u8>> {
    let uri = uri.trim_start_matches('/');
    tokio::fs::read(&uri).await.ok()
  }
}
