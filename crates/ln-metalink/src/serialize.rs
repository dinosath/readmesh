use super::{HashType, Metalink, MetalinkError};

/// Serialize a Metalink to XML string.
pub fn to_xml(metalink: &Metalink) -> Result<String, MetalinkError> {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<metalink xmlns=\"urn:ietf:params:xml:ns:metalink\">\n");

    // Origin
    if let Some(ref origin) = metalink.origin {
        let dynamic = if origin.dynamic { "true" } else { "false" };
        if let Some(priority) = origin.priority {
            xml.push_str(&format!(
                "  <origin dynamic=\"{}\" priority=\"{}\" />\n",
                dynamic, priority
            ));
        } else {
            xml.push_str(&format!(
                "  <origin dynamic=\"{}\" />\n",
                dynamic
            ));
        }
    }

    // File
    xml.push_str(&format!(
        "  <file name=\"{}\">\n",
        xml_escape(&metalink.file.name)
    ));
    xml.push_str(&format!("    <size>{}</size>\n", metalink.file.size));

    for hash in &metalink.file.hashes {
        let type_str = match &hash.hash_type {
            HashType::Sha256 => "sha-256",
            HashType::Blake3 => "blake3",
            HashType::Other(s) => s.as_str(),
        };
        xml.push_str(&format!(
            "    <hash type=\"{}\">{}</hash>\n",
            type_str, hash.value
        ));
    }

    for url_entry in &metalink.file.urls {
        let mut attrs = Vec::new();
        if let Some(p) = url_entry.priority {
            attrs.push(format!("priority=\"{}\"", p));
        }
        if let Some(ref loc) = url_entry.location {
            attrs.push(format!("location=\"{}\"", xml_escape(loc)));
        }
        if let Some(pref) = url_entry.preference {
            attrs.push(format!("preference=\"{}\"", pref));
        }
        let attrs_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        };
        xml.push_str(&format!(
            "    <url{}>{}</url>\n",
            attrs_str,
            xml_escape(&url_entry.url)
        ));
    }

    if let Some(ref sig) = metalink.file.signature {
        xml.push_str(&format!(
            "    <signature mediatype=\"{}\">{}</signature>\n",
            xml_escape(&sig.mediatype),
            sig.signature
        ));
    }

    xml.push_str("  </file>\n");
    xml.push_str("</metalink>\n");

    Ok(xml)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Metalink;

    #[test]
    fn serialize_roundtrip() {
        let hash = blake3::hash(b"test");
        let meta = Metalink::builder("test.txt", 100)
            .add_blake3(&hash)
            .add_url("https://example.com/test.txt", Some(1), Some("US".into()))
            .build();

        let xml = to_xml(&meta).unwrap();
        assert!(xml.contains("<metalink"));
        assert!(xml.contains("test.txt"));
        assert!(xml.contains("https://example.com/test.txt"));
        assert!(xml.contains("blake3"));
    }
}
