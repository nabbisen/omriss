//! RFC-060 §10.2, level 2: every landmark omriss renders has a name from the
//! catalog, in every view.
//!
//! What assistive technology announces for a region is its accessible name; an
//! unnamed one is read as "landmark" and nothing else. A bare Dioxus `VirtualDom`
//! gives the template tree the WebView is handed, so the names can be checked
//! without a window. Each view is mounted the way the shell composes it (toolbar,
//! map, the one Writing Area, status bar); a landmark added later without a name
//! fails here.

use dioxus::prelude::*;
use omriss_core::DocumentFormat;
use omriss_ui::i18n::{Locale, t};
use omriss_ui::{EditorSession, FileTextProfile, ViewMode};

use crate::test_texts::{Rendered, elements_of_settled};
use crate::{
    DocumentMapPane, FocusedContentPane, OverviewPane, RawSourceView, SearchPanel, StatusBar,
    Toolbar, UnreadablePane, WelcomeScreen,
};

const MD: &str = "# One\nbody\n\n## Child\nx\n\n# Two\ny\n";
const NO_HEADINGS: &str = "just text\n";
const JSON: &str = "{\"name\": \"omriss\", \"tags\": [\"a\"]}\n";
const BROKEN_JSON: &str = "{\"a\": ";

/// A view of the application, as the shell routes to it.
#[derive(Clone, Copy, Debug, PartialEq)]
enum View {
    Welcome,
    /// Markdown, whole document (the overview's cards).
    Overview,
    /// Markdown with nothing to show as cards.
    OverviewEmpty,
    /// Markdown, a section focused: the Writing Area with its breadcrumb.
    Focus,
    /// The same with the preview open.
    FocusPreview,
    /// JSON, a member focused.
    FocusJson,
    /// Plain file text.
    PlainText,
    /// A file whose structure cannot be shown, plain text not open.
    Unreadable,
    /// Search open over the Writing Area.
    Search,
}

const VIEWS: [View; 9] = [
    View::Welcome,
    View::Overview,
    View::OverviewEmpty,
    View::Focus,
    View::FocusPreview,
    View::FocusJson,
    View::PlainText,
    View::Unreadable,
    View::Search,
];

thread_local! {
    static CASE: std::cell::Cell<(View, Locale)> =
        const { std::cell::Cell::new((View::Welcome, Locale::En)) };
}

fn open(source: &str, format: DocumentFormat) -> EditorSession {
    EditorSession::open_detected(
        source.to_string(),
        None,
        FileTextProfile::detect(source, false),
        format,
    )
    .unwrap()
}

fn session_for(view: View) -> EditorSession {
    match view {
        View::Welcome => EditorSession::new_empty(),
        View::Overview => open(MD, DocumentFormat::Markdown),
        View::OverviewEmpty => open(NO_HEADINGS, DocumentFormat::Markdown),
        View::Focus | View::FocusPreview | View::Search => {
            let mut s = open(MD, DocumentFormat::Markdown);
            s.settle_on_open();
            s
        }
        View::FocusJson => {
            let mut s = open(JSON, DocumentFormat::Json);
            s.settle_on_open();
            s
        }
        View::PlainText => {
            let mut s = open(MD, DocumentFormat::Markdown);
            s.show_raw();
            s
        }
        View::Unreadable => open(BROKEN_JSON, DocumentFormat::Json),
    }
}

/// The shell's composition (`omriss/src/shell/app.rs`): the same components in the
/// same places, routed by the same three questions.
fn host() -> Element {
    let (view, locale) = CASE.with(|c| c.get());
    let session = use_signal(|| session_for(view));
    let locale = use_signal(move || locale);
    let draft = use_signal(String::new);
    let status = use_signal(|| "status.ready".to_string());
    let selected_card = use_signal(|| 0usize);
    let preview_open = use_signal(move || view == View::FocusPreview);
    let request = use_signal(|| None);
    let recent_files = use_signal(|| vec!["/tmp/notes.md".to_string()]);

    let is_welcome = !session.read().document_open();
    let is_raw = session.read().is_raw();
    let unreadable = session.read().empty_map() == Some(omriss_ui::EmptyMap::Unreadable);
    let mode = session.read().view_mode();

    rsx! {
        div {
            Toolbar {
                session, locale, draft, status,
                on_open: move |()| {},
                on_save: move |()| {},
                on_save_as: move |()| {},
                search_available: !is_welcome,
                on_search: move |()| {},
            }
            if is_welcome {
                WelcomeScreen {
                    locale,
                    recent_files,
                    on_open: move |()| {},
                    on_new: move |()| {},
                    on_open_recent: move |_: String| {},
                }
            } else {
                div {
                    DocumentMapPane { session, locale, draft, status, request }
                    if is_raw {
                        RawSourceView { session, locale, draft, status }
                    } else if unreadable {
                        UnreadablePane { session, locale, draft, status }
                    } else {
                        match mode {
                            ViewMode::Outline | ViewMode::RawSource => rsx! {
                                OverviewPane { session, locale, draft, status, selected_card }
                            },
                            ViewMode::Focus(_) => rsx! {
                                FocusedContentPane { session, locale, draft, status, preview_open }
                            },
                        }
                    }
                }
            }
            StatusBar { session, locale, status, on_save_as: move |()| {} }
            if view == View::Search {
                SearchPanel {
                    session, locale,
                    on_close: move |()| {},
                    on_navigate: move |_| {},
                }
            }
        }
    }
}

