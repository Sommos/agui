use crate::{
    plugin::{context::ContextPlugins, Plugins},
    render::{object::context::IterChildrenLayout, RenderObject, RenderObjectId},
    util::tree::Tree,
};

use super::{ContextRenderObject, ContextRenderObjects};

pub struct RenderObjectIntrinsicSizeContext<'ctx> {
    pub plugins: &'ctx Plugins,

    pub render_object_tree: &'ctx Tree<RenderObjectId, RenderObject>,

    pub render_object_id: &'ctx RenderObjectId,

    pub children: &'ctx [RenderObjectId],
}

impl<'ctx> ContextPlugins<'ctx> for RenderObjectIntrinsicSizeContext<'ctx> {
    fn plugins(&self) -> &Plugins {
        self.plugins
    }
}

impl ContextRenderObjects for RenderObjectIntrinsicSizeContext<'_> {
    fn render_objects(&self) -> &Tree<RenderObjectId, RenderObject> {
        self.render_object_tree
    }
}

impl ContextRenderObject for RenderObjectIntrinsicSizeContext<'_> {
    fn render_object_id(&self) -> RenderObjectId {
        *self.render_object_id
    }
}

impl<'ctx> RenderObjectIntrinsicSizeContext<'ctx> {
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn iter_children(&self) -> IterChildrenLayout {
        IterChildrenLayout {
            index: 0,

            plugins: self.plugins,

            render_object_tree: self.render_object_tree,

            children: self.children,
        }
    }
}
