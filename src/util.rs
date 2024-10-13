use std::path::Path;

pub fn get_content_type(path: &Path) -> &'static str {
    if let Some(ext) = path.extension() {
        match ext.to_str().unwrap() {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            _ => "application/octet-stream",
        }
    } else {
        "application/octet-stream"
    }
}
