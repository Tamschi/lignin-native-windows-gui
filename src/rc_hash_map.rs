use core::borrow::Borrow;
use num_traits::{CheckedAdd, CheckedSub, One, Zero};
use std::collections::{btree_map::Entry, BTreeMap};

#[derive(Debug)]
pub struct RcBTreeMap<K, C, V>(BTreeMap<K, (C, V)>)
where
	K: Ord,
	C: CheckedAdd + CheckedSub + One + Zero;

impl<K, C, V> Default for RcBTreeMap<K, C, V>
where
	K: Ord,
	C: CheckedAdd + CheckedSub + One + Zero,
{
	fn default() -> Self {
		Self::new()
	}
}

impl<K, C, V> RcBTreeMap<K, C, V>
where
	K: Ord,
	C: CheckedAdd + CheckedSub + One + Zero,
{
	#[must_use]
	pub fn new() -> Self {
		Self(BTreeMap::new())
	}

	#[allow(clippy::shadow_unrelated)]
	pub fn increment_or_insert_with<F: FnOnce(&K) -> V>(
		&mut self,
		k: K,
		v: F,
	) -> Result<&mut V, CountSaturatedError> {
		match self.0.entry(k) {
			Entry::Occupied(occupied) => {
				let (c, v) = occupied.into_mut();
				*c = c.checked_add(&C::one()).ok_or(CountSaturatedError)?;
				Ok(v)
			}
			Entry::Vacant(vacant) => {
				let v = v(vacant.key());
				let (_, v) = vacant.insert((C::one(), v));
				Ok(v)
			}
		}
	}

	pub fn weak_decrement<Q: ?Sized>(
		&mut self,
		k: &Q,
	) -> Result<Option<&mut V>, CountSaturatedError>
	where
		K: Borrow<Q>,
		Q: Ord,
	{
		match self.0.get_mut(k) {
			Some((c, v)) => {
				*c = c.checked_sub(&C::one()).ok_or(CountSaturatedError)?;
				Ok(Some(v))
			}
			None => Ok(None),
		}
	}

	#[cfg(any())]
	pub fn drain_weak(&mut self) -> DrainWeak<'_, K, C, V> {
		DrainWeak(self.0.drain_filter(DrainWeak::weak_filter))
	}

	#[allow(clippy::inline_always)] // Trivial proxy.
	#[inline(always)]
	pub fn len(&self) -> usize {
		self.0.len()
	}
}

#[cfg(any())]
#[allow(clippy::type_complexity)]
pub struct DrainWeak<'a, K, C, V>(DrainFilter<'a, K, (C, V), fn(&K, &mut (C, V)) -> bool>);

#[cfg(any())]
impl<'a, K, C, V> DrainWeak<'a, K, C, V>
where
	C: Zero,
{
	fn weak_filter(_: &K, (c, _): &mut (C, V)) -> bool {
		c.is_zero()
	}
}

#[cfg(any())]
impl<'a, K, C, V> Iterator for DrainWeak<'a, K, C, V> {
	type Item = (K, V);

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(k, (_, v))| (k, v))
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}
}

#[derive(Debug)]
pub struct CountSaturatedError;
