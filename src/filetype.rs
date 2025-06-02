use std::path::Path;

use crate::icons::{
    ICON_BOX, ICON_CROSS, ICON_ECLIPSE, ICON_FILE_AUDIO, ICON_FILE_QUESTION, ICON_FILM, ICON_IMAGE,
    ICON_MUSIC, ICON_PACKAGE, ICON_PERSON_STANDING, ICON_SETTINGS, ICON_SHAPES,
};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum FileType {
    #[default]
    Unknown,
    Animation,
    Mp4,
    Texture,
    Prefab,
    Json,

    WwiseBank,
    WwiseStream,
    WwisePack,

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
                "bnk" => return Some(Self::WwiseBank),
                "wem" => return Some(Self::WwiseStream),
                "pck" => return Some(Self::WwisePack),
                _ => {}
            }
        }

        // TODO(cohae): Potential future check: Messiah files tend to be UUIDs with hyphens, PYC files are UUIDs without hyphens.

        None
    }

    pub fn icon(&self) -> char {
        match self {
            FileType::Unknown => ICON_FILE_QUESTION,
            FileType::Animation => ICON_PERSON_STANDING,
            FileType::Mp4 => ICON_FILM,
            FileType::Texture => ICON_IMAGE,
            FileType::Prefab => ICON_SHAPES,
            FileType::UnknownMessiah => ICON_CROSS,
            FileType::Model => ICON_BOX,
            FileType::Material => ICON_ECLIPSE,
            FileType::Json => ICON_SETTINGS,
            FileType::WwiseBank => ICON_FILE_AUDIO,
            FileType::WwiseStream => ICON_MUSIC,
            FileType::WwisePack => ICON_PACKAGE,
        }
    }
}
