mod common_components;
mod common_events;
mod event_handling;
mod input_types;
mod text_node;

pub mod events {
    /*!
    For listening to HTML events.

    ```
    # use async_ui_web_components::components::{Button, Text};
    # use async_ui_web_core::combinators::join;
    # let _ = async {
    use async_ui_web_components::events::EmitElementEvent;
    let button = Button::new();
    let text = Text::new();
    let mut count = 0;
    join((
        button.render(text.render()),
        async {
            loop {
                text.set_data(&format!("count = {count}"));
                button.until_click().await; // 👈 wait for event!!!
                count += 1;
            }
        }
    )).await;
    # };
    ```
    */

    pub use super::common_events::{EmitEditEvent, EmitElementEvent};
    pub use super::event_handling::{EmitEvent, EventFutureStream};
}
pub mod components {
    /*!
    For creating HTML elements.

    ```rust
    # use async_ui_web_components::components::Input;
    # let _ = async {
    let my_input = Input::new();
    my_input.render().await;
    # };
    ```

    Most types here are named after the HTML tag they represent, for example
    [Input] corresponds to HTML `<input>`. Their are some exceptions such as
    [Anchor] corresponding to `<a>` and [Bold] corresponding to `<b>`.
    */
    pub use super::common_components::*;
    pub use super::text_node::Text;
}
