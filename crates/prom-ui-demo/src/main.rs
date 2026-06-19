extern crate alloc;

use alloc::vec;
use alloc::sync::Arc;
use prom_ui::paint::{UiColor, UiDrawCommand};
use prom_ui::layout::{UiRect, UiPoint, UiSize, UiHitTest};
use prom_ui_backend_native::draw_backend::UiDrawBackend;
use prom_ui_backend_native::wgpu_backend::WgpuDrawBackend;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use prom_ui::interaction::{ElementId, InteractionModifiers, InteractionIntentId};
use prom_ui::action_admission::{ActionAdmissionGuard, ActionAdmissionResult, AdmittedAction};
use prom_ui::action_request::{ActionRequestDescriptor, ActionKind, ActionRequestId};
use prom_ui::action_dispatch_route::ActionRuntimeDispatcher;
use prom_ui::raw_event::{RawUiEvent, RawUiEventId, RawUiEventKind, RawUiEventTarget, RawUiEventPayload, RawPointerButton, map_raw_event_to_interaction_intent};

struct DummyHitTest;
impl UiHitTest for DummyHitTest {
    fn hit_test(&self, _x: f64, _y: f64) -> Option<ElementId> {
        Some(ElementId(42))
    }
}

struct DummyAdmissionGuard;
impl ActionAdmissionGuard for DummyAdmissionGuard {
    fn admit_request(&self, request: ActionRequestDescriptor) -> ActionAdmissionResult {
        ActionAdmissionResult::Admitted(AdmittedAction { request_id: request.id })
    }
}

struct DummyDispatcher;
impl ActionRuntimeDispatcher for DummyDispatcher {
    fn dispatch_action(&self, action: AdmittedAction) {
        println!("Action executed! Request ID: {}", action.request_id.0);
    }
}

struct DemoApp {
    window: Option<Arc<Window>>,
    backend: Option<WgpuDrawBackend>,
    event_counter: u64,
}

impl ApplicationHandler for DemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title("Semantic UI Static Demo")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));
            if let Ok(w) = event_loop.create_window(attrs) {
                let arc_w = Arc::new(w);
                self.window = Some(arc_w.clone());

                let mut backend = WgpuDrawBackend::initialize(arc_w);

                // Initial draw
                let commands = vec![
                    UiDrawCommand::FillRect {
                        rect: UiRect { origin: UiPoint { x: 0, y: 0 }, size: UiSize { width: 800, height: 600 } },
                        color: UiColor { r: 255, g: 0, b: 0, a: 255 }, // Solid Red
                    }
                ];
                backend.sink().submit_frame(commands);

                self.backend = Some(backend);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Some(window) = &self.window {
            if window.id() == id {
                match event {
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        if let Some(backend) = &mut self.backend {
                            let commands = vec![
                                UiDrawCommand::FillRect {
                                    rect: UiRect { origin: UiPoint { x: 0, y: 0 }, size: UiSize { width: 800, height: 600 } },
                                    color: UiColor { r: 255, g: 0, b: 0, a: 255 },
                                }
                            ];
                            backend.sink().submit_frame(commands);
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        self.event_counter += 1;
                        let hit_test = DummyHitTest;
                        let guard = DummyAdmissionGuard;
                        let dispatcher = DummyDispatcher;

                        let raw_event = RawUiEvent::new(
                            RawUiEventId(self.event_counter),
                            RawUiEventKind::PointerMove,
                            RawUiEventTarget::Unknown,
                            InteractionModifiers::default(),
                            RawUiEventPayload::CursorMove { x: position.x, y: position.y },
                        );

                        let target = hit_test.hit_test(position.x, position.y);
                        let intent = map_raw_event_to_interaction_intent(&raw_event);

                        let request = ActionRequestDescriptor::new(
                            ActionRequestId(self.event_counter),
                            ActionKind::GenericSemanticAction,
                            target,
                            InteractionIntentId(self.event_counter),
                        );

                        if let ActionAdmissionResult::Admitted(action) = guard.admit_request(request) {
                            print!("Hover! ");
                            dispatcher.dispatch_action(action);
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        self.event_counter += 1;
                        let hit_test = DummyHitTest;
                        let guard = DummyAdmissionGuard;
                        let dispatcher = DummyDispatcher;

                        let kind = match state {
                            winit::event::ElementState::Pressed => RawUiEventKind::PointerDown,
                            winit::event::ElementState::Released => RawUiEventKind::PointerUp,
                        };
                        let b = match button {
                            winit::event::MouseButton::Left => RawPointerButton::Primary,
                            winit::event::MouseButton::Right => RawPointerButton::Secondary,
                            winit::event::MouseButton::Middle => RawPointerButton::Middle,
                            _ => RawPointerButton::Unknown,
                        };

                        let raw_event = RawUiEvent::new(
                            RawUiEventId(self.event_counter),
                            kind,
                            RawUiEventTarget::Unknown,
                            InteractionModifiers::default(),
                            RawUiEventPayload::Pointer { button: b },
                        );

                        let target = hit_test.hit_test(0.0, 0.0);
                        let intent = map_raw_event_to_interaction_intent(&raw_event);

                        let request = ActionRequestDescriptor::new(
                            ActionRequestId(self.event_counter),
                            ActionKind::GenericSemanticAction,
                            target,
                            InteractionIntentId(self.event_counter),
                        );

                        if let ActionAdmissionResult::Admitted(action) = guard.admit_request(request) {
                            print!("{:?} intent -> ", intent.kind);
                            dispatcher.dispatch_action(action);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = DemoApp {
        window: None,
        backend: None,
        event_counter: 0,
    };
    event_loop.run_app(&mut app).unwrap();
}
