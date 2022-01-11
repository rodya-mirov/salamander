//! Probably overkill but using a typemap with a nice wrapper interface so we can handle large
//! numbers of types of events with less bevy boilerplate (no registering events) and more manual
//! control (easier to clear when I say I want to clear).

use std::marker::PhantomData;

pub struct CallbackEvents {
    map: typemap::ShareMap,
    clears: Vec<Box<dyn FnMut(&mut typemap::ShareMap) + Send + Sync + 'static>>,
}

pub trait CallbackEvent: 'static + Send + Sync + Clone + std::fmt::Debug {}

struct KeyWrapper<T>(PhantomData<T>);

impl<T: CallbackEvent> typemap::Key for KeyWrapper<T> {
    type Value = Vec<T>;
}

impl Default for CallbackEvents {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackEvents {
    pub fn new() -> CallbackEvents {
        CallbackEvents {
            map: typemap::ShareMap::custom(),
            clears: Vec::new(),
        }
    }

    /// Iterate through all events of the given type.
    pub fn iter<T: CallbackEvent>(&self) -> Box<dyn Iterator<Item = &T> + Send + Sync + '_> {
        let iter = self.map.get::<KeyWrapper<T>>().into_iter().flatten();
        Box::new(iter)
    }

    pub fn is_nonempty<T: CallbackEvent>(&self) -> bool {
        self.map
            .get::<KeyWrapper<T>>()
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }

    /// Send a new event to the event pile.
    pub fn send<T: CallbackEvent>(&mut self, event: T) {
        if self.map.get_mut::<KeyWrapper<T>>().is_none() {
            self.clears.push(Box::new(|map| {
                map.get_mut::<KeyWrapper<T>>().map(|v| v.clear());
            }));
        }
        self.map
            .entry::<KeyWrapper<T>>()
            .or_insert(Vec::new())
            .push(event);
    }

    pub fn clear(&mut self) {
        for clear in self.clears.iter_mut() {
            clear(&mut self.map);
        }
    }
}
