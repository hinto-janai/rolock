//! Read Only Lock.
//!
//! This is a wrapper around [`Arc<RwLock<T>>`] that only implements [`RwLock::read()`] operations.
//!
//! Usage: Create a normal [`Arc<RwLock<T>>`] in `thread_1`, send a [`RoLock`] to `thread_2`:
//! ```
//! # use std::sync::*;
//! # use rolock::RoLock;
//! let rw = Arc::new(RwLock::new(0));     // Regular Arc<RwLock<T>>.
//! let ro = RoLock::new(&rw);             // Read Only Lock.
//!
//! assert!(*rw.read().unwrap() == 0);     // This can read...
//! *rw.write().unwrap() = 1;              // and write.
//!
//! std::thread::spawn(move|| {
//! 	assert!(*ro.read().unwrap() == 1); // This one can only read.
//! });
//! ```
//! - `thread_1` still has full read/write control
//! - `thread_2` can only [`RoLock::read()`]
//!
//! This type guarantees at compile time that you cannot write because the function doesn't even exist:
//! ```compile_fail
//! # use std::sync::{Arc,RwLock};
//! # use rolock::RoLock;
//! let rw = Arc::new(RwLock::new(0));
//! let ro = RoLock::new(&rw);
//!
//! ro.write();
//! ```
//! Since the inner field of [`RoLock`] (`self.0`) is private, you can't call [`RwLock::write`] directly either:
//! ```compile_fail
//! # use std::sync::{Arc,RwLock};
//! # use rolock::RoLock;
//! let rw = Arc::new(RwLock::new(0));
//! let ro = RoLock::new(&rw);
//!
//! ro.0.write();
//! ```
//!
//! Calling `.clone()` on `RoLock` is (relatively) cheap, as it just clones the inner [`Arc`].
//! ```rust
//! # use std::sync::Arc;
//! # use rolock::RoLock;
//! let (rw, ro) = RoLock::new_pair(0);
//!
//! // This is (relatively) cheap.
//! let clone = ro.clone();
//! ```

use std::sync::*;

/// Read Only Lock.
///
/// This is a wrapper around [`Arc<RwLock<T>>`] that only implements [`RwLock::read()`] operations.
#[derive(Debug)]
pub struct RoLock<T>(Arc<RwLock<T>>);

impl<T: std::fmt::Debug> RoLock<T> {
	#[inline(always)]
	/// Get an [`Arc`] to an existing [`Arc<RwLock<T>>`] but as a [`RoLock`].
	/// ```rust
	/// # use rolock::RoLock;
	/// # use std::sync::{Arc,RwLock};
	/// let rw = Arc::new(RwLock::new(true));
	///
	/// let ro = RoLock::new(&rw);
	///
	/// assert!(*rw.read().unwrap() == *ro.read().unwrap());
	/// ```
	pub fn new(value: &Arc<RwLock<T>>) -> Self {
		Self::from(value)
	}

	#[inline(always)]
	/// Creates a whole new [`Arc<RwLock<T>>`], returning it and an associated [`RoLock`].
	/// ```rust
	/// # use rolock::RoLock;
	/// let (rw, ro) = RoLock::new_pair(true);
	///
	/// assert!(*rw.read().unwrap() == *ro.read().unwrap());
	/// ```
	pub fn new_pair(value: T) -> (Arc<RwLock<T>>, Self) {
		let rw = Arc::new(RwLock::new(value));
		let ro = Self::from(&rw);
		(rw, ro)
	}

	#[inline(always)]
	/// Wraps a [`RwLock`] in an [`Arc`], returning it alongside an associated [`RoLock`].
	/// ```rust
	/// # use rolock::RoLock;
	/// # use std::sync::RwLock;
	/// let rw = RwLock::new(true);
	/// let (rw, ro) = RoLock::from_rw(rw);
	///
	/// assert!(*rw.read().unwrap() == *ro.read().unwrap());
	/// ```
	pub fn from_rw(value: RwLock<T>) -> (Arc<RwLock<T>>, Self) {
		let rw = Arc::new(value);
		let ro = Self::new(&rw);
		(rw, ro)
	}

	#[inline(always)]
	/// Calls [`RwLock::read`].
	pub fn read(&self) -> Result<RwLockReadGuard<'_, T>, PoisonError<RwLockReadGuard<'_, T>>> {
		self.0.read()
	}

