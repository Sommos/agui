use crate::{
    element::{Element, ElementId},
    unit::Offset,
    util::tree::Tree,
    widget::{ContextWidget, IterChildrenLayout, IterChildrenLayoutMut},
};

pub struct LayoutContext<'ctx> {
    pub(crate) element_tree: &'ctx mut Tree<ElementId, Element>,

    pub(crate) element_id: ElementId,

    pub(crate) children: &'ctx [ElementId],
    pub(crate) offsets: &'ctx mut [Offset],
}

impl ContextWidget for LayoutContext<'_> {
    fn get_elements(&self) -> &Tree<ElementId, Element> {
        self.element_tree
    }

    fn get_element_id(&self) -> ElementId {
        self.element_id
    }
}

impl LayoutContext<'_> {
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn iter_children(&self) -> IterChildrenLayout {
        IterChildrenLayout::new(self.element_tree, self.children)
    }

    pub fn iter_children_mut(&mut self) -> IterChildrenLayoutMut {
        IterChildrenLayoutMut::new(self.element_tree, self.children, self.offsets)
    }
}
