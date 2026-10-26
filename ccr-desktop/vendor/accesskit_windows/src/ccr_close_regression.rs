// CCR regression tests for an in-flight UIA provider retaining a dead HWND.
// No native window is created or shown by these tests.

use std::sync::Arc;

use accesskit::{ActionHandler, ActionRequest, Node, NodeId, Rect, Role, Tree, TreeId, TreeUpdate};
use windows::{Win32::UI::Accessibility::*, core::*};

use crate::{
    HWND,
    context::{ActionHandlerWrapper, Context},
    window_handle::WindowHandle,
};

struct NoActions;

impl ActionHandler for NoActions {
    fn do_action(&mut self, _: ActionRequest) {
        panic!("coordinate queries must not invoke actions");
    }
}

fn retained_context(bounds: bool) -> Arc<Context> {
    let mut input = Node::new(Role::TextInput);
    input.set_children([NodeId(1)]);
    let mut text = Node::new(Role::TextRun);
    text.set_value("x");
    text.set_character_lengths([1]);
    if bounds {
        input.set_bounds(Rect::new(0.0, 0.0, 100.0, 20.0));
        text.set_bounds(Rect::new(0.0, 0.0, 10.0, 20.0));
        text.set_character_positions([0.0]);
        text.set_character_widths([10.0]);
        text.set_text_direction(accesskit::TextDirection::LeftToRight);
    }
    let tree = accesskit_consumer::Tree::new(
        TreeUpdate {
            nodes: vec![(NodeId(0), input), (NodeId(1), text)],
            tree: Some(Tree::new(NodeId(0))),
            tree_id: TreeId::ROOT,
            focus: NodeId(0),
        },
        false,
    );
    Context::new(
        WindowHandle(HWND::default()),
        tree,
        Arc::new(ActionHandlerWrapper::new(NoActions)),
        false,
    )
}

fn unavailable<T>(result: Result<T>) {
    match result {
        Err(error) => assert_eq!(error.code().0 as u32, 0x80040201),
        Ok(_) => {
            panic!("a coordinate query against a dead HWND must return UIA_E_ELEMENTNOTAVAILABLE")
        }
    }
}

#[test]
fn ccr_invalid_hwnd_retained_context_provider_coordinates() {
    let context = retained_context(true);
    let root_id = context.read_tree().state().root_id();
    let node = context.get_or_create_platform_node(root_id);
    let fragment: IRawElementProviderFragment = node.to_interface();
    let root: IRawElementProviderFragmentRoot = node.to_interface();
    let text: ITextProvider = node.to_interface();
    // Keep the strong Context alive: Weak-upgrade failure would miss the race.
    unsafe {
        unavailable(fragment.BoundingRectangle());
        unavailable(root.ElementProviderFromPoint(1.0, 1.0));
        unavailable(text.RangeFromPoint(UiaPoint { x: 1.0, y: 1.0 }));
        let range = text.DocumentRange().unwrap();
        unavailable(range.GetBoundingRectangles());
    }
}

#[test]
fn ccr_invalid_hwnd_preserves_absent_bounds() {
    let context = retained_context(false);
    let root_id = context.read_tree().state().root_id();
    let node = context.get_or_create_platform_node(root_id);
    let fragment: IRawElementProviderFragment = node.to_interface();
    let text: ITextProvider = node.to_interface();
    unsafe {
        let rect = fragment.BoundingRectangle().unwrap();
        assert_eq!(
            (rect.left, rect.top, rect.width, rect.height),
            (0.0, 0.0, 0.0, 0.0)
        );
        let range = text.DocumentRange().unwrap();
        assert!(range.GetBoundingRectangles().unwrap().is_null());
    }
}
