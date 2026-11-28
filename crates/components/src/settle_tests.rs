//! RFC-086 §3.2: a settled application does no work.
//!
//! Dioxus tracks an effect's dependencies by what it *reads*. An effect that reads
//! a signal and also writes it subscribes to its own writes, so each run queues
//! the next and the pane re-renders forever. The Document Map's effect did that
//! from `321144d` until RFC-086: invisible on a small file, a whole CPU core on an
//! 8,000-item one, and every click waited behind the render in progress. The
//! visible symptom is no mutation, so a test of the output could never see it.
//!
//! This asserts the cause. Each view is mounted the way the shell mounts it
//! (`landmark_tests::host`: toolbar, map, the one Writing Area, status bar, search,
//! the shell's own effects), over Markdown and JSON, and run until it is quiet.
//! Then it must stay quiet: the `VirtualDom` has no dirty scope, no queued task and
//! no effect to run, however many times it is asked. A new self-triggering effect in
//! any of those components fails here.

use std::future::Future;
use std::task::{Context, Poll, Waker};

use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use omriss_ui::i18n::Locale;

use crate::landmark_tests::{CASE, VIEWS, View, host};

/// Whether the `VirtualDom` has work: asks it to wait for some, once. `wait_for_work`
/// runs the queued effects and tasks first, so an effect that re-queues itself shows
/// up as ready work, and a quiet `VirtualDom` is pending.
fn has_work(dom: &mut VirtualDom) -> bool {
    let mut waiting = std::pin::pin!(dom.wait_for_work());
    matches!(
        waiting
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(())
    )
}

/// Renders until quiet, at most `limit` rounds. Returns the rounds it took, or
/// `None` if it never went quiet.
fn settle(dom: &mut VirtualDom, limit: usize) -> Option<usize> {
    for round in 0..limit {
        if !has_work(dom) {
            return Some(round);
        }
        dom.render_immediate(&mut NoOpMutations);
    }
    (!has_work(dom)).then_some(limit)
}

/// What a legitimate start-up needs is a handful of rounds (an effect fills the
/// map, which renders, which settles). A loop never finishes.
const LIMIT: usize = 12;

fn mounted(view: View, locale: Locale) -> VirtualDom {
    CASE.with(|c| c.set((view, locale)));
    let mut dom = VirtualDom::new(host);
    dom.rebuild_in_place();
    dom
}

#[test]
fn every_view_settles_and_then_does_no_work() {
    for locale in [Locale::En, Locale::Ja] {
        for view in VIEWS {
            let mut dom = mounted(view, locale);
            let rounds = settle(&mut dom, LIMIT).unwrap_or_else(|| {
                panic!(
                    "{view:?}/{locale:?}: still has work after {LIMIT} rounds; an effect is \
                     re-triggering itself (RFC-086). Inside an effect, read a signal it also \
                     writes with `peek()`"
                )
            });
            assert!(
                rounds <= 6,
                "{view:?}/{locale:?}: {rounds} rounds to settle"
            );
            // And it stays settled: ask again, and render again, many times.
            for i in 0..20 {
                assert!(
                    !has_work(&mut dom),
                    "{view:?}/{locale:?}: work appeared on idle check {i}"
                );
                dom.render_immediate(&mut NoOpMutations);
            }
        }
    }
}

#[test]
fn the_settle_check_sees_a_self_triggering_effect() {
    // The check is only worth having if it fails for the fault it is for.
    fn looping() -> Element {
        let mut n = use_signal(|| 0u32);
        use_effect(move || {
            let current = *n.read(); // a read of what the effect writes next
            n.set(current + 1);
        });
        rsx! { div { "{n}" } }
    }
    let mut dom = VirtualDom::new(looping);
    dom.rebuild_in_place();
    assert_eq!(
        settle(&mut dom, LIMIT),
        None,
        "a read-then-write effect never settles"
    );

    fn fixed() -> Element {
        let mut n = use_signal(|| 0u32);
        use_effect(move || {
            let current = *n.peek();
            n.set(current + 1);
        });
        rsx! { div { "{n}" } }
    }
    let mut dom = VirtualDom::new(fixed);
    dom.rebuild_in_place();
    assert!(
        settle(&mut dom, LIMIT).is_some(),
        "the same effect with peek() settles"
    );
}
