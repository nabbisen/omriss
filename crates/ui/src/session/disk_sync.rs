//! What the session knows about the file on disk, and Reload (RFC-068 §7.6–§7.7).
//!
//! The question a save asks is "is the file still what this document was made from?".
//! Its answer is the [`ContentStamp`] of the bytes last read or written. The wrinkle is
//! Reload: it replaces the document with the file's new contents as **one undo step**,
//! and undoing it brings back the user's unsaved text, which was made from the *old*
//! file. If the stamp stayed at the new file's, saving that text would overwrite the
//! other program's changes without asking, and the user has just seen those changes and
//! chosen not to lose their own. So the stamp follows the undo history: undoing a reload
//! puts the old stamp back, redoing it puts the new one back.
//!
//! History is not asked which step was undone (`omriss-core` does not say, and revisions
//! are fresh after every undo). Instead each reload records the two document texts it
//! moved between, by stamp, with the disk stamp and profile that belong to each; after any
//! undo or redo, a document whose text is one of those gets its recorded state back.
//! That is exact for the states a reload creates, and any text in between, which no
//! reload made, keeps the stamp it has: the disk has not changed by undoing a typo.

use omriss_core::{ByteRange, EditError};

use crate::file::content_stamp::ContentStamp;
use crate::file::file_profile::FileTextProfile;

/// A document text a reload moved from or to, and what was true of the disk then.
#[derive(Debug, Clone)]
pub(super) struct DiskState {
    /// The stamp of the document's text (not of a file: no BOM).
    text: ContentStamp,
    /// The stamp of the file this text was made from.
    disk: Option<ContentStamp>,
    profile: FileTextProfile,
}

impl super::EditorSession {
    /// The stamp of the file as this document last knew it, or `None` without a file.
    /// A save asks whether the file on disk still has it.
    pub fn disk_stamp(&self) -> Option<ContentStamp> {
        self.disk_stamp
    }

    /// Records that the file now holds the bytes `stamp` stands for: after opening it
    /// or saving to it. Forgets earlier reloads: the disk is no longer what they
    /// recorded, so undoing past them must not bring an older stamp back.
    pub fn set_disk_stamp(&mut self, stamp: Option<ContentStamp>) {
        self.disk_stamp = stamp;
        self.disk_states.clear();
    }

    /// Replaces the document with the file's contents `text` (read from disk, with
    /// its `profile` and byte `stamp`) as one undo step, and marks it saved.
    ///
    /// Undo returns the text and profile the document had, and the stamp of the file
    /// that text was made from, so saving it asks again. When `text` is what the
    /// document already holds, nothing is written to history and only the file's stamp
    /// and profile are recorded. Focus returns to the outline (nothing is remapped: a
    /// node that still resolves in a rewritten document is a coincidence), and no
    /// structure notice is raised: the user asked for the file as it is.
    pub fn reload_from_disk(
        &mut self,
        text: String,
        profile: FileTextProfile,
        stamp: ContentStamp,
    ) -> Result<(), EditError> {
        let before = DiskState {
            text: ContentStamp::of(self.document.source().as_bytes()),
            disk: self.disk_stamp,
            profile: self.profile.clone(),
        };
        if let Some((start, end, replacement)) = minimal_replacement(self.document.source(), &text)
        {
            let base = self.document.revision();
            self.document
                .replace_range(ByteRange::new(start, end)?, replacement, base)?;
        }
        self.view.reset_to_outline();
        self.structure_change = None;
        self.raw_editing = false;
        self.raw_base = None;
        self.prune_dead_history();

        self.saved = super::SavedFingerprint::of(self.document.source(), self.document.revision());
        let after = DiskState {
            text: ContentStamp::of(self.document.source().as_bytes()),
            disk: Some(stamp),
            profile,
        };
        self.profile = after.profile.clone();
        self.disk_stamp = after.disk;
        self.disk_states.push(before);
        self.disk_states.push(after);
        Ok(())
    }

