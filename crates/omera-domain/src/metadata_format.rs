//! The AI image generators whose metadata Berry AI Studio understands.

use serde::{Deserialize, Serialize};

/// The generation platform a piece of embedded metadata came from.
///
/// Each variant corresponds to a known on-disk metadata format. Detection and
/// parsing of each format is implemented in `berry-metadata` (full parsers
/// arrive in M2); this enum is the shared vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MetadataFormat {
    /// AUTOMATIC1111 / SD.Next PNGInfo `parameters` text chunk.
    #[default]
    A1111,
    /// InvokeAI (Dream / sd-metadata / invokeai_metadata).
    InvokeAI,
    /// NovelAI.
    NovelAI,
    /// Stable Diffusion (SD 1.x metadata variants).
    StableDiffusion,
    /// Fooocus / FooocusMRE.
    Fooocus,
    /// ComfyUI PNGInfo.
    ComfyUI,
    /// EasyDiffusion.
    EasyDiffusion,
    /// Stable Swarm.
    StableSwarm,
}

impl MetadataFormat {
    /// All known formats, in the order they are checked when sniffing.
    pub const ALL: &'static [MetadataFormat] = &[
        Self::A1111,
        Self::InvokeAI,
        Self::NovelAI,
        Self::StableDiffusion,
        Self::Fooocus,
        Self::ComfyUI,
        Self::EasyDiffusion,
        Self::StableSwarm,
    ];

    /// A stable identifier used for database storage and display.
    pub const fn id(self) -> &'static str {
        match self {
            Self::A1111 => "a1111",
            Self::InvokeAI => "invokeai",
            Self::NovelAI => "novelai",
            Self::StableDiffusion => "sd",
            Self::Fooocus => "fooocus",
            Self::ComfyUI => "comfyui",
            Self::EasyDiffusion => "easydiffusion",
            Self::StableSwarm => "stableswarm",
        }
    }
}

impl std::fmt::Display for MetadataFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ids_are_unique() {
        let ids: Vec<&str> = MetadataFormat::ALL.iter().map(|f| f.id()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len(), "duplicate format id");
    }

    #[test]
    fn display_uses_stable_id() {
        assert_eq!(MetadataFormat::A1111.to_string(), "a1111");
        assert_eq!(MetadataFormat::ComfyUI.to_string(), "comfyui");
    }

    #[test]
    fn serde_roundtrip() {
        for format in MetadataFormat::ALL {
            let json = serde_json::to_string(format).unwrap();
            let back: MetadataFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(back, *format);
        }
    }
}
