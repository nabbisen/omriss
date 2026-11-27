//! A node's own text (RFC-085 §8.2).
//!
//! What a node says *itself*, as opposed to what its children say: a Markdown
//! section's body, a JSON value, none for a JSON group or list (its members are
//! nodes), the whole region for plain text. Search and statistics read this; the
//! format's adapter is the one that knows what it is, so it is the adapter's to
//! say ([`DocumentFormatAdapter::own_text`]), read from the node itself and not
//! looked up by id.

use crate::formats::focused_content::ValueKind;
use crate::range::ByteRange;

/// One node's own text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnText {
    /// The text, as a person reads it: a JSON string is decoded, its escapes
    /// resolved.
    pub text: String,
    /// Where the text sits in the source. When `exact`, `text` is the source at
    /// this range byte for byte, so an offset in the text is an offset in the
    /// source. When not (a decoded JSON string), the range is the whole value.
    pub range: ByteRange,
    pub exact: bool,
    /// What a JSON value is (text, number, on/off, empty); `None` for every other
    /// node. A card labels itself with it, and reading it here means a card needs
    /// no second decode.
    pub value_kind: Option<ValueKind>,
}
