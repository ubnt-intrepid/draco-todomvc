use draco::html as h;
use draco::{Application, Mailbox, VNode};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::mem;
use ulid::Ulid;
use wasm_bindgen::prelude::*;
use web_sys as web;

type TodoId = Ulid;

const STORAGE_KEY: &str = "draco-todomvc-save";

#[derive(Debug, Default, Deserialize, Serialize)]
struct Model {
    #[serde(skip)]
    input: String,
    entries: IndexMap<TodoId, Entry>,
    visibility: Option<Visibility>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Entry {
    id: TodoId,
    description: String,
    completed: bool,
    #[serde(skip)]
    editing: bool,
}

enum Message {
    Nop,
    UpdateField(String),
    Add,
    Check(TodoId, bool),
    CheckAll(bool),
    Delete(TodoId),
    DeleteCompleted,
    UpdateEntry(TodoId, String),
    EditingEntry(TodoId, bool),
    FocusEntry(web::Element),
    ChangeVisibility(Visibility),
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
enum Visibility {
    All,
    Active,
    Completed,
}

#[derive(Debug)]
struct TodoMVC {
    model: Model,
    storage: web::Storage,
}

impl TodoMVC {
    fn set_storage(&self) {
        let encoded = serde_json::to_string(&self.model).unwrap_throw();
        self.storage.set_item(STORAGE_KEY, &encoded).unwrap_throw();
    }
}

impl Application for TodoMVC {
    type Message = Message;

    fn update(&mut self, message: Self::Message, _: &Mailbox<Self::Message>) {
        let Self {
            model: ref mut current,
            ..
        } = *self;

        match message {
            Message::Nop => (),
            Message::UpdateField(input) => {
                current.input = input;
            }
            Message::Add => {
                let description = mem::take(&mut current.input).trim().to_owned();
                if !description.is_empty() {
                    let id = TodoId::new();
                    current.entries.insert(
                        id,
                        Entry {
                            id,
                            description,
                            completed: false,
                            editing: false,
                        },
                    );
                }
                self.set_storage();
            }
            Message::Check(id, completed) => {
                if let Some(entry) = current.entries.get_mut(&id) {
                    entry.completed = completed;
                    self.set_storage();
                }
            }
            Message::CheckAll(completed) => {
                for entry in current.entries.values_mut() {
                    entry.completed = completed;
                }
                self.set_storage();
            }
            Message::Delete(id) => {
                if let Some(..) = current.entries.remove(&id) {
                    self.set_storage();
                }
            }
            Message::DeleteCompleted => {
                current.entries.retain(|_, entry| !entry.completed);
                self.set_storage();
            }
            Message::UpdateEntry(id, description) => {
                if let Some(entry) = current.entries.get_mut(&id) {
                    entry.description = description;
                }
            }
            Message::EditingEntry(id, editing) => {
                if let Some(entry) = current.entries.get_mut(&id) {
                    entry.editing = editing;
                }
            }
            Message::FocusEntry(e) => {
                let e: web::HtmlElement = JsValue::from(e).into();
                e.focus().unwrap_throw();
            }
            Message::ChangeVisibility(visibility) => {
                current.visibility.replace(visibility);
                self.set_storage();
            }
        }
    }

    fn view(&self) -> VNode<Self::Message> {
        let Self { model: m, .. } = self;

        let has_completed = m.entries.values().any(|entry| entry.completed);
        let all_completed = m.entries.values().all(|entry| entry.completed);
        let entries_left = m.entries.values().filter(|e| !e.completed).count();

        let input = h::header() //
            .class("header")
            .with((
                h::h1().with("todos"),
                h::input()
                    .class("new-todo")
                    .placeholder("What needs to be done?")
                    .autofocus(true)
                    .name("new_todo")
                    .value(m.input.clone())
                    .on_input(Message::UpdateField)
                    .on_enter(|| Message::Add),
            ));

        fn view_entry(entry: &Entry) -> draco::VNonKeyedElement<Message> {
            let Entry {
                id,
                completed,
                editing,
                ref description,
                ..
            } = *entry;

            h::li()
                .if_(completed, |h| h.class("completed"))
                .with(
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
                            .on("click", move |_| Message::Delete(id)),
                    )),
                )
                .if_(editing, |h| {
                    h.class("editing").with(
                        h::input()
                            .class("edit")
                            .value(description.clone())
                            .name("title")
                            .ref_(move |elem| elem.map_or(Message::Nop, Message::FocusEntry))
                            .on_input(move |input| Message::UpdateEntry(id, input))
                            .on("blur", move |_| Message::EditingEntry(id, false))
                            .on_enter(move || Message::EditingEntry(id, false)),
                    )
                })
        }

