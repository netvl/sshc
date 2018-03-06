use std::rc::Rc;
use std::cell::{Cell, RefCell};

use cursive::Cursive;
use cursive::event::{Key, EventResult};
use cursive::traits::*;
use cursive::views::{SelectView, OnEventView, Dialog, LinearLayout, TextView, DummyView};
use either::Either;
use itertools::Itertools;

use config::{Config, ConfigItem, ConfigDefinition, ConfigGroup};
use execution::Execution;

struct State {
    config: Config,
    path: RefCell<Vec<String>>,
    execute: Cell<bool>,
}

impl State {
    fn current_item(&self) -> Either<&ConfigDefinition, &ConfigGroup> {
        let mut current_group = &self.config.root;
        for path_item in self.path.borrow().iter() {
            match current_group.definitions[path_item] {
                ConfigItem::Subgroup(ref group) => current_group = group,
                ConfigItem::Definition(ref definition) => return Either::Left(definition),
            }
        }
        Either::Right(current_group)
    }
}

pub fn run(config  : Config,
           dry_run : bool) {
    let state = Rc::new(State {
        config,
        path: RefCell::new(Vec::new()),
        execute: Cell::new(false),
    });

    let mut siv = Cursive::new();

    render_current_group(&mut siv,
                         state.clone(),
                         &state.config.root,
                         None,
                         dry_run);

    siv.run();
    drop(siv);  // to dispose of backend messing with the terminal

    if state.execute.get() {
        if let Either::Left(definition) = state.current_item() {
            let mut e = Execution::from(definition.clone());
            if dry_run {
                println!("{}", e.command_line());
            } else {
                e.run();
            }
        }
    }
}

fn render_current_group(s             : &mut Cursive,
                        state         : Rc<State>,
                        group         : &ConfigGroup,
                        last_selected : Option<String>,
                        dry_run       : bool) {
    s.pop_layer();

    let mut select = SelectView::<String>::new()
        .on_submit({
            let state = state.clone();
            move |s: &mut Cursive, name: &String|
            handle_selection_submit(s,
                                    state.clone(),
                                    name,
                                    dry_run)
        });

    let mut items = Vec::new();

    if !state.path.borrow().is_empty() {
        items.push(("../".into(), "".into()));
    }

    let groups = group.definitions.iter().filter(|&(_, i)| i.is_group());
    let definitions = group.definitions.iter().filter(|&(_, i)| !i.is_group());
    for (k, _) in groups {
        items.push((k.clone() + "/", k.clone()));
    }
    for (k, _) in definitions {
        items.push((k.clone(), k.clone()));
    }

    select.add_all(items.iter().cloned());

    if let Some(last_selected) = last_selected {
        if let Some(idx) = items.iter().position(|&(_, ref v)| v == &last_selected) {
            select.set_selection(idx);
        }
    }

    let select = OnEventView::new(select)
        .on_pre_event('e', {
            let state = state.clone();
            move |s| {
                s.call_on_id("select", |sel: &mut SelectView| {
                    let selection = (*sel.selection()).clone();
                    state.path.borrow_mut().push(selection);
                });
                execute_definition(s, state.clone());
            }
        })
        .on_pre_event_inner('k', |s| {
            s.select_up(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner('j', |s| {
            s.select_down(1);
            Some(EventResult::Consumed(None))
        })
        .with_id("select");

    let layout = LinearLayout::vertical()
        .child(TextView::new(format_path(state.path.borrow().iter())))
        .child(DummyView)
        .child(select.fixed_size((80, 10)))
        .child(DummyView);

    s.add_layer(
        Dialog::around(layout)
            .title("Profiles")
            .button("Quit", |s| s.quit())
    );

    // up one level
    s.add_global_callback(Key::Esc, {
        let state = state.clone();
        move |s| handle_selection_submit(s, state.clone(), "", dry_run)
    });
}

fn render_current_definition(s          : &mut Cursive,
                             state      : Rc<State>,
                             definition : &ConfigDefinition,
                             dry_run    : bool) {
    s.pop_layer();

    let layout = LinearLayout::vertical()
        .child(TextView::new(
            format!("Will {} the following command:",
                    if dry_run { "print" } else { "execute" })))
        .child(DummyView)
        .child(TextView::new(Execution::from(definition.clone()).command_line()))
        .child(DummyView)
        .child(TextView::new(
            format!("Press Enter to {}, Esc to go back",
                    if dry_run { "print" } else { "run" })));

    s.add_layer(
        Dialog::around(layout)
            .title(format!("Profile {}", state.path.borrow().iter().join(".")))
    );

    s.add_global_callback(Key::Enter, {
        let state = state.clone();
        move |s| execute_definition(s, state.clone())
    });

    s.add_global_callback(Key::Esc, {
        let state = state.clone();
        move |s: &mut Cursive| {
            s.add_global_callback(Key::Enter, |_| {});
            s.add_global_callback(Key::Esc, |_| {});
            handle_selection_submit(s, state.clone(), "", dry_run)
        }
    });
}

fn handle_selection_submit(s       : &mut Cursive,
                           state   : Rc<State>,
                           name    : &str,
                           dry_run : bool) {
    let last_selected = {
        let mut path = state.path.borrow_mut();
        if name.is_empty() {
            if path.is_empty() {
                s.quit();
                None
            } else {
                path.pop()
            }
        } else {
            path.push(name.into());
            None
        }
    };

    match state.current_item() {
        Either::Left(definition) => render_current_definition(s,
                                                              state.clone(),
                                                              definition,
                                                              dry_run),
        Either::Right(group)     => render_current_group(s,
                                                         state.clone(),
                                                         group,
                                                         last_selected,
                                                         dry_run),
    }
}

fn format_path<I: IntoIterator>(path: I) -> String where I::Item: AsRef<str> {
    let mut result = String::new();

    for part in path {
        result.push_str("/");
        result.push_str(part.as_ref());
    }

    if result.is_empty() {
        result.push_str("/")
    }

    result
}

fn execute_definition(s: &mut Cursive, state: Rc<State>) {
    state.execute.set(true);
    s.quit();
}
