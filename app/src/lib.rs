use draco::html as h;
use draco::{Application, Mailbox, VNode};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::mem;
use ulid::Ulid;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast as _;

type TodoId = Ulid;

const STORAGE_KEY: &str = "draco-todomvc-save";

#[derive(Debug, Default, Deserialize, Serialize)]
struct Model {
    #[serde(skip)]
    input: String,
    entries: IndexMap<TodoId, Entry>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Entry {
    id: TodoId,
    description: String,
    completed: bool,
    editing: bool,
    #[serde(skip)]
    ref_: Option<web_sys::Element>,
}

enum Message {
    UpdateField(String),
    Add,
    Check(TodoId, bool),
    Delete(TodoId),
    UpdateEntry(TodoId, String),
    EditingEntry(TodoId, bool),
    RefEntry(TodoId, Option<web_sys::Element>),
}

#[derive(Debug)]
struct TodoMVC {
    model: Model,
    storage: web_sys::Storage,
}

impl TodoMVC {
    fn set_storage(&self) {
        let encoded = serde_json::to_string(&self.model).expect("failed to encode JSON");
        self.storage.set_item(STORAGE_KEY, &encoded).unwrap();
    }
}

impl Application for TodoMVC {
    type Message = Message;

    fn update(&mut self, message: Self::Message, _: &Mailbox<Self::Message>) {
        let Self {
            model:
                Model {
                    ref mut input,
                    ref mut entries,
                    ..
                },
            ..
        } = *self;

        match message {
            Message::UpdateField(new_input) => {
                *input = new_input;
            }
            Message::Add => {
                let description = mem::take(input).trim().to_owned();
                if !description.is_empty() {
                    let id = TodoId::new();
                    entries.insert(
                        id,
                        Entry {
                            id,
                            description,
                            completed: false,
                            editing: false,
                            ref_: None,
                        },
                    );
                }
                self.set_storage();
            }
            Message::Check(id, completed) => {
                if let Some(entry) = entries.get_mut(&id) {
                    entry.completed = completed;
                    self.set_storage();
                }
            }
            Message::Delete(id) => {
                entries.remove(&id);
                self.set_storage();
            }
            Message::UpdateEntry(id, description) => {
                if let Some(entry) = entries.get_mut(&id) {
                    entry.description = description;
                }
            }
            Message::EditingEntry(id, editing) => {
                if let Some(entry) = entries.get_mut(&id) {
                    entry.editing = editing;
                    if let Some(ref e) = entry.ref_ {
                        if let Some(e) = e.dyn_ref::<web_sys::HtmlElement>() {
                            let _ = e.focus();
                        }
                    }
                    self.set_storage();
                }
            }
            Message::RefEntry(id, ref_) => {
                if let Some(entry) = entries.get_mut(&id) {
                    entry.ref_ = ref_;
                }
            }
        }
    }

    fn view(&self) -> VNode<Self::Message> {
        let Self {
            model: Model { input, entries, .. },
            ..
        } = self;

        let input = h::header().class("header").with((
            h::h1().with("todos"),
            h::input()
                .name("new_todo")
                .placeholder("What needs to be done?")
                .autofocus(true)
                .value(input.clone())
                .on_input(Message::UpdateField)
                .on_enter(|| Message::Add),
        ));

        let view_entry = |entry: &Entry| {
            let Entry {
                id,
                completed,
                ref description,
                ..
            } = *entry;

            h::li().with((
                h::div().class("view").with((
                    h::input()
                        .class("toggle")
                        .type_("checkbox")
                        .checked(completed)
                        .on_check(move |checked| Message::Check(id, checked)),
                    h::label()
                        .on("dblclick", move |_| Message::EditingEntry(id, true))
                        .with(description.clone()),
                    h::button()
                        .class("destroy")
                        .on("click", move |_| Message::Delete(id))
                        .with("Delete"),
                )),
                h::input()
                    .class("edit")
                    .value(description.clone())
                    .name("title")
                    .id(format!("todo-{}", id))
                    .on_input(move |input| Message::UpdateEntry(id, input))
                    .on("blur", move |_| Message::EditingEntry(id, false))
                    .on_enter(move || Message::EditingEntry(id, false))
                    .ref_(move |e| Message::RefEntry(id, e)),
            ))
        };

        let entries = h::section().class("main").with(
            h::ul()
                .class("todo-list")
                .append(entries.values().map(view_entry)),
        );

        h::div().class("todoapp").with((input, entries)).into()
    }
}

trait Ext<Msg> {
    fn on_check(self, f: impl Fn(bool) -> Msg + 'static) -> Self
    where
        Self: Sized;

    fn on_enter(self, f: impl Fn() -> Msg + 'static) -> Self
    where
        Self: Sized;
}

impl<Msg> Ext<Msg> for draco::VNonKeyedElement<Msg> {
    fn on_check(self, f: impl Fn(bool) -> Msg + 'static) -> Self {
        self.on_("click", move |event| {
            let checked = js_sys::Reflect::get(&&event.target()?, &JsValue::from_str("checked"))
                .ok()?
                .as_bool()?;
            Some(f(checked))
        })
    }

    fn on_enter(self, f: impl Fn() -> Msg + 'static) -> Self {
        self.on_("keydown", move |event| {
            let key = js_sys::Reflect::get(&event, &JsValue::from_str("key"))
                .ok()?
                .as_string()?;
            match &*key {
                "Enter" => Some(f()),
                _ => None,
            }
        })
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

    let storage = window
        .local_storage()?
        .ok_or("cannot access localStorage")?;

    let model_raw = storage.get_item(STORAGE_KEY).ok().flatten();
    let model = model_raw
        .and_then(|val| serde_json::from_str(&val).ok())
        .unwrap_or_default();

    let _mailbox = draco::start(TodoMVC { model, storage }, node.into());

    Ok(())
}
