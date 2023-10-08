use crate::{
    callback::{CallbackQueue, ContextCallbackQueue},
    element::{Element, ElementId},
    engine::DirtyElements,
    plugin::{
        context::{ContextPlugins, ContextPluginsMut},
        Plugins,
    },
    util::tree::Tree,
};

use super::{ContextElement, ContextMarkDirty};

pub struct ElementBuildContext<'ctx> {
    pub plugins: &'ctx mut Plugins,

    pub element_tree: &'ctx Tree<ElementId, Element>,
    pub dirty: &'ctx mut DirtyElements,
    pub callback_queue: &'ctx CallbackQueue,

    pub element_id: &'ctx ElementId,
}

impl<'ctx> ContextPlugins<'ctx> for ElementBuildContext<'ctx> {
    fn get_plugins(&self) -> &Plugins {
        self.plugins
    }
}

impl<'ctx> ContextPluginsMut<'ctx> for ElementBuildContext<'ctx> {
    fn get_plugins_mut(&mut self) -> &mut Plugins {
        self.plugins
    }
}

impl ContextElement for ElementBuildContext<'_> {
    fn get_elements(&self) -> &Tree<ElementId, Element> {
        self.element_tree
    }

    fn get_element_id(&self) -> ElementId {
        *self.element_id
    }
}

impl ContextMarkDirty for ElementBuildContext<'_> {
    fn mark_dirty(&mut self, element_id: ElementId) {
        self.dirty.insert(element_id);
    }
}

impl ContextCallbackQueue for ElementBuildContext<'_> {
    fn get_callback_queue(&self) -> &CallbackQueue {
        self.callback_queue
    }
}
