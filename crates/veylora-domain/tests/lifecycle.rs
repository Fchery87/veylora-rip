use veylora_domain::lifecycle::{transition, JobState::*, LifecycleEvent::*};
use veylora_domain::lifecycle::{JobState, LifecycleEvent};

const STATES: [JobState; 12] = [
    Draft,
    PreflightFailed,
    Held,
    Rendering,
    Ready,
    Submitting,
    Submitted,
    Ambiguous,
    Review,
    Cancelled,
    Failed,
    OperatorConfirmedFinished,
];
const EVENTS: [LifecycleEvent; 15] = [
    PreflightRejected,
    PreflightPassed,
    BeginRender,
    RenderCompleted,
    RenderFailed,
    Hold,
    Release,
    BeginSubmission,
    SubmissionAccepted,
    SubmissionUncertain,
    SubmissionRejectedBeforeWrite,
    RequestReview,
    OperatorConfirmFinished,
    OperatorCancel,
    Cancel,
];

/// Every legal move, written out independently of the implementation. Anything not listed must be
/// refused, which is how "no retry edge after submission" is enforced.
const LEGAL: &[(JobState, LifecycleEvent, JobState)] = &[
    (Draft, PreflightRejected, PreflightFailed),
    (PreflightFailed, PreflightRejected, PreflightFailed),
    (Draft, PreflightPassed, Held),
    (PreflightFailed, PreflightPassed, Held),
    (Held, BeginRender, Rendering),
    (Held, Release, Rendering),
    (Rendering, RenderCompleted, Ready),
    (Rendering, RenderFailed, Failed),
    (Ready, Hold, Held),
    (Ready, BeginSubmission, Submitting),
    (Submitting, SubmissionAccepted, Submitted),
    (Submitting, SubmissionUncertain, Ambiguous),
    (Submitting, Cancel, Ambiguous),
    (Submitting, SubmissionRejectedBeforeWrite, Failed),
    (Ambiguous, RequestReview, Review),
    (Submitted, RequestReview, Review),
    (
        Submitted,
        OperatorConfirmFinished,
        OperatorConfirmedFinished,
    ),
    (Review, OperatorConfirmFinished, OperatorConfirmedFinished),
    (Review, OperatorCancel, Cancelled),
    (Draft, Cancel, Cancelled),
    (PreflightFailed, Cancel, Cancelled),
    (Held, Cancel, Cancelled),
    (Rendering, Cancel, Cancelled),
    (Ready, Cancel, Cancelled),
    // An operator may release a failed job to try again. History is kept; this is not a retry of
    // an ambiguous submission.
    (Failed, Release, Held),
];

#[test]
fn exactly_the_listed_transitions_are_legal() {
    for &state in &STATES {
        for &event in &EVENTS {
            let expected = LEGAL
                .iter()
                .find(|(s, e, _)| *s == state && *e == event)
                .map(|(_, _, to)| *to);
            assert_eq!(
                transition(state, event).ok(),
                expected,
                "{state:?} + {event:?}"
            );
        }
    }
}

#[test]
fn no_event_leaves_a_terminal_state_or_retries_an_uncertain_submission() {
    for &event in &EVENTS {
        assert!(transition(Cancelled, event).is_err(), "{event:?}");
        assert!(
            transition(OperatorConfirmedFinished, event).is_err(),
            "{event:?}"
        );
        // Neither an ambiguous nor a reviewed submission can be re-submitted or re-rendered.
        for state in [Ambiguous, Review, Submitted] {
            assert!(
                !matches!(
                    transition(state, event),
                    Ok(Held | Rendering | Ready | Submitting)
                ),
                "{state:?} + {event:?}"
            );
        }
    }
}
