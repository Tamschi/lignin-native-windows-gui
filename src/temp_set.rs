use std::{
	collections::{BTreeSet, HashSet},
	ops::{Deref, DerefMut},
};

use lignin::{EventBinding, ThreadBound};

#[derive(Debug)]
pub struct TempEventBindingSet(BTreeSet<EventBinding<'static, ThreadBound>>);
impl TempEventBindingSet {
	pub fn new() -> Self {
		Self(BTreeSet::new())
	}

	pub fn temp<'a>(&mut self) -> &mut HashSet<EventBinding<'a, ThreadBound>> {
		unsafe {
			//SAFETY: The collection is cleared before each borrow, so no values can leak between them.
			// Note that clearing after each borrow instead would NOT be (particularly) safe without poisoning,
			// as globals in the Wasm instance could be re-referenced after an abort via re-entry from JS.
			self.0.clear();
			&mut *(&mut self.0 as *mut BTreeSet<EventBinding<'static, ThreadBound>>).cast()
		}
	}
}

pub struct TempEventBindingSetBorrow<'a, 'b>(&'a mut BTreeSet<EventBinding<'b, ThreadBound>>);
impl<'b> Deref for TempEventBindingSetBorrow<'_, 'b> {
	type Target = BTreeSet<EventBinding<'b, ThreadBound>>;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}
impl<'b> DerefMut for TempEventBindingSetBorrow<'_, 'b> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0
	}
}
