//! Engine event stream — the channel the orchestrator talks to a UI on.
//!
//! The CLI does not need events (it prints a final report), so it
//! passes a [`NullSink`]. The TUI (M3) passes a [`ChannelSink`] and
//! renders frames as events arrive. Tests use [`RecordingSink`] to
//! assert the exact event sequence a run produces.
//!
//! Events are *observational*: emitting one never changes pipeline
//! behavior, and a sink that drops every event is always valid.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super_dev_spec::Phase;

use crate::gates::Gate;

/// One thing the engine did, surfaced to whatever UI is watching.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EngineEvent {
    /// The pipeline run has begun.
    PipelineStarted {
        /// Project slug.
        slug: String,
        /// The free-form requirement driving the run.
        requirement: String,
    },
    /// A phase has started executing.
    PhaseStarted {
        /// The phase that just began.
        phase: Phase,
    },
    /// A phase wrote an artifact to disk.
    ArtifactWritten {
        /// The phase that produced it.
        phase: Phase,
        /// Absolute path of the written file.
        path: PathBuf,
    },
    /// A phase finished.
    PhaseCompleted {
        /// The phase that finished.
        phase: Phase,
    },
    /// The pipeline paused at a confirmation gate awaiting user input.
    GateOpened {
        /// Which gate is now open.
        gate: Gate,
    },
    /// A block of execution finished — either paused at a gate, or the
    /// whole pipeline completed when `paused_at` is `None`.
    BlockCompleted {
        /// The phase the engine settled on.
        final_phase: Phase,
        /// The gate it paused at, or `None` when delivery completed.
        paused_at: Option<Gate>,
    },
    /// A host backend was probed for availability (TUI startup).
    BackendProbed {
        /// Stable backend id (`claude-code` / `codex`).
        backend_id: String,
        /// `true` when the host CLI is installed and reachable.
        ready: bool,
        /// Human-readable detail (version string or failure reason).
        detail: String,
    },
    /// Verify is starting after a code-producing phase. The runner
    /// inspected the workspace and is about to run the build command.
    VerifyStarted {
        /// The phase whose output is being verified.
        phase: Phase,
        /// Human-readable command string (e.g. `cargo check --quiet`).
        command: String,
    },
    /// Verify ran but the workspace had no recognised project manifest
    /// (no `package.json` / `Cargo.toml` / `pyproject.toml`).
    VerifySkipped {
        /// The phase that was being verified.
        phase: Phase,
        /// Why it was skipped (always "no recognised manifest" today).
        reason: String,
    },
    /// Verify completed successfully — the build / install command
    /// returned exit code 0.
    VerifyPassed {
        /// The phase that was verified.
        phase: Phase,
        /// Wall-clock duration of the command, milliseconds.
        duration_ms: u64,
    },
    /// Verify ran and the build / install command failed. The runner
    /// will fall back to the next attempt or surface the failure to
    /// the user (depending on policy).
    VerifyFailed {
        /// The phase that was verified.
        phase: Phase,
        /// Exit code from the build command.
        exit_code: i32,
        /// Truncated stderr (≤ 8 KiB) for the UI to show.
        stderr: String,
    },
    /// One chunk of output produced by the host CLI (Claude Code /
    /// Codex). Emitted per non-empty line so a UI can render the host's
    /// response as it scrolls past, instead of dumping the whole thing
    /// at the end of the phase.
    HostOutput {
        /// The phase that produced this chunk.
        phase: Phase,
        /// One line of host stdout (already ANSI-stripped).
        line: String,
    },
    /// A human-readable progress note (free-form).
    Note(String),
}

/// Anything that consumes [`EngineEvent`]s. Implementations must be
/// cheap to call and never block the engine.
pub trait EventSink: Send + Sync {
    /// Receive one event. Must not panic.
    fn emit(&self, event: EngineEvent);
}

/// Drops every event. The default for headless / CLI runs.
#[derive(Debug, Default, Clone, Copy)]
pub struct NullSink;

impl EventSink for NullSink {
    fn emit(&self, _event: EngineEvent) {}
}

/// Forwards events into an async channel — the TUI's input.
#[derive(Debug, Clone)]
pub struct ChannelSink {
    tx: tokio::sync::mpsc::UnboundedSender<EngineEvent>,
}

impl ChannelSink {
    /// Build a sink plus the receiver the UI loop should poll.
    #[must_use]
    pub fn new() -> (Self, tokio::sync::mpsc::UnboundedReceiver<EngineEvent>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (Self { tx }, rx)
    }
}

impl EventSink for ChannelSink {
    fn emit(&self, event: EngineEvent) {
        // A closed receiver just means the UI went away; drop silently.
        let _ = self.tx.send(event);
    }
}

/// Captures every event in memory — for tests.
#[derive(Debug, Default, Clone)]
pub struct RecordingSink {
    events: Arc<Mutex<Vec<EngineEvent>>>,
}

impl RecordingSink {
    /// Build an empty recording sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot every event captured so far.
    #[must_use]
    pub fn events(&self) -> Vec<EngineEvent> {
        self.events
            .lock()
            .expect("recording sink mutex poisoned")
            .clone()
    }

    /// Count events matching a predicate.
    #[must_use]
    pub fn count(&self, pred: impl Fn(&EngineEvent) -> bool) -> usize {
        self.events().iter().filter(|e| pred(e)).count()
    }
}

impl EventSink for RecordingSink {
    fn emit(&self, event: EngineEvent) {
        self.events
            .lock()
            .expect("recording sink mutex poisoned")
            .push(event);
    }
}

/// Convenience: a no-op sink behind an `Arc` for the default runner.
#[must_use]
pub fn null_sink() -> Arc<dyn EventSink> {
    Arc::new(NullSink)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_sink_drops_everything() {
        let sink = NullSink;
        sink.emit(EngineEvent::Note("ignored".into()));
        // nothing to assert — just must not panic
    }

    #[test]
    fn recording_sink_captures_in_order() {
        let sink = RecordingSink::new();
        sink.emit(EngineEvent::PhaseStarted {
            phase: Phase::Research,
        });
        sink.emit(EngineEvent::PhaseCompleted {
            phase: Phase::Research,
        });
        let events = sink.events();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], EngineEvent::PhaseStarted { .. }));
        assert!(matches!(events[1], EngineEvent::PhaseCompleted { .. }));
    }

    #[test]
    fn recording_sink_count_filters() {
        let sink = RecordingSink::new();
        sink.emit(EngineEvent::ArtifactWritten {
            phase: Phase::Docs,
            path: "a.md".into(),
        });
        sink.emit(EngineEvent::ArtifactWritten {
            phase: Phase::Docs,
            path: "b.md".into(),
        });
        sink.emit(EngineEvent::Note("x".into()));
        assert_eq!(
            sink.count(|e| matches!(e, EngineEvent::ArtifactWritten { .. })),
            2
        );
    }

    #[tokio::test]
    async fn channel_sink_delivers_to_receiver() {
        let (sink, mut rx) = ChannelSink::new();
        sink.emit(EngineEvent::Note("hello".into()));
        let got = rx.recv().await.unwrap();
        assert_eq!(got, EngineEvent::Note("hello".into()));
    }

    #[tokio::test]
    async fn channel_sink_tolerates_dropped_receiver() {
        let (sink, rx) = ChannelSink::new();
        drop(rx);
        // Must not panic even though nobody is listening.
        sink.emit(EngineEvent::Note("nobody home".into()));
    }
}