fn render(view: View, locale: Locale) -> Vec<Rendered> {
    CASE.with(|c| c.set((view, locale)));
    elements_of_settled(host)
}

/// Landmarks, by tag or by role: what a screen reader lists as a region.
fn is_landmark(e: &Rendered) -> bool {
    matches!(
        e.tag.as_str(),
        "main" | "header" | "footer" | "nav" | "aside"
    ) || matches!(
        e.attr("role"),
        Some(
            "region"
                | "toolbar"
                | "search"
                | "navigation"
                | "banner"
                | "complementary"
                | "contentinfo"
                | "main"
        )
    )
}

fn name_of(e: &Rendered) -> Option<&str> {
    e.attr("aria-label")
        .or_else(|| e.attr("aria-labelledby"))
        .filter(|n| !n.trim().is_empty())
}

fn describe(e: &Rendered) -> String {
    format!("<{} role={:?}>", e.tag, e.attr("role"))
}

#[test]
fn every_landmark_has_a_name_in_every_view() {
    for locale in [Locale::En, Locale::Ja] {
        for view in VIEWS {
            let all = render(view, locale);
            let landmarks: Vec<&Rendered> = all.iter().filter(|e| is_landmark(e)).collect();
            // The walker must see something, or an empty result would pass.
            assert!(
                landmarks.len() >= 3,
                "{view:?}/{locale:?}: only {} landmarks found",
                landmarks.len()
            );
            let unnamed: Vec<String> = landmarks
                .iter()
                .filter(|e| name_of(e).is_none())
                .map(|e| describe(e))
                .collect();
            assert!(
                unnamed.is_empty(),
                "{view:?}/{locale:?}: unnamed landmarks {unnamed:?}"
            );
        }
    }
}

#[test]
fn the_main_region_is_always_the_writing_area() {
    for locale in [Locale::En, Locale::Ja] {
        for view in VIEWS {
            let all = render(view, locale);
            let mains: Vec<&Rendered> = all.iter().filter(|e| e.tag == "main").collect();
            assert_eq!(mains.len(), 1, "{view:?}/{locale:?}: exactly one main");
            assert_eq!(
                name_of(mains[0]),
                Some(t(locale, "focused_content.title")),
                "{view:?}/{locale:?}"
            );
        }
    }
}

#[test]
fn the_chrome_has_its_own_names() {
    for locale in [Locale::En, Locale::Ja] {
        for view in VIEWS {
            let all = render(view, locale);
            let named = |tag: &str| {
                all.iter()
                    .find(|e| e.tag == tag)
                    .and_then(name_of)
                    .map(str::to_string)
            };
            // P-4: the toolbar says what it is, not the name of a menu that does
            // not exist. P-2: the footer is the status bar.
            assert_eq!(
                named("header").as_deref(),
                Some(t(locale, "aria.toolbar")),
                "{view:?}/{locale:?}"
            );
            assert_eq!(
                named("footer").as_deref(),
                Some(t(locale, "aria.status_bar")),
                "{view:?}/{locale:?}"
            );
            if view != View::Welcome {
                assert_eq!(
                    named("aside").as_deref(),
                    Some(t(locale, "aria.document_map")),
                    "{view:?}/{locale:?}"
                );
            }
        }
    }
    // The words themselves, once, so a catalog edit cannot change them silently.
    assert_eq!(t(Locale::En, "aria.toolbar"), "Toolbar");
    assert_eq!(t(Locale::Ja, "aria.toolbar"), "ツールバー");
    assert_eq!(t(Locale::En, "aria.status_bar"), "Status bar");
    assert_eq!(t(Locale::Ja, "aria.status_bar"), "ステータスバー");
}

#[test]
fn the_landmarks_inside_the_writing_area_are_there_to_be_checked() {
    // The test above covers every landmark it finds; this pins that the ones inside
    // the Writing Area are found, so an empty walk cannot pass for a full one.
    for locale in [Locale::En, Locale::Ja] {
        let focus = render(View::Focus, locale);
        let has = |all: &[Rendered], tag: &str, name: &str| {
            all.iter().any(|e| e.tag == tag && name_of(e) == Some(name))
        };
        assert!(
            has(&focus, "nav", t(locale, "aria.breadcrumb")),
            "{locale:?}"
        );
        let preview = render(View::FocusPreview, locale);
        assert!(
            preview.iter().any(|e| e.attr("role") == Some("region")
                && name_of(e) == Some(t(locale, "editor.preview"))),
            "{locale:?}: the preview region"
        );
        let search = render(View::Search, locale);
        assert!(
            search.iter().any(|e| e.attr("role") == Some("search")
                && name_of(e) == Some(t(locale, "search.title"))),
            "{locale:?}: the search region"
        );
        let plain = render(View::PlainText, locale);
        assert!(
            plain.iter().any(|e| e.attr("role") == Some("region")
                && name_of(e) == Some(t(locale, "raw.source_label"))),
            "{locale:?}: plain file text"
        );
    }
}