	#[inline(always)]
	/// Calls [`RwLock::try_read`].
	pub fn try_read(&self) -> TryLockResult<RwLockReadGuard<'_, T>> {
		self.0.try_read()
	}

	#[inline(always)]
	/// Calls [`RwLock::is_poisoned`].
	pub fn is_poisoned(&self) -> bool {
		self.0.is_poisoned()
	}

	#[inline(always)]
	/// Gets the number of [`RoLock`]'s pointing to the same data.
	///
	/// [`RoLock::new_pair`] creates 2 [`Arc`]'s:
	/// ```rust
	/// # use rolock::RoLock;
	/// # use std::sync::Arc;
	/// let (rw, ro) = RoLock::new_pair(0);
	/// assert!(Arc::strong_count(&rw) == 2);
	/// assert!(ro.strong_count()      == 2);
	///
	/// drop(rw);
	/// assert!(ro.strong_count() == 1);
	/// ```
	///
	/// Calls [`Arc::strong_count`].
	pub fn strong_count(&self) -> usize {
		Arc::strong_count(&self.0)
	}

	#[inline(always)]
	/// Calls [`Arc::try_unwrap`] and [`RwLock::into_inner`] and returns the inner value:
	///
	/// We silently [`drop`] the `Arc<RwLock>` by using `_` here,
	/// so when we `into_inner()`, it should be the only one left:
	/// ```rust
	/// # use rolock::RoLock;
	/// # use std::sync::{Arc,RwLock};
	/// let (_, ro) = RoLock::new_pair(0);
	/// assert!(ro.into_inner().unwrap() == 0);
	/// ```
	///
	/// We _don't [`drop`] the `Arc<RwLock>` here, this should panic:
	/// ```rust,should_panic
	/// # use rolock::RoLock;
	/// # use std::sync::{Arc,RwLock};
	/// let (rw, ro) = RoLock::new_pair(0);
	/// assert!(ro.into_inner().unwrap() == 0);
	/// ```
	///
	/// # Errors
	/// You must ensure that:
	/// 1. There are no other [`RoLock`]'s
	/// 2. The inner [`RwLock`] is not poisoned
	///
	/// If [`Arc::try_unwrap`] fails (there are multiple [`RoLock`]'s), the [`RoLock`] will be returned.
	/// If [`RwLock::into_inner`] fails (poison error), an empty [`IntoInnerError::Poison`] will be returned.
	pub fn into_inner(self) -> Result<T, IntoInnerError<T>> {
		let rw = match Arc::try_unwrap(self.0) {
			Ok(rw) => rw,
			Err(e) => return Err(IntoInnerError::Multiple(RoLock(e))),
		};

		match RwLock::into_inner(rw) {
			Ok(inner) => Ok(inner),
			Err(_)    => return Err(IntoInnerError::Poison),
		}
	}

	#[inline(always)]
	/// Same as [`RoLock::into_inner`], but panics instead of erroring.
	///
	/// # Panics
	/// We silently [`drop`] the `Arc<RwLock>` by using `_` here,
	/// so when we `into_inner()`, it should be the only one left:
	/// ```rust
	/// # use rolock::RoLock;
	/// # use std::sync::{Arc,RwLock};
	/// let (_, ro) = RoLock::new_pair(0);
	/// assert!(ro.into_inner_unchecked() == 0);
	/// ```
	///
	/// We _don't [`drop`] the `Arc<RwLock>` here, this should panic:
	/// ```rust,should_panic
	/// # use rolock::RoLock;
	/// # use std::sync::{Arc,RwLock};
	/// let (rw, ro) = RoLock::new_pair(0);
	/// assert!(ro.into_inner_unchecked() == 0);
	/// ```
	///
	/// You must ensure that:
	/// 1. There are no other [`RoLock`]'s
	/// 2. The inner [`RwLock`] is not poisoned
	pub fn into_inner_unchecked(self) -> T {
		Arc::try_unwrap(self.0).unwrap().into_inner().unwrap()
	}
}

//---------------------------------------------------------------------------------------------------- Error
/// The error returned when [`RoLock::into_inner`] fails.
///
/// It either returns the [`RoLock`] or returns an empty `Poison` error,
/// indicating the inner [`RwLock`] is poisoned.
#[derive(Debug)]
pub enum IntoInnerError<T> {
	Multiple(RoLock<T>),
	Poison,
}

//---------------------------------------------------------------------------------------------------- Common Impls
impl<T> Clone for RoLock<T> {
	#[inline(always)]
	fn clone(&self) -> Self {
		Self(Arc::clone(&self.0))
	}
}

impl<T> From<&Arc<RwLock<T>>> for RoLock<T> {
	#[inline(always)]
	fn from(value: &Arc<RwLock<T>>) -> Self {
		Self(Arc::clone(value))
	}
}