    /// After an undo or redo: if the document's text is one a reload made or left,
    /// the disk stamp and profile that belong to it come back.
    pub(super) fn resync_disk_state(&mut self) {
        if self.disk_states.is_empty() {
            return;
        }
        let now = ContentStamp::of(self.document.source().as_bytes());
        if let Some(state) = self.disk_states.iter().rev().find(|s| s.text == now) {
            self.disk_stamp = state.disk;
            self.profile = state.profile.clone();
        }
    }
}

/// The smallest single replacement that turns `old` into `new`, or `None` if they are
/// the same: everything both share at the start and at the end stays, so the history
/// entry is the size of the change, not of the document. Kept off the middle of a
/// character and of a CRLF pair.
fn minimal_replacement(old: &str, new: &str) -> Option<(usize, usize, String)> {
    if old == new {
        return None;
    }
    let (o, n) = (old.as_bytes(), new.as_bytes());
    let boundary = |at: usize| {
        old.is_char_boundary(at)
            && new.is_char_boundary(at)
            && !(at > 0 && o.get(at - 1) == Some(&b'\r') && o.get(at) == Some(&b'\n'))
            && !(at > 0 && n.get(at - 1) == Some(&b'\r') && n.get(at) == Some(&b'\n'))
    };
    let mut prefix = o.iter().zip(n).take_while(|(a, b)| a == b).count();
    while prefix > 0 && !boundary(prefix) {
        prefix -= 1;
    }
    let max_suffix = o.len().min(n.len()) - prefix;
    let mut suffix = o
        .iter()
        .rev()
        .zip(n.iter().rev())
        .take(max_suffix)
        .take_while(|(a, b)| a == b)
        .count();
    let end_boundary = |s: usize| {
        let (oe, ne) = (o.len() - s, n.len() - s);
        old.is_char_boundary(oe)
            && new.is_char_boundary(ne)
            && !(oe > 0 && o.get(oe - 1) == Some(&b'\r') && o.get(oe) == Some(&b'\n'))
            && !(ne > 0 && n.get(ne - 1) == Some(&b'\r') && n.get(ne) == Some(&b'\n'))
    };
    while suffix > 0 && !end_boundary(suffix) {
        suffix -= 1;
    }
    Some((
        prefix,
        o.len() - suffix,
        new[prefix..n.len() - suffix].to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::minimal_replacement;

    fn apply(old: &str, new: &str) -> String {
        match minimal_replacement(old, new) {
            None => old.to_string(),
            Some((start, end, replacement)) => {
                format!("{}{}{}", &old[..start], replacement, &old[end..])
            }
        }
    }

    #[test]
    fn the_replacement_turns_old_into_new_for_awkward_texts() {
        let samples = [
            "",
            "a",
            "# T\n\nbody\n",
            "# T\r\n\r\nbody\r\n",
            "caf\u{e9} \u{3042}\u{3044} x",
            "caf\u{e8} \u{3042}\u{3046} x",
            "a\r\nb",
            "a\nb",
            "a\r\n\r\nb",
            "\r\n",
            "\n",
            "xx\u{1f600}yy",
            "xx\u{1f601}yy",
            "same",
        ];
        for old in samples {
            for new in samples {
                assert_eq!(apply(old, new), new, "{old:?} -> {new:?}");
            }
        }
    }

    #[test]
    fn identical_texts_need_no_replacement() {
        assert!(minimal_replacement("same", "same").is_none());
    }

    #[test]
    fn only_the_changed_part_is_replaced() {
        let (start, end, replacement) =
            minimal_replacement("# T\n\nold body\n\n## B\n", "# T\n\nnew body\n\n## B\n").unwrap();
        assert_eq!((start, end), (5, 8));
        assert_eq!(replacement, "new");
    }

    #[test]
    fn a_crlf_pair_is_never_cut() {
        // "a\r\nb" -> "a\nb": the shared prefix "a" then differing; the shared "\n" suffix
        // must not leave the replacement starting between the CR and the LF.
        let (start, _, _) = minimal_replacement("a\r\nb", "a\nb").unwrap();
        assert!(start <= 1, "start {start}");
    }
}
