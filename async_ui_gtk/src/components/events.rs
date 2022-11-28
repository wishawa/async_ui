pub(super) enum QueuedEvent {
    Click,
    // MouseDown,
    // MouseUp,
    Input,
    Submit,
    // KeyPress,
    // KeyUp,
    // KeyDown,
    Focus,
    Blur,
}

pub(super) type EventsManager = async_ui_props::events::EventsManager<QueuedEvent>;
