use crate::{
    element::{ContextElement, ContextElements, ContextMarkDirty, Element, ElementId},
    engine::DirtyElements,
    util::tree::Tree,
};

pub struct PluginElementMountContext<'ctx> {
    pub element_tree: &'ctx Tree<ElementId, Element>,
    pub dirty: &'ctx mut DirtyElements,

    pub parent_element_id: Option<&'ctx ElementId>,
    pub element_id: &'ctx ElementId,
    pub element: &'ctx Element,
}

impl ContextElements for PluginElementMountContext<'_> {
    fn elements(&self) -> &Tree<ElementId, Element> {
        self.element_tree
    }
}

impl ContextElement for PluginElementMountContext<'_> {
    fn element_id(&self) -> ElementId {
        *self.element_id
    }
}

impl ContextMarkDirty for PluginElementMountContext<'_> {
    fn mark_dirty(&mut self, element_id: ElementId) {
        self.dirty.insert(element_id);
    }
}

impl PluginElementMountContext<'_> {
    pub fn parent_element_id(&self) -> Option<ElementId> {
        self.parent_element_id.copied()
    }
}
