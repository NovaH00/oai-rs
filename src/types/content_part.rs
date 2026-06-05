use std::fmt;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    #[default]
    Png,
    Jpeg,
    Jpg,
    Webp,
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageFormat::Png => write!(f, "png"),
            ImageFormat::Jpeg => write!(f, "jpeg"),
            ImageFormat::Jpg => write!(f, "jpg"),
            ImageFormat::Webp => write!(f, "webp"),
        }
    }
}

/// A content segment within a chat message.
///
/// A message may consist of multiple content parts, allowing multimedia
/// (text, images, ...) to be combined in a single message.
#[derive(Clone)]
pub enum ContentPart {
    /// Plain text content.
    ///
    /// # Example
    ///
    /// ```rust
    /// ContentPart::Text {
    ///     text: "Hello, world!".into(),
    /// }
    /// ```
    Text {
        /// The text content.
        text: String,
    },

    /// Image content encoded as a Base64 string.
    ///
    /// The image will typically be converted into a
    /// `data:image/<format>;base64,<data>` URL when sent to an
    /// OpenAI-compatible API.
    ///
    /// # Example
    ///
    /// ```rust
    /// ContentPart::Image {
    ///     data: "...".into(),
    ///     format: ImageFormat::Png,
    /// }
    /// ```
    Image {
        /// Base64-encoded image data.
        data: String,

        /// The image format.
        format: ImageFormat,
    },
}

impl fmt::Debug for ContentPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentPart::Text { text } => {
                f.debug_struct("Text")
                    .field("text", text)
                    .finish()
            }

            ContentPart::Image { format, .. } => {
                f.debug_struct("Image")
                    .field("data", &"...")
                    .field("format", format)
                    .finish()
            }
        }
    }
}

impl ContentPart {
    /// Converts this content part into the JSON format expected by
    /// OpenAI-compatible chat completion APIs.
    ///
    /// Text content is converted to:
    ///
    /// ```json
    /// {
    ///   "type": "text",
    ///   "text": "Hello"
    /// }
    /// ```
    ///
    /// Image content is converted to:
    ///
    /// ```json
    /// {
    ///   "type": "image_url",
    ///   "image_url": {
    ///     "url": "data:image/png;base64,..."
    ///   }
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` representing this content part in the
    /// format required by OpenAI-compatible APIs.
    pub fn to_oai_json(&self) -> Value {
        match self {
            ContentPart::Text { text } => {
                json!({
                    "type": "text",
                    "text": text,
                })
            }

            ContentPart::Image { data, format } => {
                json!({
                    "type": "image_url",
                    "image_url": {
                        "url": format!(
                            "data:image/{};base64,{}",
                            format,
                            data
                        )
                    }
                })
            }
        }
    }
}
