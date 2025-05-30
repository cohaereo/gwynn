use std::path::Path;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum FileType {
    #[default]
    Unknown,
    Animation,
    Mp4,
    Texture,
    Prefab,
    Json,

    // Messiah files
    UnknownMessiah,
    Model,
    Material,
}

impl FileType {
    pub fn guess_from_path(path: &str) -> Option<Self> {
        let path = Path::new(path);
        if let Some(extension) = path.extension().map(|p| p.to_string_lossy()) {
            match extension.as_ref() {
                "mp4" => return Some(Self::Mp4),
                "anim" => return Some(Self::Animation),
                "1" | "4" | "6" => return Some(Self::Texture),
                "etsb" => return Some(Self::Prefab),
                "json" => return Some(Self::Json),
                _ => {}
            }
        }

        // TODO(cohae): Potential future check: Messiah files tend to be UUIDs with hyphens, PYC files are UUIDs without hyphens.

        None
    }
}
