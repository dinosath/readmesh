use quick_xml::events::Event;
use quick_xml::Reader;

use super::{HashEntry, HashType, Metalink, MetalinkError, Origin, SignatureEntry, UrlEntry};

/// Parse a Metalink4 XML document string.
pub fn from_xml(xml: &str) -> Result<Metalink, MetalinkError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut metalink: Option<Metalink> = None;

    // Track what element we're inside for associating text content.
    enum Context {
        None,
        Size,
        Hash,
        Url,
        Signature,
    }
    let mut context = Context::None;

    // Temporary values for building elements.
    let mut file_name = String::new();
    let mut current_size: u64 = 0;
    let mut hashes: Vec<HashEntry> = Vec::new();
    let mut urls: Vec<UrlEntry> = Vec::new();
    let mut signature: Option<SignatureEntry> = None;
    let mut origin: Option<Origin> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag_name.as_str() {
                    "metalink" => {
                        metalink = Some(Metalink {
                            origin: None,
                            file: super::FileEntry {
                                name: String::new(),
                                size: 0,
                                hashes: vec![],
                                urls: vec![],
                                signature: None,
                            },
                        });
                    }
                    "origin" => {
                        let mut dynamic = false;
                        let mut priority = None;
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value =
                                String::from_utf8_lossy(attr.value.as_ref()).to_string();
                            match key.as_str() {
                                "dynamic" => dynamic = value == "true",
                                "priority" => priority = value.parse().ok(),
                                _ => {}
                            }
                        }
                        origin = Some(Origin { dynamic, priority });
                    }
                    "file" => {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "name" {
                                file_name = String::from_utf8_lossy(attr.value.as_ref())
                                    .to_string();
                            }
                        }
                        current_size = 0;
                        hashes.clear();
                        urls.clear();
                        signature = None;
                    }
                    "size" => {
                        context = Context::Size;
                    }
                    "hash" => {
                        let mut hash_type = HashType::Sha256;
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "type" {
                                let val =
                                    String::from_utf8_lossy(attr.value.as_ref()).to_string();
                                hash_type = match val.as_str() {
                                    "sha-256" => HashType::Sha256,
                                    "blake3" => HashType::Blake3,
                                    other => HashType::Other(other.to_string()),
                                };
                            }
                        }
                        hashes.push(HashEntry {
                            hash_type,
                            value: String::new(),
                        });
                        context = Context::Hash;
                    }
                    "url" => {
                        let mut priority = None;
                        let mut location = None;
                        let mut preference = None;
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let value =
                                String::from_utf8_lossy(attr.value.as_ref()).to_string();
                            match key.as_str() {
                                "priority" => priority = value.parse().ok(),
                                "location" => location = Some(value),
                                "preference" => preference = value.parse().ok(),
                                _ => {}
                            }
                        }
                        urls.push(UrlEntry {
                            url: String::new(),
                            priority,
                            location,
                            preference,
                        });
                        context = Context::Url;
                    }
                    "signature" => {
                        let mut mediatype = String::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "mediatype" {
                                mediatype = String::from_utf8_lossy(attr.value.as_ref())
                                    .to_string();
                            }
                        }
                        signature = Some(SignatureEntry {
                            mediatype,
                            signature: String::new(),
                        });
                        context = Context::Signature;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e
                    .unescape()
                    .map_err(|e| MetalinkError::Parse(e.to_string()))?
                    .trim()
                    .to_string();

                if text.is_empty() {
                    continue;
                }

                match context {
                    Context::Size => {
                        if let Ok(size) = text.parse() {
                            current_size = size;
                        }
                    }
                    Context::Hash => {
                        if let Some(last) = hashes.last_mut()
                            && last.value.is_empty() {
                                last.value = text;
                            }
                    }
                    Context::Url => {
                        if let Some(last) = urls.last_mut()
                            && last.url.is_empty() {
                                last.url = text;
                            }
                    }
                    Context::Signature => {
                        if let Some(ref mut sig) = signature
                            && sig.signature.is_empty() {
                                sig.signature = text;
                            }
                    }
                    Context::None => {}
                }
                context = Context::None;
            }
            Ok(Event::Empty(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                // Handle self-closing origin
                if tag_name == "origin" {
                    let mut dynamic = false;
                    let mut priority = None;
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value =
                            String::from_utf8_lossy(attr.value.as_ref()).to_string();
                        match key.as_str() {
                            "dynamic" => dynamic = value == "true",
                            "priority" => priority = value.parse().ok(),
                            _ => {}
                        }
                    }
                    origin = Some(Origin { dynamic, priority });
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                context = Context::None;
                if tag_name == "file"
                    && let Some(ref mut m) = metalink {
                        m.file = super::FileEntry {
                            name: std::mem::take(&mut file_name),
                            size: current_size,
                            hashes: std::mem::take(&mut hashes),
                            urls: std::mem::take(&mut urls),
                            signature: std::mem::take(&mut signature),
                        };
                    }
                if tag_name == "metalink" {
                    if let Some(ref mut m) = metalink {
                        m.origin = origin.take();
                    }
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(MetalinkError::Parse(format!("XML error: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    metalink.ok_or_else(|| MetalinkError::Parse("no metalink element found".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_metalink() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<metalink xmlns="urn:ietf:params:xml:ns:metalink">
  <file name="test.txt">
    <size>1024</size>
    <hash type="sha-256">abc123def456</hash>
    <url priority="1">https://example.com/test.txt</url>
  </file>
</metalink>"#;

        let meta = from_xml(xml).unwrap();
        assert_eq!(meta.file.name, "test.txt");
        assert_eq!(meta.file.size, 1024);
        assert_eq!(meta.file.hashes.len(), 1);
        assert_eq!(meta.file.hashes[0].value, "abc123def456");
        assert_eq!(meta.file.urls.len(), 1);
        assert_eq!(meta.file.urls[0].url, "https://example.com/test.txt");
        assert_eq!(meta.file.urls[0].priority, Some(1));
    }

    #[test]
    fn parse_metalink_with_multiple_urls() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<metalink xmlns="urn:ietf:params:xml:ns:metalink">
  <file name="chapter.html">
    <size>5000</size>
    <hash type="blake3">deadbeef1234</hash>
    <url priority="1">https://source.com/ch1</url>
    <url priority="2" location="EU">iroh:blob:abc</url>
    <url priority="3">magnet:?xt=urn:btih:def</url>
  </file>
</metalink>"#;

        let meta = from_xml(xml).unwrap();
        assert_eq!(meta.file.urls.len(), 3);
        assert_eq!(meta.file.hashes[0].hash_type, HashType::Blake3);
        assert_eq!(meta.urls_by_scheme("iroh").len(), 1);
        assert_eq!(meta.urls_by_scheme("magnet").len(), 1);
    }
}
