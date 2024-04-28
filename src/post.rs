use crate::util::*;

pub struct PostHandler;

impl PostHandler {
  async fn fishbowl_endpoint(&self, file: &[u8]) -> Vec<u8> {
    fishbowl::make_gif(&file, fishbowl::make_quickdraw().await.unwrap())
      .await
      .unwrap()
  }

  pub async fn handle_post(&self, request: &mut tiny_http::Request) -> Option<Response> {
    let form = crate::forms::parse_form(request)?;

    match request.url() {
      "/projects/fishbowl" => {
        let image = form.get("image")?;
        let bytes = self.fishbowl_endpoint(&image).await;
        return Some(Response::from_data(bytes).with_header(header("Content-Type", "image/gif")));
      }
      _ => {
        log::warn!("Unhandled POST at {}", request.url());
        return None;
      }
    }
  }
}
