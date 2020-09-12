use draco::html as h;
use draco::{Application, Mailbox, VNode};
use std::mem;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
struct Model {
    input: String,
    entries: Vec<Entry>,
}

#[derive(Debug)]
struct Entry {
    description: String,
}

enum Message {
    UpdateField(String),
    Add,
}

impl Application for Model {
    type Message = Message;

    fn update(&mut self, message: Self::Message, _: &Mailbox<Self::Message>) {
        match message {
            Message::UpdateField(input) => {
                self.input = input;
            }
            Message::Add => {
                let title = mem::take(&mut self.input);
                self.entries.push(Entry { description: title });
            }
        }

        // FIXME: save model to localStorage
    }

    fn view(&self) -> VNode<Self::Message> {
        let input = h::header().class("header").with((
            h::h1().with("todos"),
            h::input()
                .name("new_todo")
                .placeholder("What needs to be done?")
                .autofocus(true)
                .value(self.input.clone())
                .on_input(Message::UpdateField),
            // FIXME: remove this and add on_enter event to above input element.
            h::button().on("click", |_| Message::Add).with("Add"),
        ));

        let entries = h::section()
            .class("main")
            .with(
                h::ul()
                    .class("todo-list")
                    .append(self.entries.iter().map(|todo| {
                        h::li().with(
                            h::div()
                                .class("view")
                                .with(h::label().with(todo.description.clone())),
                        )
                    })),
            );

        h::div() //
            .class("todoapp")
            .with((input, entries))
            .into()
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no global `window` exists")?;
    let document = window
        .document()
        .ok_or("should have a document on window")?;

    let app = document
        .get_element_by_id("app")
        .ok_or("missing `app` in document")?;

    let node = document.create_element("div")?;
    app.append_child(&node)?;

    let todomvc = Model {
        input: "".into(),
        entries: vec![],
    };

    let _mailbox = draco::start(todomvc, node.into());

    Ok(())
}
