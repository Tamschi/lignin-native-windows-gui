//! Required features: "a"

use lignin::{Attribute, Element, ElementCreationOptions, Node};
use lignin_native_windows_gui::LigninUi;
use lignin_schema::html::elements::a;
use native_windows_gui as nwg;
use nwg::{PartialUi, Window};

fn main() {
	nwg::init().unwrap();
	nwg::Font::set_global_family("Segoe UI").unwrap();

	let mut window = Window::default();
	Window::builder().build(&mut window).unwrap();

	let mut ui = LigninUi::new(|bump| Node::HtmlElement {
		dom_binding: None,
		element: bump.alloc_with(|| Element {
			name: a::TAG_NAME,
			creation_options: ElementCreationOptions::new(),
			attributes: bump.alloc_with(|| {
				[Attribute {
					name: "href",
					value: "https://example.com/",
				}]
			}),
			content: Node::Text {
				text: "This is a clickable link!",
				dom_binding: None,
			},
			event_bindings: &[],
		}),
	});

	LigninUi::build_partial(&mut ui, Some(&window)).unwrap();
	nwg::dispatch_thread_events();
}
