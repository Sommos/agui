use slotmap::hop::Iter;

use crate::{
    element::{Element, ElementId},
    unit::Key,
    util::tree::{Tree, TreeNode},
    widget::Widget,
};

pub mod by_key;
pub mod by_type;

use self::{by_key::QueryByKey, by_type::QueryByType};

pub struct WidgetQuery<'query> {
    pub iter: Iter<'query, ElementId, TreeNode<ElementId, Element>>,
}

impl<'query> WidgetQuery<'query> {
    pub(crate) fn new(tree: &'query Tree<ElementId, Element>) -> WidgetQuery<'query> {
        WidgetQuery { iter: tree.iter() }
    }
}

impl<'query> Iterator for WidgetQuery<'query> {
    type Item = &'query Element;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find_map(|(_, node)| node.value.as_ref())
    }
}

pub trait WidgetQueryExt<'query> {
    fn by_key(self, key: Key) -> QueryByKey<Self>
    where
        Self: Sized;

    fn by_type<W>(self) -> QueryByType<Self, W>
    where
        Self: Sized,
        W: Widget;
}

impl<'query, I> WidgetQueryExt<'query> for I
where
    I: Iterator<Item = &'query Element>,
{
    fn by_key(self, key: Key) -> QueryByKey<Self> {
        QueryByKey::new(self, key)
    }

    fn by_type<W>(self) -> QueryByType<Self, W>
    where
        W: Widget,
    {
        QueryByType::<Self, W>::new(self)
    }
}
