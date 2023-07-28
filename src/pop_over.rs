use floem::glazier::kurbo::Rect;
use floem::id::Id;
use floem::taffy;
use floem::view::{ChangeFlags, View};
use floem::ViewContext;
use floem::{context::*, Renderer};

pub struct PopOver<PV: View, CV: View> {
    id: Id,
    parent: PV,
    popover: CV,
}

pub fn pop_over<PV: View, CV: View>(
    parent: impl FnOnce() -> PV,
    popover: impl FnOnce() -> CV,
) -> PopOver<PV, CV> {
    let cx = ViewContext::get_current();
    let id = cx.new_id();
    let mut child_cx = cx;
    child_cx.id = id;
    ViewContext::save();
    ViewContext::set_current(child_cx);
    let parent = parent();
    let child = popover();
    ViewContext::restore();

    PopOver {
        id,
        parent,
        popover: child,
    }
}

impl<PV: View, CV: View> View for PopOver<PV, CV> {
    fn id(&self) -> Id {
        self.id
    }

    fn child(&mut self, id: Id) -> Option<&mut dyn View> {
        if self.popover.id() == id {
            Some(&mut self.popover)
        } else if self.parent.id() == id {
            Some(&mut self.parent)
        } else {
            None
        }
    }

    fn children(&mut self) -> Vec<&mut dyn View> {
        vec![&mut self.parent, &mut self.popover]
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "Container".into()
    }

    fn update(
        &mut self,
        _cx: &mut floem::context::UpdateCx,
        _state: Box<dyn std::any::Any>,
    ) -> floem::view::ChangeFlags {
        ChangeFlags::empty()
    }

    fn layout(&mut self, cx: &mut floem::context::LayoutCx) -> taffy::prelude::Node {
        cx.layout_node(self.id, true, |cx| {
            vec![self.parent.layout_main(cx), self.popover.layout_main(cx)]
        })
    }

    fn compute_layout(&mut self, cx: &mut floem::context::LayoutCx) -> Option<Rect> {
        Some(self.popover.compute_layout_main(cx))
    }

    fn event(
        &mut self,
        cx: &mut floem::context::EventCx,
        id_path: Option<&[Id]>,
        event: floem::event::Event,
    ) -> bool {
        false
    }

    fn paint(&mut self, cx: &mut PaintCx) {
        // cx.fill();
        self.child.paint_main(cx);
    }
}
