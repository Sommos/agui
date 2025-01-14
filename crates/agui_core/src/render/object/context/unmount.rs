use crate::{
    plugin::{
        context::{ContextPlugins, ContextPluginsMut},
        Plugins,
    },
    render::{RenderObject, RenderObjectId},
    util::tree::Tree,
};

use super::{ContextRenderObject, ContextRenderObjects};

pub struct RenderObjectUnmountContext<'ctx> {
    pub plugins: &'ctx mut Plugins,

    pub render_object_tree: &'ctx Tree<RenderObjectId, RenderObject>,

    pub render_object_id: &'ctx RenderObjectId,
}

impl<'ctx> ContextPlugins<'ctx> for RenderObjectUnmountContext<'ctx> {
    fn plugins(&self) -> &Plugins {
        self.plugins
    }
}

impl<'ctx> ContextPluginsMut<'ctx> for RenderObjectUnmountContext<'ctx> {
    fn plugins_mut(&mut self) -> &mut Plugins {
        self.plugins
    }
}

impl ContextRenderObjects for RenderObjectUnmountContext<'_> {
    fn render_objects(&self) -> &Tree<RenderObjectId, RenderObject> {
        self.render_object_tree
    }
}

impl ContextRenderObject for RenderObjectUnmountContext<'_> {
    fn render_object_id(&self) -> RenderObjectId {
        *self.render_object_id
    }
}
