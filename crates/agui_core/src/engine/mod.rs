use std::collections::VecDeque;

use rustc_hash::FxHashMap;

use crate::{
    callback::{CallbackInvoke, CallbackQueue},
    element::{
        Element, ElementBuildContext, ElementCallbackContext, ElementId, ElementMountContext,
        ElementUnmountContext, ElementUpdate,
    },
    engine::event::{ElementDestroyedEvent, ElementSpawnedEvent},
    listenable::EventBus,
    plugin::{
        context::{
            ContextPlugins, PluginAfterUpdateContext, PluginBeforeUpdateContext,
            PluginElementBuildContext, PluginElementMountContext, PluginElementUnmountContext,
            PluginInitContext,
        },
        Plugins,
    },
    query::WidgetQuery,
    render::{RenderObject, RenderObjectContextMut, RenderObjectId},
    unit::{Constraints, Key},
    util::{map::ElementSet, tree::Tree},
    widget::Widget,
};

use self::{builder::EngineBuilder, event::ElementRebuiltEvent};

pub mod builder;
mod dirty;
pub mod event;

pub use dirty::DirtyElements;

pub struct Engine {
    plugins: Plugins,

    bus: EventBus,

    element_tree: Tree<ElementId, Element>,
    render_object_tree: Tree<RenderObjectId, RenderObject>,

    dirty: DirtyElements,
    callback_queue: CallbackQueue,

    rebuild_queue: VecDeque<ElementId>,
    removal_queue: ElementSet,

    sync_render_object_children: ElementSet,
    create_render_object: VecDeque<ElementId>,
    update_render_object: ElementSet,
}

impl ContextPlugins<'_> for Engine {
    fn plugins(&self) -> &Plugins {
        &self.plugins
    }
}

impl Engine {
    pub fn builder() -> EngineBuilder<()> {
        EngineBuilder::new()
    }

    pub fn events(&self) -> &EventBus {
        &self.bus
    }

    /// Get the element tree.
    pub fn elements(&self) -> &Tree<ElementId, Element> {
        &self.element_tree
    }

    /// Get the render object tree.
    pub fn render_objects(&self) -> &Tree<RenderObjectId, RenderObject> {
        &self.render_object_tree
    }

    /// Get the root widget.
    pub fn root(&self) -> ElementId {
        self.element_tree.root().expect("root is not set")
    }

    /// Check if a widget exists in the tree.
    pub fn contains(&self, element_id: ElementId) -> bool {
        self.element_tree.contains(element_id)
    }

    /// Query widgets from the tree.
    ///
    /// This essentially iterates the widget tree's element Vec, and as such does not guarantee
    /// the order in which widgets will be returned.
    pub fn query(&self) -> WidgetQuery {
        WidgetQuery::new(&self.element_tree)
    }

    pub fn callback_queue(&self) -> &CallbackQueue {
        &self.callback_queue
    }

    pub fn has_changes(&self) -> bool {
        !self.rebuild_queue.is_empty() || !self.dirty.is_empty() || !self.callback_queue.is_empty()
    }

    /// Mark a widget as dirty, causing it to be rebuilt on the next update.
    pub fn mark_dirty(&mut self, element_id: ElementId) {
        self.dirty.insert(element_id);
    }

    /// Initializes plugins and sets the initial root widget, but does not build it or spawn
    /// any children.
    ///
    /// This keeps the initial engine creation fast, and allows the user to delay the
    /// first build until they are ready. This does, however, that the root element has
    /// slightly different semantics. It will be mounted but not built until the first
    /// update.
    fn init(&mut self, root: Widget) {
        self.plugins.on_init(&mut PluginInitContext {
            bus: &self.bus,

            element_tree: &self.element_tree,
        });

        let root_id = self.process_spawn(None, root);

        self.rebuild_queue.push_back(root_id);
    }

    /// Update the UI tree.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn update(&mut self) {
        tracing::debug!("updating widget tree");

        self.plugins
            .on_before_update(&mut PluginBeforeUpdateContext {
                element_tree: &self.element_tree,
            });

        // Update everything until all widgets fall into a stable state. Incorrectly set up widgets may
        // cause an infinite loop, so be careful.
        'layout: loop {
            'changes: loop {
                self.flush_rebuilds();

