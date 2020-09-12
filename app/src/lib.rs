use draco::html as h;
use draco::{Application, Mailbox, VNode};
use indexmap::IndexMap;
use std::mem;
use ulid::Ulid;
use wasm_bindgen::prelude::*;

type TodoId = Ulid;

#[derive(Debug)]
struct Model {
    input: String,
    entries: IndexMap<TodoId, Entry>,
}

#[derive(Debug)]
struct Entry {
    id: TodoId,
    description: String,
    completed: bool,
}

enum Message {
    UpdateField(String),
    Add,
    Check(TodoId, bool),
    Delete(TodoId),
}

impl Application for Model {
    type Message = Message;

    fn update(&mut self, message: Self::Message, _: &Mailbox<Self::Message>) {
        match message {
            Message::UpdateField(input) => {
                self.input = input;
            }
            Message::Add => {
                let description = mem::take(&mut self.input);
                let id = TodoId::new();
                self.entries.insert(
                    id,
                    Entry {
                        id,
                        description,
                        completed: false,
                    },
                );
            }
            Message::Check(id, completed) => {
                self.entries
                    .get_mut(&id)
                    .map(|todo| todo.completed = completed);
            }
            Message::Delete(id) => {
                self.entries.remove(&id);
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

        let view_entry = |entry: &Entry| {
            let Entry {
                id,
                completed,
                ref description,
                ..
            } = *entry;

            h::li().with(
                h::div().class("view").with((
                    h::input()
                        .class("toggle")
                        .type_("checkbox")
                        .checked(completed)
                        .on_("click", move |event| {
                            let checked = js_sys::Reflect::get(
                                &&event.target()?,
                                &JsValue::from_str("checked"),
                            )
                            .ok()?
                            .as_bool()?;
                            Some(Message::Check(id, checked))
                        }),
                    h::label().with(description.clone()),
                    h::button()
                        .class("destroy")
                        .on("click", move |_| Message::Delete(id))
                        .with("Delete"),
                )),
            )
        };

        let entries = h::section().class("main").with(
            h::ul()
                .class("todo-list")
                .append(self.entries.values().map(view_entry)),
        );

        h::div()
            .class("todoapp")
            .with(
                h::textarea()
                    .disabled(true)
                    .rows(12)
                    .cols(80)
                    .with(format!("{:#?}", self)),
            )
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
        entries: IndexMap::new(),
    };

    let _mailbox = draco::start(todomvc, node.into());

    Ok(())
}