        let entries = h::section().class("main").with((
            h::input()
                .class("toggle-all")
                .type_("checkbox")
                .id("toggle-all")
                .checked(all_completed)
                .on("click", move |_| Message::CheckAll(!all_completed)),
            h::label().for_("toggle-all").with("Mark all as complete"),
            h::ul().class("todo-list").append(
                m.entries
                    .values()
                    .filter(|entry| match m.visibility {
                        Some(Visibility::Active) => !entry.completed,
                        Some(Visibility::Completed) => entry.completed,
                        _ => true,
                    })
                    .map(view_entry),
            ),
        ));

        let controls = if !m.entries.is_empty() {
            let visibility_swap = |v: Visibility, url: &'static str, text: &'static str| {
                h::li()
                    .on("click", move |_| Message::ChangeVisibility(v))
                    .with(
                        h::a()
                            .href(url)
                            .if_(m.visibility.map_or(false, |vis| vis == v), |h| {
                                h.class("selected")
                            })
                            .with(text),
                    )
            };
            Some(
                h::footer()
                    .class("footer")
                    .with((
                        h::span().class("todo-count").with((
                            h::strong().with(entries_left),
                            " item",
                            plural_suffix(entries_left),
                            " left",
                        )),
                        h::ul().class("filters").with((
                            visibility_swap(Visibility::All, "#/", "All"),
                            visibility_swap(Visibility::Active, "#/active", "Active"),
                            visibility_swap(Visibility::Completed, "#/completed", "Completed"),
                        )),
                    ))
                    .if_(has_completed, |h| {
                        h.with(
                            h::button()
                                .class("clear-completed")
                                .on("click", |_| Message::DeleteCompleted)
                                .with("Clear completed"),
                        )
                    }),
            )
        } else {
            None
        };

        let info_footer = h::footer().class("info").with((
            h::p().with("Double-click to edit a todo"),
            h::p().with((
                "Written by ",
                h::a()
                    .href("https://github.com/ubnt-intrepid/")
                    .with("@ubnt-intrepid"),
            )),
            h::p().with((
                "Part of ",
                h::a().href("http://todomvc.com").with("TodoMVC"),
            )),
        ));

        h::div()
            .class("todomvc-wrapper")
            .visibility("hidden")
            .with((
                h::section()
                    .class("todoapp")
                    .with((input, entries))
                    .if_some(controls, |h, c| h.with(c)),
                info_footer,
            ))
            .into()
    }
}

trait BuilderExt {
    fn if_(self, pred: bool, f: impl FnOnce(Self) -> Self) -> Self
    where
        Self: Sized,
    {
        if pred {
            f(self)
        } else {
            self
        }
    }

    fn if_some<T>(self, v: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self
    where
        Self: Sized,
    {
        if let Some(v) = v {
            f(self, v)
        } else {
            self
        }
    }
}

impl<T> BuilderExt for T {}

trait VElementExt<Msg> {
    fn on_check(self, f: impl Fn(bool) -> Msg + 'static) -> Self
    where
        Self: Sized;

    fn on_enter(self, f: impl Fn() -> Msg + 'static) -> Self
    where
        Self: Sized;
}

impl<Msg> VElementExt<Msg> for draco::VNonKeyedElement<Msg> {
    fn on_check(self, f: impl Fn(bool) -> Msg + 'static) -> Self {
        self.on_("click", move |event| {
            #[allow(unused_unsafe)]
            let checked = unsafe {
                js_sys::Reflect::get(
                    &&event.target()?, //
                    &JsValue::from_str("checked"),
                )
            }
            .ok()?
            .as_bool()?;
            Some(f(checked))
        })
    }

    fn on_enter(self, f: impl Fn() -> Msg + 'static) -> Self {
        self.on_("keydown", move |event| {
            #[allow(unused_unsafe)]
            let key = unsafe {
                js_sys::Reflect::get(
                    &event, //
                    &JsValue::from_str("key"),
                )
            }
            .ok()?
            .as_string()?;
            match &*key {
                "Enter" => Some(f()),
                _ => None,
            }
        })
    }
}

fn plural_suffix(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
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
