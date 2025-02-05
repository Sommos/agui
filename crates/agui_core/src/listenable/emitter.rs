use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Weak},
};

use crate::listenable::Event;

#[allow(clippy::type_complexity)]
pub struct EventEmitter<T> {
    listeners: Rc<RefCell<Vec<Weak<dyn Fn(&T)>>>>,
}

impl<T> Default for EventEmitter<T> {
    fn default() -> Self {
        Self {
            listeners: Rc::default(),
        }
    }
}

impl<T> Clone for EventEmitter<T> {
    fn clone(&self) -> Self {
        Self {
            listeners: Rc::clone(&self.listeners),
        }
    }
}

impl<T> EventEmitter<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Event> EventEmitter<T> {
    pub fn emit(&self, value: &T) {
        self.listeners.borrow_mut().retain(|handle| {
            if let Some(handle) = handle.upgrade() {
                (handle)(value);
                true
            } else {
                false
            }
        });
    }

    #[must_use]
    pub fn add_listener(&self, func: impl Fn(&T) + 'static) -> EventEmitterHandle<T> {
        let func = Arc::new(func) as Arc<dyn Fn(&T)>;

        self.listeners.borrow_mut().push(Arc::downgrade(&func));

        EventEmitterHandle { _guard: func }
    }
}

impl<T: Event + PartialEq> EventEmitter<T> {
    #[must_use]
    pub fn on(&self, value: T, func: impl Fn() + 'static) -> EventEmitterHandle<T> {
        self.add_listener(move |received_value| {
            if received_value == &value {
                func();
            }
        })
    }
}

#[derive(Clone)]
pub struct EventEmitterHandle<T> {
    _guard: Arc<dyn Fn(&T)>,
}
