//! TODO_DOCS_DESCRIPTION
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Flignin-native-windows-gui)](https://iteration-square.schichler.dev/#narrow/stream/project.2Flignin-native-windows-gui)

#![doc(html_root_url = "https://docs.rs/lignin-native-windows-gui/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(missing_docs)]

use bumpalo::Bump;
use lignin::{CallbackRef, Element, Node, ReorderableFragment, ThreadBound};
use lignin_schema::html::attributes::href;
use native_windows_gui::{self as nwg, ControlHandle, NwgError, PartialUi};
use nwg::{
	bind_event_handler, unbind_event_handler, CharFormat, Event, EventHandler, Label,
	MousePressEvent, RichLabel, Tooltip, UnderlineType,
};
use std::{any::Any, collections::HashMap, convert::TryInto, fmt::Write, mem, process};
use tap::Pipe;
use tracing::{error, trace_span};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

pub struct LigninUi<R: for<'a> FnMut(&'a Bump) -> lignin::Node<'a, ThreadBound>> {
	bump_a: Bump,
	bump_b: Bump,
	/// Fake lifetime, actually linked to `bump_a`.
	vdom_a: Node<'static, ThreadBound>,
	/// Mirrors `vdom_a` while at rest.
	controls: Box<dyn Any>,
	render: R,
}

impl<R: for<'a> FnMut(&'a Bump) -> lignin::Node<'a, ThreadBound>> LigninUi<R> {
	pub fn new(render: R) -> Self {
		Self {
			bump_a: Bump::new(),
			bump_b: Bump::new(),
			vdom_a: Node::Multi(&[]),
			controls: Box::new(Vec::<Box<dyn Any>>::new()),
			render,
		}
	}
}

impl<R: for<'a> FnMut(&'a Bump) -> lignin::Node<'a, ThreadBound>> PartialUi for LigninUi<R> {
	fn build_partial<W: Into<ControlHandle>>(
		data: &mut Self,
		parent: Option<W>,
	) -> Result<(), NwgError> {
		parent.map_or(Ok(()), |parent| {
			let parent = parent.into();

			let vdom_b = (data.render)(&data.bump_b);

			diff_splice_node(&mut data.controls, &data.vdom_a, &vdom_b, &parent, 1000)?;

			data.bump_a.reset();

			unsafe {
				data.vdom_a = mem::transmute(vdom_b);
				mem::swap(&mut data.bump_a, &mut data.bump_b);
			}

			Ok(())
		})
	}
}

fn diff_splice_node(
	controls: &mut Box<dyn Any>,
	vdom_a: &Node<'_, ThreadBound>,
	vdom_b: &Node<'_, ThreadBound>,
	parent: &ControlHandle,
	depth_limit: usize,
) -> Result<(), NwgError> {
	if depth_limit == 0 {
		error!("Depth limit exceeded.");
		return Err(NwgError::LayoutCreationError(
			"Depth limit exceeded.".to_string(),
		));
	}

	match (vdom_a, vdom_b) {
		(Node::Comment { .. }, Node::Comment { .. }) => (),
		(
			Node::HtmlElement {
				element: e_a,
				dom_binding: db_a,
			},
			Node::HtmlElement {
				element: e_b,
				dom_binding: db_b,
			},
		) => todo!(),
		(
			Node::MathMlElement {
				element: e_a,
				dom_binding: db_a,
			},
			Node::MathMlElement {
				element: e_b,
				dom_binding: db_b,
			},
		) => todo!(),
		(
			Node::SvgElement {
				element: e_a,
				dom_binding: db_a,
			},
			Node::SvgElement {
				element: e_b,
				dom_binding: db_b,
			},
		) => todo!(),

		(
			Node::Memoized {
				state_key: sk_a,
				content: c_a,
			},
			Node::Memoized {
				state_key: sk_b,
				content: c_b,
			},
		) => {
			if sk_a != sk_b {
				diff_splice_node(controls, c_a, c_b, parent, depth_limit - 1)?
			}
		}

		(Node::Multi(_), Node::Multi(_)) => todo!(),
		(Node::Keyed(_), Node::Keyed(_)) => todo!(),
		(
			Node::Text {
				text: t_a,
				dom_binding: db_a,
			},
			Node::Text {
				text: t_b,
				dom_binding: db_b,
			},
		) => todo!(),
		(Node::RemnantSite(_), Node::RemnantSite(_)) => todo!(),

		(_a, b) => *controls = insert_node(b, parent, depth_limit)?,
	}

	Ok(())
}

fn insert_node(
	vdom_b: &Node<'_, ThreadBound>,
	parent: &ControlHandle,
	depth_limit: usize,
) -> Result<Box<dyn Any>, NwgError> {
	if depth_limit == 0 {
		error!("Depth limit exceeded.");
		return Err(NwgError::LayoutCreationError(
			"Depth limit exceeded.".to_string(),
		));
	}

	match *vdom_b {
		Node::Comment { .. } => Box::new(()) as Box<dyn Any>,
		Node::HtmlElement {
			element,
			dom_binding: _, //TODO
		} => insert_html_element(element, parent, depth_limit)?,
		Node::MathMlElement {
			element,
			dom_binding,
		} => todo!(),
		Node::SvgElement {
			element,
			dom_binding,
		} => todo!(),
		Node::Memoized { state_key, content } => insert_node(content, parent, depth_limit - 1)?,
		Node::Multi(multi) => {
			let mut controls = Vec::<Box<dyn Any>>::with_capacity(multi.len());
			for node in multi {
				controls.push(insert_node(node, parent, depth_limit - 1)?)
			}
			Box::new(controls)
		}
		Node::Keyed(_) => todo!(),

		Node::Text { text, dom_binding } => {
			let mut label = Label::default();
			nwg::Label::builder()
				.text(text)
				.parent(parent)
				.build(&mut label)?;
			Box::new(label)
		}

		Node::RemnantSite(_) => todo!(),
	}
	.pipe(Ok)
}

