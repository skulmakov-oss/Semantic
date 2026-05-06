use prom_ui_runtime::{
    DesktopSession, EventBuffer, InMemoryBackend, InputEvent, InputEventKind, LoopControl,
    WindowConfig,
};

#[test]
fn in_memory_backend_starts_with_empty_event_queue() {
    let backend = InMemoryBackend::new();

    assert_eq!(backend.queued_event_count(), 0);
    assert!(backend.queued_events().is_empty());
}

#[test]
fn in_memory_backend_push_event_preserves_payload() {
    let mut backend = InMemoryBackend::new();

    backend.push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));
    backend.push_event(InputEvent::new(InputEventKind::KeyUp { key_code: 65 }));

    assert_eq!(backend.queued_event_count(), 2);
    assert_eq!(
        backend.queued_events()[0].kind,
        InputEventKind::KeyDown { key_code: 65 }
    );
    assert_eq!(
        backend.queued_events()[1].kind,
        InputEventKind::KeyUp { key_code: 65 }
    );
}

#[test]
fn in_memory_backend_extend_events_preserves_order() {
    let mut backend = InMemoryBackend::new();

    backend.extend_events([
        InputEvent::new(InputEventKind::KeyDown { key_code: 1 }),
        InputEvent::new(InputEventKind::KeyUp { key_code: 1 }),
        InputEvent::new(InputEventKind::CloseRequested),
    ]);

    let events = backend.queued_events();

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].kind, InputEventKind::KeyDown { key_code: 1 });
    assert_eq!(events[1].kind, InputEventKind::KeyUp { key_code: 1 });
    assert_eq!(events[2].kind, InputEventKind::CloseRequested);
}

#[test]
fn in_memory_backend_drain_events_returns_and_clears_queue() {
    let mut backend = InMemoryBackend::new();

    backend.push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 42 }));
    backend.push_event(InputEvent::new(InputEventKind::CloseRequested));

    let drained = backend.drain_events();

    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].kind, InputEventKind::KeyDown { key_code: 42 });
    assert_eq!(drained[1].kind, InputEventKind::CloseRequested);

    assert_eq!(backend.queued_event_count(), 0);
    assert!(backend.queued_events().is_empty());
}

#[test]
fn desktop_session_backend_mut_allows_in_memory_event_injection() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("InjectedEvents", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 77 }));

    assert_eq!(session.backend().queued_event_count(), 1);
    assert_eq!(
        session.backend().queued_events()[0].kind,
        InputEventKind::KeyDown { key_code: 77 }
    );
}

#[test]
fn injected_events_can_be_moved_into_event_buffer_and_polled() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("PollInjectedEvents", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.backend_mut().extend_events([
        InputEvent::new(InputEventKind::KeyDown { key_code: 11 }),
        InputEvent::new(InputEventKind::KeyUp { key_code: 11 }),
    ]);

    session.run(|_| LoopControl::ExitRequested).unwrap();

    let mut buffer = EventBuffer::new();
    for event in session.backend_mut().drain_events() {
        buffer.push(event);
    }

    let events = session.poll_events(&mut buffer).unwrap();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind, InputEventKind::KeyDown { key_code: 11 });
    assert_eq!(events[1].kind, InputEventKind::KeyUp { key_code: 11 });
    assert_eq!(session.backend().queued_event_count(), 0);
}

#[test]
fn failed_poll_does_not_clear_in_memory_backend_queue() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("FailedPollInjected", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::CloseRequested));

    let mut buffer = EventBuffer::new();

    let err = session
        .poll_events(&mut buffer)
        .expect_err("poll before run must fail");

    assert!(matches!(
        err,
        prom_ui_runtime::UiRuntimeError::LifecycleViolation { .. }
    ));

    assert_eq!(session.backend().queued_event_count(), 1);
    assert_eq!(
        session.backend().queued_events()[0].kind,
        InputEventKind::CloseRequested
    );
}
