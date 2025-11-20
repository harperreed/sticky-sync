// ABOUTME: Parser for StickiesState.plist metadata
// ABOUTME: Extracts color, position, and window state from plist dictionaries

use crate::{Result, StickyError};
use plist::{Dictionary, Value};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct StickyMetadata {
    pub color_index: i64,
    pub frame: String,
    pub is_floating: bool,
}

impl StickyMetadata {
    pub fn from_plist_dict(dict: &Dictionary) -> Result<Self> {
        let color_index = dict
            .get("Color")
            .and_then(|v| v.as_signed_integer())
            .unwrap_or(0);

        let frame = dict
            .get("Frame")
            .and_then(|v| v.as_string())
            .unwrap_or("{{100, 100}, {250, 250}}")
            .to_string();

        let is_floating = dict
            .get("Floating")
            .and_then(|v| v.as_boolean())
            .unwrap_or(false);

        Ok(Self {
            color_index,
            frame,
            is_floating,
        })
    }

    pub fn color_name(&self) -> &str {
        match self.color_index {
            0 => "yellow",
            1 => "blue",
            2 => "green",
            3 => "pink",
            4 => "purple",
            5 => "gray",
            _ => "yellow",
        }
    }
}

pub fn read_stickies_state(path: &Path) -> Result<HashMap<String, StickyMetadata>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let value = Value::from_file(path)?;

    // Try array format first (.SavedStickiesState)
    if let Some(array) = value.as_array() {
        let mut result = HashMap::new();

        for entry in array {
            if let Some(dict) = entry.as_dictionary() {
                // Extract UUID from dictionary value
                if let Some(Value::String(uuid)) = dict.get("UUID") {
                    let metadata = StickyMetadata::from_plist_dict(dict)?;
                    result.insert(uuid.to_lowercase(), metadata);
                }
            }
        }

        return Ok(result);
    }

    // Fall back to dictionary format (StickiesState.plist)
    let dict = value
        .as_dictionary()
        .ok_or_else(|| StickyError::Config("Invalid plist format".into()))?;

    let mut result = HashMap::new();

    for (uuid, metadata_value) in dict {
        if let Some(metadata_dict) = metadata_value.as_dictionary() {
            let metadata = StickyMetadata::from_plist_dict(metadata_dict)?;
            result.insert(uuid.clone(), metadata);
        }
    }

    Ok(result)
}
