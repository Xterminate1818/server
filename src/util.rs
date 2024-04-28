use std::path::Path;

use tiny_http::Header;

pub type Response = tiny_http::Response<std::io::Cursor<Vec<u8>>>;
pub fn mime_from_file_ext(url: &str) -> &'static str {
  let as_path = Path::new(url);
  match as_path.extension().unwrap_or_default().as_encoded_bytes() {
    b"css" => "text/css",
    b"gif" => "image/gif",
    b"html" | b"htm" => "text/html",
    b"js" | b"mjs" => "text/javascript",
    b"svg" => "image/svg+xml",
    b"woff2" => "font/woff2",
    b"wasm" => "application/wasm",
    _ => "application/octet-stream",
  }
}

pub fn header(key: &str, value: &str) -> Header {
  Header::from_bytes(key.as_bytes(), value.as_bytes()).unwrap()
}
