//! TODO_DOCS_DESCRIPTION
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Flignin-native-windows-gui)](https://iteration-square.schichler.dev/#narrow/stream/project.2Flignin-native-windows-gui)

#![doc(html_root_url = "https://docs.rs/lignin-native-windows-gui/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(missing_docs)]

use bumpalo::Bump;
use core::slice;
use lignin::{Node, ThreadBound};
use native_windows_gui::{self as nwg, ControlHandle, NwgError, PartialUi};
use nwg::Label;
use std::{any::Any, collections::HashMap, mem, ptr::addr_of_mut};
use tracing::{error, trace_span};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

mod rc_hash_map;
mod temp_set;

pub struct LigninUi<R: for<'a> FnMut(&'a Bump) -> lignin::Node<'a, ThreadBound>> {
	bump_a: Bump,
	bump_b: Bump,
	/// Fake lifetime, actually linked to `bump_a`.
	vdom_a: Node<'static, ThreadBound>,

	/// Horrible and faulty. Needs to be a proper tree mirroring `vdom_a`.
	controls: HashMap<usize, Box<dyn Any>>,
	render: R,
}

impl<R: for<'a> FnMut(&'a Bump) -> lignin::Node<'a, ThreadBound>> LigninUi<R> {
	pub fn new(render: R) -> Self {
		Self {
			bump_a: Bump::new(),
			bump_b: Bump::new(),
			vdom_a: Node::Multi(&[]),
			controls: HashMap::new(),
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

			diff_splice_node_list(
				&mut data.controls,
				slice::from_ref(&data.vdom_a),
				slice::from_ref(&vdom_b),
				parent,
				&mut 0,
				1000,
			)?;

			data.bump_a.reset();

			unsafe {
				data.vdom_a = mem::transmute(vdom_b);
				mem::swap(&mut data.bump_a, &mut data.bump_b);
			}

			Ok(())
		})
	}
}

fn diff_splice_node_list(
	controls: &mut HashMap<usize, Box<dyn Any>>,
	mut vdom_a: &[Node<'_, ThreadBound>],
	mut vdom_b: &[Node<'_, ThreadBound>],
	parent: ControlHandle,
	i: &mut usize,
	depth_limit: usize,
) -> Result<(), NwgError> {
	if depth_limit == 0 {
		error!("Depth limit exceeded.");
		return Err(NwgError::LayoutCreationError(
			"Depth limit exceeded.".to_string(),
		));
	}

	#[allow(clippy::never_loop)]
	while !vdom_a.is_empty() && !vdom_b.is_empty() {
		*i += match (vdom_a[0], vdom_b[0]) {
			(Node::Multi(a), Node::Multi(b)) => {
				let _span = trace_span!("Diffing multi", "a.len()" = a.len(), "b.len()" = b.len())
					.entered();
				// Skip `depth_limit` check one level down if there are no items at all.
				if !a.is_empty() || !b.is_empty() {
					diff_splice_node_list(controls, a, b, parent, i, depth_limit - 1)?;
				}
				0
			}
			(a, b) => todo!(),
		};

		vdom_a = &vdom_a[1..];
		vdom_b = &vdom_b[1..];
	}

	let mut vdom_a = vdom_a.iter();
	for removed_node in vdom_a.by_ref() {
		todo!()
	}

	for new_node in vdom_b {
		*i += match *new_node {
			Node::Comment {
				comment,
				dom_binding,
			} => 1,
			Node::HtmlElement {
				element,
				dom_binding,
			} => todo!(),
			Node::MathMlElement {
				element,
				dom_binding,
			} => todo!(),
			Node::SvgElement {
				element,
				dom_binding,
			} => todo!(),
			Node::Memoized { state_key, content } => todo!(),
			Node::Multi(_) => todo!(),
			Node::Keyed(_) => todo!(),

			Node::Text {
				text,
				dom_binding: _,
			} => {
				let mut label = Label::default();
				nwg::Label::builder()
					.text(text)
					.parent(&parent)
					.build(&mut label)?;
				controls.insert(new_node as *const _ as usize, Box::new(label));

				1
			}

			Node::RemnantSite(_) => todo!(),
		}
	}

	Ok(())
}
