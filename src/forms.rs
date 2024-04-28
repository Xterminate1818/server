use std::collections::HashMap;

type Parse<'a, T> = Option<(T, &'a [u8])>;

fn parse_byte(i: &[u8], byte: u8) -> Parse<u8> {
  let first = i.first()?;
  if first == &byte {
    Some((byte, &i[1..]))
  } else {
    None
  }
}

fn parse_escaped_codepoint(i: &[u8]) -> Parse<u8> {
  let (_, i) = parse_byte(i, b'%')?;
  let slice = i.get(..2)?;
  let i = i.get(2..)?;
  let num = u8::from_str_radix(std::str::from_utf8(slice).ok()?, 16).ok()?;
  Some((num, i))
}

pub fn url_unescape(mut i: &[u8]) -> Option<Vec<u8>> {
  let mut bytes = Vec::with_capacity(i.len());
  while let Some(b) = i.first() {
    if b == &b'%' {
      let (escaped, new_i) = parse_escaped_codepoint(i)?;
      bytes.push(escaped);
      i = new_i;
    } else {
      bytes.push(*b);
      i = &i[1..];
    }
  }
  Some(bytes)
}

/// Parses form bodies with enctype=x-www-form-urlencoded
/// (default)
pub fn parse_urlencoded(i: &[u8]) -> Option<HashMap<String, Vec<u8>>> {
  let mut output = HashMap::new();
  let args = i.split(|b| b == &b'&');
  for arg in args {
    let unescaped = url_unescape(arg)?;
    let (key, value) = unescaped.split_once(|b| b == &b'=')?;
    output.insert(String::from_utf8_lossy(key).to_string(), value.to_vec());
  }
  Some(output)
}

pub fn parse_form(request: &mut tiny_http::Request) -> Option<HashMap<String, Vec<u8>>> {
  let content_type = request.headers().iter().find_map(|header| {
    if header.field.as_str().to_ascii_lowercase() == "content-type" {
      Some(header.value.clone())
    } else {
      None
    }
  })?;
  if let Some(s) = content_type
    .as_str()
    .strip_prefix("multipart/form-data; boundary=")
  {
    let mut boundary = vec![b'-', b'-'];
    boundary.append(&mut s.as_bytes().to_vec());
    return parse_multipart_form(request, boundary);
  } else if content_type.as_str() == "application/x-www-form-urlencoded" {
    let mut bytes = vec![];
    request.as_reader().read_to_end(&mut bytes).ok()?;
    return parse_urlencoded(&bytes);
  } else {
    log::warn!("Unrecognized form type: {}", content_type.as_str());
    None
  }
}

fn position_of(slice: &[u8], pattern: &[u8]) -> Option<usize> {
  for index in 0..slice.len() {
    if slice[..index].ends_with(pattern) {
      return Some(index - pattern.len());
    }
  }
  None
}

/// Parses forms with enctype=multipart/form-data
pub fn parse_multipart_form(
  request: &mut tiny_http::Request,
  boundary: Vec<u8>,
) -> Option<HashMap<String, Vec<u8>>> {
  let mut bytes = vec![];
  request.as_reader().read_to_end(&mut bytes).unwrap();
  // Strip out cruft to make it easy to parse
  let mut bytes = bytes
    .strip_prefix(b"--")?
    .strip_suffix(b"--\r\n")?
    .strip_suffix(boundary.as_slice())?;

  let mut map = HashMap::new();

  while !bytes.is_empty() {
    bytes = &bytes[boundary.len()..];
    // Find next boundary, or end of body
    let end = position_of(bytes, &boundary).unwrap_or(bytes.len());
    let slice = &bytes[..end].strip_suffix(b"\r\n")?;
    bytes = &bytes[end..];
    // Find end of headers
    let end = position_of(slice, b"\r\n\r\n").unwrap_or(0);
    let headers = &slice[..end];
    let string = String::from_utf8_lossy(headers);
    let name = string
      .lines()
      .find_map(|line| line.strip_prefix("Content-Disposition: "))?
      .split("; ")
      .find_map(|arg| arg.strip_prefix("name="))?
      .trim();
    let name = name.strip_prefix('"').unwrap_or(name);
    let name = name.strip_suffix('"').unwrap_or(name);
    let body = &slice[end + 4..];
    map.insert(name.into(), body.to_vec());
  }
  Some(map)
}
