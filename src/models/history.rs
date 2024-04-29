use undo::Action as UndoAction;
use crate::interface::editing::EditingView;
use crate::models::modifier::Modifier;

pub enum Action {
    ModifierAdded(ModifierAdded),
    ModifierRemoved(ModifierRemoved),
    ModifierOptionsApplied(ModifierOptionsApplied),
    ModifierSelected(ModifierSelected)
}

pub struct ModifierAdded {
    modifier: Modifier,
    previous_selected: Option<(usize, Modifier)>
}

impl ModifierAdded {
    pub fn new(modifier: Modifier) -> Self {
        ModifierAdded {
            modifier,
            previous_selected: None,
        }
    }
}

pub struct ModifierRemoved {
    idx: usize,
    modifier: Option<Modifier>,
    was_selected: bool
}

impl ModifierRemoved {
    pub fn new(idx: usize) -> Self {
        ModifierRemoved {
            idx,
            modifier: None,
            was_selected: false,
        }
    }
}

pub struct ModifierOptionsApplied {
    previous: Option<Modifier>
}

impl ModifierOptionsApplied {
    pub fn new() -> Self {
        Self {
            previous: None,
        }
    }
}

pub struct ModifierSelected {
    idx: usize,
    modifier: Modifier,
    previous: Option<(usize, Modifier)>
}

impl ModifierSelected {
    pub fn new(idx: usize, modifier: Modifier) -> Self {
        Self {
            idx,
            modifier,
            previous: None,
        }
    }
}

impl UndoAction for Action {
    type Target = EditingView;
    type Output = ();

    fn apply(&mut self, target: &mut Self::Target) -> Self::Output {
        match self {
            Action::ModifierAdded(data) => {
                target.modifiers.push(data.modifier.clone());
                data.previous_selected = target.selected_modifier.clone();
                target.selected_modifier = Some((target.modifiers.len() - 1, data.modifier.clone()));
            }
            Action::ModifierRemoved(data) => {
                if let Some((i, _)) = &target.selected_modifier {
                    if *i == data.idx {
                        data.was_selected = true;
                        target.selected_modifier = None;
                    }
                }
                data.modifier = Some(target.modifiers.remove(data.idx));
            }
            Action::ModifierOptionsApplied(data) => {
                let selected = target.selected_modifier.clone().unwrap();
                data.previous = Some(target.modifiers[selected.0].clone());
                target.modifiers[selected.0] = selected.1;
            }
            Action::ModifierSelected(data) => {
                data.previous = target.selected_modifier.clone();
                if let Some((i, _)) = &target.selected_modifier {
                    if *i == data.idx {
                        target.selected_modifier = None;
                    } else {
                        target.selected_modifier = Some((data.idx, data.modifier.clone()));
                    }
                } else {
                    target.selected_modifier = Some((data.idx, data.modifier.clone()));
                }
            }
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> Self::Output {
        match self {
            Action::ModifierAdded(data) => {
                target.selected_modifier = data.previous_selected.clone();
                data.modifier = target.modifiers.pop().unwrap()
            }
            Action::ModifierRemoved(data) => {
                target.modifiers.insert(data.idx, data.modifier.clone().unwrap());
                if data.was_selected {
                    target.selected_modifier = Some((data.idx, data.modifier.clone().unwrap()));
                }
                data.was_selected = false;
            }
            Action::ModifierOptionsApplied(data) => {
                let selected = target.selected_modifier.clone().unwrap();
                target.modifiers[selected.0] = data.previous.clone().unwrap();
                if let Some((idx, _)) = &target.selected_modifier {
                    if *idx == selected.0 {
                        target.selected_modifier = Some((*idx, data.previous.clone().unwrap()))
                    }
                }
                data.previous = None;
            }
            Action::ModifierSelected(data) => {
                target.selected_modifier = data.previous.clone();
            }
        }
    }
}