                self.flush_dirty();

                self.flush_callbacks();

                if !self.has_changes() {
                    break 'changes;
                }
            }

            // We sync render after the rebuild loop to prevent unnecessary work keeping the render
            // tree up-to-date. This is done before `flush_removals` so that we can steal any render
            // objects that would otherwise be removed.
            self.sync_render_objects();

            self.flush_removals();

            self.flush_layout();

            if !self.has_changes() {
                break 'layout;
            }
        }

        self.plugins.on_after_update(&mut PluginAfterUpdateContext {
            element_tree: &self.element_tree,
        });
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn flush_rebuilds(&mut self) {
        // Apply any queued modifications
        while let Some(element_id) = self.rebuild_queue.pop_front() {
            self.process_rebuild(element_id);
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn flush_dirty(&mut self) {
        for element_id in self.dirty.drain() {
            tracing::trace!(
                ?element_id,
                widget = self.element_tree.get(element_id).unwrap().widget_name(),
                "queueing widget for rebuild"
            );

            self.rebuild_queue.push_back(element_id);
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn flush_callbacks(&mut self) {
        let callback_invokes = self.callback_queue.take();

        for CallbackInvoke {
            callback_id,
            arg: callback_arg,
        } in callback_invokes
        {
            let element_id = callback_id.element_id();

            self.element_tree
                .with(element_id, |element_tree, element| {
                    let changed = element.call(
                        ElementCallbackContext {
                            plugins: &mut self.plugins,

                            element_tree,
                            dirty: &mut self.dirty,

                            element_id: &element_id,
                        },
                        callback_id,
                        callback_arg,
                    );

                    if changed {
                        tracing::debug!(
                            ?element_id,
                            widget = element.widget_name(),
                            "element updated, queueing for rebuild"
                        );

                        self.rebuild_queue.push_back(element_id);
                    }
                })
                .expect("cannot call a callback on a widget that does not exist");
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn flush_layout(&mut self) {
        let Some(root_id) = self.render_object_tree.root() else {
            return;
        };

        // TODO: Layout using a loop rather than deeply recursively
        self.render_object_tree
            .with(root_id, |render_object_tree, render_object| {
                render_object.layout(
                    RenderObjectContextMut {
                        plugins: &mut self.plugins,

                        render_object_tree,

                        render_object_id: &root_id,
                    },
                    // The root element is always unbounded
                    Constraints::expand(),
                );
            })
            .expect("cannot layout a widget that doesn't exist");
    }

    #[tracing::instrument(level = "trace", name = "spawn", skip(self))]
    fn process_spawn(&mut self, parent_id: Option<ElementId>, widget: Widget) -> ElementId {
        let element = Element::new(widget.clone());

        tracing::trace!("spawning widget");

        let element_id = self.element_tree.add(parent_id, element);

        self.element_tree.with(element_id, |element_tree, element| {
            self.plugins
                .on_element_mount(&mut PluginElementMountContext {
                    element_tree,
                    dirty: &mut self.dirty,

                    parent_element_id: parent_id.as_ref(),
                    element_id: &element_id,
                    element,
                });

            element.mount(ElementMountContext {
                plugins: &mut self.plugins,

                element_tree,
                dirty: &mut self.dirty,

                parent_element_id: parent_id.as_ref(),
                element_id: &element_id,
            });
        });

        self.create_render_object.push_back(element_id);

        self.bus.emit(&ElementSpawnedEvent {
            parent_id,
            element_id,
        });

        element_id
    }

    #[tracing::instrument(level = "trace", name = "build", skip(self, element_id))]
    fn process_build(&mut self, element_id: ElementId) {
        let mut build_queue = VecDeque::new();

        build_queue.push_back(element_id);

        while let Some(element_id) = build_queue.pop_front() {
            let new_widgets = self
                .element_tree
                .with(element_id, |element_tree, element| {
                    self.plugins
                        .on_element_build(&mut PluginElementBuildContext {
                            element_tree,
                            dirty: &mut self.dirty,
                            callback_queue: &self.callback_queue,

                            element_id: &element_id,
                            element,
                        });

                    element.build(ElementBuildContext {
                        plugins: &mut self.plugins,

                        element_tree,
                        dirty: &mut self.dirty,
                        callback_queue: &self.callback_queue,

                        element_id: &element_id,
                    })
                })
                .expect("cannot build a widget that doesn't exist");

            self.bus.emit(&ElementRebuiltEvent { element_id });

            if new_widgets.is_empty() {
                continue;
            }

            let old_children = self
                .element_tree
                .get_children(element_id)
                .expect("newly created element does not exist in the tree")
                .clone();

            let mut new_children_top = 0;
            let mut old_children_top = 0;
            let mut new_children_bottom = new_widgets.len() as i32 - 1;
            let mut old_children_bottom = old_children.len() as i32 - 1;

            let mut new_children = vec![None; new_widgets.len()];

            // Update the top of the list.
            while (old_children_top <= old_children_bottom)
                && (new_children_top <= new_children_bottom)
            {
                let old_child_id = old_children.get(old_children_top as usize).copied();
                let new_widget = new_widgets.get(new_children_top as usize);

                if let Some((old_child_id, new_widget)) = old_child_id.zip(new_widget) {
                    let old_child = self
                        .element_tree
                        .get_mut(old_child_id)
                        .expect("child element does not exist in the tree");

                    match old_child.update(new_widget) {
                        ElementUpdate::Noop => {
                            tracing::trace!(
                                parent_id = ?element_id,
                                element_id = ?old_child_id,
                                widget = ?new_widget,
                                old_position = old_children_top,
                                new_position = new_children_top,
                                "element was retained"
                            );
                        }

                        ElementUpdate::RebuildNecessary => {
                            tracing::trace!(
                                parent_id = ?element_id,
                                element_id = ?old_child_id,
                                widget = ?new_widget,
                                old_position = old_children_top,
                                new_position = new_children_top,
                                "element was retained but must be rebuilt"
                            );

                            self.rebuild_queue.push_back(old_child_id);
                            self.update_render_object.insert(old_child_id);
                        }

                        ElementUpdate::Invalid => break,
                    }

                    new_children[new_children_top as usize] = Some(old_child_id);
                } else {
                    break;
                }

                new_children_top += 1;
                old_children_top += 1;
            }

            // Scan the bottom of the list.
            while (old_children_top <= old_children_bottom)
                && (new_children_top <= new_children_bottom)
            {
                let old_child_id = old_children.get(old_children_bottom as usize).copied();
                let new_widget = new_widgets.get(new_children_bottom as usize);

                if let Some((old_child_id, new_widget)) = old_child_id.zip(new_widget) {
                    let old_child = self
                        .element_tree
                        .get_mut(old_child_id)
                        .expect("child element does not exist in the tree");

                    match old_child.update(new_widget) {
                        ElementUpdate::Noop => {
                            tracing::trace!(
                                parent_id = ?element_id,
                                element_id = ?old_child_id,
                                widget = ?new_widget,
                                old_position = old_children_bottom,
                                new_position = new_children_bottom,
                                "element was retained"
                            );
                        }

                        ElementUpdate::RebuildNecessary => {
                            tracing::trace!(
                                parent_id = ?element_id,
                                element_id = ?old_child_id,
                                widget = ?new_widget,
                                position = new_children_top,
                                "element was retained but must be rebuilt"
                            );

                            self.rebuild_queue.push_back(old_child_id);

                            // If the child has a render object, we need to update it.
                            if old_child.render_object_id().is_some() {
                                self.update_render_object.insert(old_child_id);
                            }
                        }

                        ElementUpdate::Invalid => break,
                    }
                } else {
                    break;
                }

                old_children_bottom -= 1;
                new_children_bottom -= 1;
            }

            // Scan the old children in the middle of the list.
            let have_old_children = old_children_top <= old_children_bottom;
            let mut old_keyed_children = FxHashMap::<Key, ElementId>::default();

            while old_children_top <= old_children_bottom {
                if let Some(old_child_id) = old_children.get(old_children_top as usize) {
                    let old_child = self
                        .element_tree
                        .get(*old_child_id)
                        .expect("child element does not exist in the tree");

                    if let Some(key) = old_child.widget().key() {
                        old_keyed_children.insert(key, *old_child_id);
                    } else {
                        // unmount / deactivate the child
                    }
                }

                old_children_top += 1;
            }

            // Update the middle of the list.
            while new_children_top <= new_children_bottom {
                let new_widget = &new_widgets[new_children_top as usize];

                if have_old_children {
                    if let Some(key) = new_widget.key() {
                        if let Some(old_child_id) = old_keyed_children.get(&key).copied() {
                            let old_child = self
                                .element_tree
                                .get_mut(old_child_id)
                                .expect("child element does not exist in the tree");

                            match old_child.update(new_widget) {
                                ElementUpdate::Noop => {
                                    tracing::trace!(
                                        parent_id = ?element_id,
                                        element_id = ?old_child_id,
                                        widget = ?new_widget,
                                        key = ?key,
                                        new_position = new_children_top,
                                        "keyed element was retained"
                                    );
                                }

                                ElementUpdate::RebuildNecessary => {
                                    tracing::trace!(
                                        parent_id = ?element_id,
                                        element_id = ?old_child_id,
                                        widget = ?new_widget,
                                        key = ?key,
                                        new_position = new_children_top,
                                        "keyed element was retained but must be rebuilt"
                                    );

                                    self.rebuild_queue.push_back(old_child_id);

                                    // If the child has a render object, we need to update it.
                                    if old_child.render_object_id().is_some() {
                                        self.update_render_object.insert(old_child_id);
                                    }
                                }

                                ElementUpdate::Invalid => break,
                            }

                            // Remove it from the list so that we don't try to use it again.
                            old_keyed_children.remove(&key);

                            new_children[new_children_top as usize] = Some(old_child_id);
                            new_children_top += 1;

                            continue;
                        }
                    }
                }

                let new_child_id = self.process_spawn(Some(element_id), new_widget.clone());

                new_children[new_children_top as usize] = Some(new_child_id);
                new_children_top += 1;

                build_queue.push_back(new_child_id);
            }

            // We've scanned the whole list.
            assert!(old_children_top == old_children_bottom + 1);
            assert!(new_children_top == new_children_bottom + 1);
            assert!(
                new_widgets.len() as i32 - new_children_top
                    == old_children.len() as i32 - old_children_top
            );

            new_children_bottom = new_widgets.len() as i32 - 1;
            old_children_bottom = old_children.len() as i32 - 1;

            // Update the bottom of the list.
            while (old_children_top <= old_children_bottom)
                && (new_children_top <= new_children_bottom)
            {
                new_children[new_children_top as usize] =
                    Some(old_children[old_children_top as usize]);
                new_children_top += 1;
                old_children_top += 1;
            }

            // Clean up any of the remaining middle nodes from the old list.
            // for old_keyed_child_id in old_keyed_children {
            //     // deactivate the child
            // }

            // The list of new children should never have any holes in it.
            let new_children = new_children
                .into_iter()
                .map(Option::unwrap)
                .collect::<Vec<_>>();

            // If the list of children has changed, we need to make sure the parent has its
            // render object child order updated as well.
            if old_children != new_children {
                self.sync_render_object_children.insert(element_id);
            }

            for child_id in new_children {
                self.removal_queue.remove(&child_id);

                // reparent each child
                if self.element_tree.reparent(Some(element_id), child_id) {
                    panic!("element should have remained as a child of the same parent")
                }
            }
        }
    }

    #[tracing::instrument(level = "trace", name = "rebuild", skip(self))]
    fn process_rebuild(&mut self, element_id: ElementId) {
        // Grab the current children so we know which ones to remove post-build
        let children = self
            .element_tree
            .get_children(element_id)
            .map(Vec::clone)
            .unwrap_or_default();

        // Add the children to the removal queue. If any wish to be retained, they will be
        // removed from the queue during `process_build`.
        for child_id in children {
            self.removal_queue.insert(child_id);
        }

        self.process_build(element_id);
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn flush_removals(&mut self) {
        let mut destroy_queue = self.removal_queue.drain().collect::<VecDeque<_>>();

        while let Some(element_id) = destroy_queue.pop_front() {
            // Queue the element's children for removal
            if let Some(children) = self.element_tree.get_children(element_id) {
                for child_id in children {
                    destroy_queue.push_back(*child_id);
                }
            }

            self.element_tree
                .with(element_id, |element_tree, element| {
                    self.plugins
                        .on_element_unmount(&mut PluginElementUnmountContext {
                            element_tree,
                            dirty: &mut self.dirty,

                            element_id: &element_id,
                            element,
                        });

                    element.unmount(ElementUnmountContext {
                        plugins: &mut self.plugins,

                        element_tree,
                        dirty: &mut self.dirty,

                        element_id: &element_id,
                    });
                })
                .expect("cannot destroy an element that doesn't exist");

            self.bus.emit(&ElementDestroyedEvent { element_id });

            let element = self.element_tree.remove(element_id, false).unwrap();

            let widget = element.widget();

            tracing::trace!(?element_id, ?widget, "destroyed widget");
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn create_render_object(&mut self, element_id: ElementId) -> Option<RenderObjectId> {
        // No point in creating a render object for an element that is being removed.
        if self.removal_queue.contains(&element_id) {
            return None;
        }

        let parent_render_object_id =
            self.element_tree
                .get_parent(element_id)
                .map(|parent_element_id| {
                    self.element_tree
                        .get(parent_element_id)
                        .expect("parent element missing while creating render objects")
                        .render_object_id()
                        .expect("parent element has no render object")
                });

        let element = self
            .element_tree
            .get_mut(element_id)
            .expect("element missing while creating render objects");

        // If we've already created a render object for this element, skip it.
        if let Some(render_object_id) = element.render_object_id() {
            return Some(render_object_id);
        }

        let render_object_id = self
            .render_object_tree
            .add(parent_render_object_id, element.create_render_object());

        element.set_render_object_id(render_object_id);

        Some(render_object_id)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn sync_render_objects(&mut self) {
        let mut sync_render_object_queue = self
            .sync_render_object_children
            .drain()
            .filter(|element_id| !self.removal_queue.contains(element_id))
            .collect::<VecDeque<_>>();

        while let Some(element_id) = sync_render_object_queue.pop_front() {
            // Elements that were removed should still be available in the tree, so this should
            // never fail.
            let element_node = self
                .element_tree
                .get_node(element_id)
                .expect("element missing while syncing render object children");

            if let Some(render_object_id) = element_node.value().render_object_id() {
                let mut first_child_render_object_id = None;

                let children = element_node.children().to_vec();

                // Yank the render objects of the element's children from wheverever they are in
                // the tree to the end of the list.
                for child_id in children {
                    let child_render_object_id = self
                        .element_tree
                        .get(child_id)
                        .expect("child element missing while syncing render object children")
                        .render_object_id();

                    let child_render_object_id =
                        if let Some(child_render_object_id) = child_render_object_id {
                            self.render_object_tree
                                .reparent(Some(render_object_id), child_render_object_id);

                            child_render_object_id
                        } else {
                            // If they don't already have a render object, create it.
                            if let Some(render_object_id) = self.create_render_object(child_id) {
                                render_object_id
                            } else {
                                // If the child is being removed, it won't have a render object.
                                continue;
                            }
                        };

                    if first_child_render_object_id.is_none() {
                        first_child_render_object_id = Some(child_render_object_id);
                    }
                }

                let children = self
                    .render_object_tree
                    .get_children(render_object_id)
                    .expect("element has a render object but the render object is missing")
                    .clone();

                // Remove any render objects that were previously children but are no longer.
                // Since the `reparent` call reorders them to the end of the list, we can remove
                // every child from the beginning of the list until we reach the first child
                // that is still a child of the element.
                for child_id in children {
                    if first_child_render_object_id == Some(child_id) {
                        break;
                    }

                    self.render_object_tree.remove(child_id, false);
                }
            }
        }

        while let Some(element_id) = self.create_render_object.pop_front() {
            self.create_render_object(element_id);
        }

        // Remove any render objects owned by elements that are being removed.
        for element_id in self.removal_queue.iter().copied() {
            if let Some(render_object_id) = self
                .element_tree
                .get(element_id)
                .expect("element missing while syncing render object children")
                .render_object_id()
            {
                self.render_object_tree.remove(render_object_id, false);
            }
        }

        for element_id in self.update_render_object.drain() {
            let element = self
                .element_tree
                .get(element_id)
                .expect("element missing while updating render objects");

            let render_object_id = element
                .render_object_id()
                .expect("element has no render object to update");

            let render_object = self
                .render_object_tree
                .get_mut(render_object_id)
                .expect("render object missing while updating");

            element.update_render_object(render_object);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use rustc_hash::FxHashSet;

    use crate::{
        element::mock::{render::MockRenderWidget, DummyRenderObject, DummyWidget},
        engine::event::{ElementDestroyedEvent, ElementRebuiltEvent, ElementSpawnedEvent},
        plugin::{context::ContextPlugins, Plugin},
        widget::IntoWidget,
    };

    use super::Engine;

    #[test]
    pub fn adding_a_root_widget() {
        let mut engine = Engine::builder().with_root(DummyWidget).build();

        let did_rebuild = Rc::new(RefCell::new(None));

        let _handler = engine.events().add_listener::<ElementRebuiltEvent>({
            let did_rebuild = Rc::clone(&did_rebuild);

            move |event| {
                *did_rebuild.borrow_mut() = Some(event.element_id);
            }
        });

        engine.update();

        let root_id = engine.root();

        assert_eq!(
            *did_rebuild.borrow(),
            Some(root_id),
            "should have emitted a rebuild event for the root"
        );

        let render_object_id = engine
            .elements()
            .get(root_id)
            .expect("no element found for the root widget")
            .render_object_id()
            .expect("no render object attached to the root element");

        let root_render_object_id = engine
            .render_objects()
            .root()
            .expect("no root render object");

        assert_eq!(render_object_id, root_render_object_id);

        engine
            .render_object_tree
            .get(render_object_id)
            .expect("should have created a render object for the root element");
    }

    #[test]
    pub fn rebuilding_widgets() {
        let mut engine = Engine::builder().with_root(DummyWidget).build();

        engine.update();

        let root_id = engine.root();

        let did_rebuild = Rc::new(RefCell::new(false));

        let _handler = engine.events().add_listener::<ElementRebuiltEvent>({
            let did_rebuild = Rc::clone(&did_rebuild);

            move |event| {
                if event.element_id != root_id {
                    return;
                }

                *did_rebuild.borrow_mut() = true;
            }
        });

        engine.mark_dirty(root_id);

        engine.update();

        assert!(*did_rebuild.borrow(), "should have emitted a rebuild event");
    }

    #[test]
    pub fn spawns_children() {
        let root_widget = MockRenderWidget::new("RootWidget");
        {
            root_widget
                .mock
                .borrow_mut()
                .expect_children()
                .returning(|| vec![DummyWidget.into_widget(), DummyWidget.into_widget()]);

            root_widget
                .mock
                .borrow_mut()
                .expect_create_render_object()
                .returning(|| DummyRenderObject.into());
        }

        let mut engine = Engine::builder().with_root(root_widget).build();

        let widgets_spawned = Rc::new(RefCell::new(FxHashSet::default()));

        let _handler = engine.events().add_listener::<ElementSpawnedEvent>({
            let widgets_spawned = Rc::clone(&widgets_spawned);

            move |event| {
                widgets_spawned.borrow_mut().insert(event.element_id);
            }
        });

        engine.update();

        let root_id = engine.root();

        assert_eq!(
            engine.elements().len(),
            3,
            "children should have been added"
        );

        assert_eq!(
            engine.render_objects().len(),
            3,
            "child render objects should have been added"
        );

        let children = engine.elements().get_children(root_id).unwrap();

        assert_eq!(children.len(), 2, "root should have two children");

        assert!(
            widgets_spawned.borrow().contains(&children[0]),
            "should have emitted a spawn event for the first child"
        );

        assert!(
            widgets_spawned.borrow().contains(&children[1]),
            "should have emitted a spawn event for the second child"
        );

        println!("{:?}", engine.element_tree);
        println!("{:?}", engine.render_object_tree);
    }

    #[test]
    pub fn removes_children() {
        let children = Rc::new(RefCell::new({
            let mut children = Vec::new();

            for _ in 0..1000 {
                children.push(DummyWidget.into_widget());
            }

            children
        }));

        let root_widget = MockRenderWidget::new("RootWidget");
        {
            root_widget
                .mock
                .borrow_mut()
                .expect_children()
                .returning_st({
                    let children = Rc::clone(&children);

                    move || children.borrow().clone()
                });

            root_widget
                .mock
                .borrow_mut()
                .expect_create_render_object()
                .returning(|| DummyRenderObject.into());
        }

        let mut engine = Engine::builder().with_root(root_widget).build();

        engine.update();

        assert_eq!(
            engine.elements().len(),
            1001,
            "children should have been added"
        );

        assert_eq!(
            engine.render_objects().len(),
            1001,
            "child render objects should have been added"
        );

        children.borrow_mut().clear();

        let root_id = engine.root();

        let widgets_destroyed = Rc::new(RefCell::new(FxHashSet::default()));

        let _handler = engine.events().add_listener::<ElementDestroyedEvent>({
            let widgets_destroyed = Rc::clone(&widgets_destroyed);

            move |event| {
                widgets_destroyed.borrow_mut().insert(event.element_id);
            }
        });

        engine.mark_dirty(root_id);

        engine.update();

        assert_eq!(
            engine.elements().len(),
            1,
            "nested children should have been removed"
        );

        assert_eq!(
            widgets_destroyed.borrow().len(),
            1000,
            "should have emitted a destroyed event for all children"
        );

        assert_eq!(
            engine.render_object_tree.len(),
            1,
            "root root render object should remain"
        );
    }

    #[test]
    pub fn rebuilds_children() {
        let child = Rc::new(RefCell::new(DummyWidget.into_widget()));

        let root_widget = MockRenderWidget::new("RootWidget");
        {
            root_widget
                .mock
                .borrow_mut()
                .expect_children()
                .returning_st({
                    let child = Rc::clone(&child);

                    move || vec![child.borrow().clone()]
                });

            root_widget
                .mock
                .borrow_mut()
                .expect_create_render_object()
                .returning(|| DummyRenderObject.into());
        }

        let mut engine = Engine::builder().with_root(root_widget).build();

        engine.update();

        let root_id = engine.root();

        let widgets_rebuilt = Rc::new(RefCell::new(FxHashSet::default()));

        let _handler = engine.events().add_listener::<ElementRebuiltEvent>({
            let widgets_rebuilt = Rc::clone(&widgets_rebuilt);

            move |event| {
                widgets_rebuilt.borrow_mut().insert(event.element_id);
            }
        });

        engine.mark_dirty(root_id);

        *child.borrow_mut() = DummyWidget.into_widget();

        engine.update();

        assert!(
            widgets_rebuilt.borrow().contains(&root_id),
            "should have emitted a rebuild event for the root widget"
        );

        assert_eq!(
            widgets_rebuilt.borrow().len(),
            2,
            "should have generated rebuild event for the child"
        );
    }

    #[test]
    pub fn reuses_unchanged_widgets() {
        let root_widget = MockRenderWidget::new("RootWidget");
        {
            root_widget
                .mock
                .borrow_mut()
                .expect_children()
                .returning_st(|| vec![DummyWidget.into_widget()]);

            root_widget
                .mock
                .borrow_mut()
                .expect_create_render_object()
                .returning(|| DummyRenderObject.into());
        }

        let mut engine = Engine::builder().with_root(root_widget).build();

        engine.update();

        let root_id = engine.root();
        let element_id = engine
            .elements()
            .get_children(root_id)
            .cloned()
            .expect("no children");

        engine.mark_dirty(engine.root());

        engine.update();

        assert_eq!(
            root_id,
            engine.root(),
            "root widget should have remained unchanged"
        );

        assert_eq!(
            element_id,
            engine
                .elements()
                .get_children(root_id)
                .cloned()
                .expect("no children"),
            "root widget should not have regenerated its child"
        );
    }

    #[derive(Debug)]
    struct TestPlugin1;

    impl Plugin for TestPlugin1 {}

    #[derive(Debug)]
    struct TestPlugin2;

    impl Plugin for TestPlugin2 {}

    #[test]
    pub fn can_get_plugins() {
        let mut engine = Engine::builder()
            .add_plugin(TestPlugin1)
            .add_plugin(TestPlugin2)
            .with_root(DummyWidget)
            .build();

        engine.update();

        assert!(
            engine.plugins().get::<TestPlugin1>().is_some(),
            "should have grabbed plugin 1"
        );

        assert!(
            engine.plugins().get::<TestPlugin2>().is_some(),
            "should have grabbed plugin 2"
        );
    }
}