fn insert_html_element(
	element: &Element<ThreadBound>,
	parent: &ControlHandle,
	depth_limit: usize,
) -> Result<Box<dyn Any>, NwgError> {
	let Element {
		name,
		creation_options,
		attributes,
		ref content,
		event_bindings,
	} = *element;

	match name.to_ascii_uppercase().as_str() {
		lignin_schema::html::elements::a::TAG_NAME => {
			let rtf = collect_text(content, depth_limit)?;
			// let mut rtf = rtf.replace('\\', "\\\\");
			// rtf.insert_str(0, "{\\colortbl\\red0\\green0\\blue255}\\fs100\\u");

			let mut label = RichLabel::default();
			RichLabel::builder()
				.text(&rtf)
				.parent(parent)
				.build(&mut label)?;

			label.set_char_format(
				0..rtf.len().try_into().unwrap(),
				&CharFormat {
					height: Some(250),
					text_color: Some([0, 0, 255]),
					underline_type: Some(UnderlineType::Solid),
					..CharFormat::default()
				},
			);

			let mut handlers = vec![];
			for attribute in attributes {
				match attribute.name {
					<dyn href>::NAME => handlers.push((
						bind_event_handler(&label.handle, parent, {
							let target = attribute.value.to_string();
							move |event, _, _| {
								//TODO: This doesn't behave quite as expected yet.
								if let Event::OnMousePress(event) = event {
									if event == MousePressEvent::MousePressLeftUp {
										webbrowser::open(&target)
											.unwrap_or_else(|error| error!("{}", error))
									}
								}
							}
						})
						.pipe(EventHandlerHandle),
						{
							let mut tooltip = Tooltip::default();
							Tooltip::builder()
								.register(&label, attribute.value)
								.build(&mut tooltip)?;
							tooltip
						},
					)),
					name => todo!("<a> attribute with name {:?}", name),
				}
			}

			Box::new((handlers, label)) as Box<dyn Any>
		}

		name => todo!("Insert element with name {:?}", name),
	}
	.pipe(Ok)
}

fn collect_text(content: &Node<ThreadBound>, depth_limit: usize) -> Result<String, NwgError> {
	fn collect_text_inner(
		content: &Node<ThreadBound>,
		w: &mut String,
		depth_limit: usize,
	) -> Result<(), NwgError> {
		if depth_limit == 0 {
			error!("Depth limit exceeded.");
			return Err(NwgError::LayoutCreationError(
				"Depth limit exceeded.".to_string(),
			));
		}

		match *content {
			Node::Comment { .. } => (),
			Node::HtmlElement { element, .. }
			| Node::MathMlElement { element, .. }
			| Node::SvgElement { element, .. } => write!(w, "<{} …>", element.name).unwrap(),
			Node::Memoized { content, .. } => collect_text_inner(content, w, depth_limit - 1)?,
			Node::Multi(content) => {
				for content in content {
					collect_text_inner(content, w, depth_limit - 1)?
				}
			}
			Node::Keyed(reorderable_fragments) => {
				for ReorderableFragment { content, .. } in reorderable_fragments {
					collect_text_inner(content, w, depth_limit - 1)?
				}
			}
			Node::Text { text, .. } => w.write_str(text).unwrap(),
			Node::RemnantSite(_) => todo!(),
		}

		Ok(())
	}

	let mut text = String::new();
	collect_text_inner(content, &mut text, depth_limit)?;
	Ok(text)
}

fn diff_splice_node_list(
	controls: &mut Vec<Box<dyn Any>>,
	mut vdom_a: &[Node<'_, ThreadBound>],
	mut vdom_b: &[Node<'_, ThreadBound>],
	parent: &ControlHandle,
	depth_limit: usize,
) -> Result<(), NwgError> {
	if depth_limit == 0 {
		error!("Depth limit exceeded.");
		return Err(NwgError::LayoutCreationError(
			"Depth limit exceeded.".to_string(),
		));
	}

	let mut i = 0;

	#[allow(clippy::never_loop)]
	while !vdom_a.is_empty() && !vdom_b.is_empty() {
		match (vdom_a[0], vdom_b[0]) {
			(Node::Multi(vdom_a), Node::Multi(vdom_b)) => {
				let _span = trace_span!(
					"Diffing multi",
					"a.len()" = vdom_a.len(),
					"b.len()" = vdom_b.len()
				)
				.entered();

				// Skip `depth_limit` check one level down if there are no items at all.
				if !vdom_a.is_empty() || !vdom_b.is_empty() {
					let controls = controls[i].downcast_mut::<Vec<Box<dyn Any>>>().unwrap();
					diff_splice_node_list(controls, vdom_a, vdom_b, parent, depth_limit - 1)?;
				}
			}
			(ref vdom_a, ref vdom_b) => {
				diff_splice_node(&mut controls[i], vdom_a, vdom_b, parent, depth_limit)?
			}
		};

		i += 1;
		vdom_a = &vdom_a[1..];
		vdom_b = &vdom_b[1..];
	}

	controls.truncate(i);

	for new_node in vdom_b {
		controls.push(insert_node(new_node, parent, depth_limit)?)
	}

	Ok(())
}

struct EventHandlerHandle(EventHandler);
impl Drop for EventHandlerHandle {
	fn drop(&mut self) {
		unbind_event_handler(&self.0)
	}
}
