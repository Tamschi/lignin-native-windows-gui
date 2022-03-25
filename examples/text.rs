use lignin::Node;
use lignin_native_windows_gui::LigninUi;
use native_windows_gui as nwg;
use nwg::{PartialUi, Window};

fn main() {
	nwg::init().unwrap();
	nwg::Font::set_global_family("Segoe UI").unwrap();

	let mut window = Window::default();
	Window::builder().build(&mut window).unwrap();

	let mut ui = LigninUi::new(|bump| {
		Node::Multi(&*bump.alloc([Node::Text {
			text: "Hello Windows!",
			dom_binding: None,
		}]))
	});

	LigninUi::build_partial(&mut ui, Some(&window)).unwrap();
	nwg::dispatch_thread_events();
}
