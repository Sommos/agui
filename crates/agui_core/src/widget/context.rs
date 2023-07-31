use crate::{
    element::{Element, ElementId},
    util::tree::Tree,
};

pub trait ContextWidget {
    fn get_elements(&self) -> &Tree<ElementId, Element>;

    fn get_element_id(&self) -> ElementId;
}
