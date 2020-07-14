
//!
//! Module for canning values.
//!
//! Canning means wrapping values in some package type, which is better for
//! storing. Thus these Can types contain some `dyn Any` value to allow casting
//! various values into cans.
//! In order to keep them more usable, a Can can be downcasted back to some `T`.
//!
//! This module also contains a notion for Bins which are 'open' Cans. For
//! instance an `Rc<dyn Any>` as one kind of Can, and its respective Bin is
//! `Rc<T>` for every `T`.
//!

use std::ops::Deref;
use std::fmt::Debug;
use std::any::Any;



/// Represents an opaque wrapper for `dyn Any`.
///
/// This type reperesents a wrapper for a `dyn Any`. It is basis for the `Can`
/// type which allows to be downcasted.
///
/// See `Can`.
///
pub trait CanBase: Sized {
	/// Returns the pointer to the inner value.
	///
	fn as_ptr(&self) -> *const dyn Any;
}

/// Represents an opaque wrapper for `dyn Any` which can be casted to `T`.
///
/// Since `dyn Any` can't be stored, a `Can` encapsules a `dyn Any` while
/// allowing it to be casted to specific wrapper `Bin` for `T`.
///
/// A good example for a `Can` is `Rc<dyn Any>`. Which for `T` can be casted
/// to a `Rc<T>` which would be the `Bin` type.
///
pub trait Can<T: ?Sized>: CanBase {
	/// A specific wrapper for `T` which can be casted from `Self`.
	///
	type Bin: Debug;
}

pub trait CanOwned<T: ?Sized>: Can<T> {

	/// Creates Self form a `Bin`.
	///
	/// This is a upcast and can not fail.
	fn from_bin(b: Self::Bin) -> Self;

	/// Tries to downcast the opaque `Can` to an specific `Bin`.
	///
	/// Because `Can`s are supposed to be alike `Any` allowing various `T`s to
	/// be casted to the same `Can`, this operation inherently may fail.
	///
	fn downcast_can(self) -> Option<Self::Bin>;

	/// Gets the pointer to
	fn bin_as_ptr(b: &Self::Bin) -> *const dyn Any;
}

/// Can that has a weak representation.
///
/// In the context of reference counting, a weak representation is supposed to
/// only allow access if there is at least one strong representation. It is a
/// good representation for caching, since it can be used to determine whether
/// there is any active user left (which has to have a strong representation).
///
/// Again the `Rc` type is a good example here, it is the `CanStrong` here and
/// the `std::rc::Weak` is the `CanWeak` in this regards.
///
pub trait CanStrong: CanBase {
	/// The weak representation for this type.
	type CanWeak: Debug;

	/// Allows to obtain a weak value for this can type.
	fn downgrade(&self) -> Self::CanWeak;

	/// Tries to upgrade a weak to a strong value, if there was any other
	/// strong value left.
	fn upgrade_from_weak(weak: &Self::CanWeak) -> Option<Self>;
}

/// Transparent variant of `Can`.
///
/// It allows additional to `Can` to get `T` from `Bin` and directly downcasting
/// this `Can` to `T`.
///
pub trait CanRef<T: ?Sized>: Can<T> {

	/// Tries to downcast the opaque `Can` to an specific `T`, by passing the
	/// `Bin` and cloning.
	///
	fn downcast_can_ref(&self) -> Option<&T>;

}

/// Mutable transparent variant of `Can`.
///
/// It allows additional to `Can` to get `T` from `Bin` and directly downcasting
/// this `Can` to `T`.
///
pub trait CanRefMut<T: ?Sized>: Can<T> {
	/// Tries to downcast the opaque `Can` to an specific `T`, by passing the
	/// `Bin` and cloning.
	///
	fn downcast_can_mut(&mut self) -> Option<&mut T>;

}

/// Sized variant of `Can`.
///
pub trait CanSized<T>: CanOwned<T> {
	/// Create a `Bin` for `T`.
	///
	fn into_bin(t: T) -> Self::Bin;

	/// Create `Self` directly from `T`.
	fn from_inner(t: T) -> Self {
		Self::from_bin(Self::into_bin(t))
	}
}


use std::rc::Rc;
use std::rc::Weak as WeakRc;

impl CanBase for Rc<dyn Any> {
	fn as_ptr(&self) -> *const dyn Any {
		self.deref()
	}
}

impl CanStrong for Rc<dyn Any> {
	type CanWeak = WeakRc<dyn Any>;

	fn downgrade(&self) -> Self::CanWeak {
		Rc::downgrade(self)
	}

	fn upgrade_from_weak(weak: &Self::CanWeak) -> Option<Self> {
		weak.upgrade()
	}
}

impl<T: ?Sized + Debug + 'static> Can<T> for Rc<dyn Any> {
	type Bin = Rc<T>;
}

impl<T: Debug + 'static> CanOwned<T> for Rc<dyn Any> {
	fn downcast_can(self) -> Option<Self::Bin> {
		self.downcast().ok()
	}
	fn from_bin(b: Self::Bin) -> Self {
		b
	}
	fn bin_as_ptr(b: &Self::Bin) -> *const dyn Any {
		b.deref()
	}
}

impl<T: Debug + 'static> CanRef<T> for Rc<dyn Any> {
	fn downcast_can_ref(&self) -> Option<&T> {
		self.downcast_ref()
	}
}

impl<T: Debug + 'static> CanSized<T> for Rc<dyn Any> {
	fn into_bin(t: T) -> Self::Bin {
		Rc::new(t)
	}
}


impl CanBase for Box<dyn Any> {
	fn as_ptr(&self) -> *const dyn Any {
		self.deref()
	}
}

impl<T: ?Sized + Debug + 'static> Can<T> for Box<dyn Any> {
	type Bin = Box<T>;
}

impl<T: Debug + 'static> CanOwned<T> for Box<dyn Any> {
	fn downcast_can(self) -> Option<Self::Bin> {
		self.downcast().ok()
		//	.map(|r: &T| Box::new(r.clone()))
	}
	fn from_bin(b: Self::Bin) -> Self {
		b
	}
	fn bin_as_ptr(b: &Self::Bin) -> *const dyn Any {
		b.deref()
	}
}

impl<T: Debug + 'static> CanRef<T> for Box<dyn Any> {
	fn downcast_can_ref(&self) -> Option<&T> {
		self.downcast_ref()
	}
}

impl<T: Debug + 'static> CanRefMut<T> for Box<dyn Any> {
	fn downcast_can_mut(&mut self) -> Option<&mut T> {
		self.downcast_mut()
	}
}

impl<T: Debug + 'static> CanSized<T> for Box<dyn Any> {
	fn into_bin(t: T) -> Self::Bin {
		Box::new(t)
	}
}


// TODO: impl for AP, Arc, maybe T/Box

use std::sync::Arc;
use std::sync::Weak as WeakArc;

impl CanBase for Arc<dyn Any + Send + Sync> {
	fn as_ptr(&self) -> *const dyn Any {
		self.deref()
	}
}

impl CanStrong for Arc<dyn Any + Send + Sync> {
	type CanWeak = WeakArc<dyn Any + Send + Sync>;

	fn downgrade(&self) -> Self::CanWeak {
		Arc::downgrade(self)
	}

	fn upgrade_from_weak(weak: &Self::CanWeak) -> Option<Self> {
		weak.upgrade()
	}
}

impl<T: Debug + Send + Sync + 'static> Can<T> for Arc<dyn Any + Send + Sync> {
	type Bin = Arc<T>;
}

impl<T: Debug + Send + Sync + 'static> CanOwned<T> for Arc<dyn Any + Send + Sync> {
	fn downcast_can(self) -> Option<Self::Bin> {
		self.downcast().ok()
	}
	fn from_bin(b: Self::Bin) -> Self {
		b
	}
	fn bin_as_ptr(b: &Self::Bin) -> *const dyn Any {
		b.deref()
	}
}

impl<T: Debug + Send + Sync + 'static> CanRef<T> for Arc<dyn Any + Send + Sync> {
	fn downcast_can_ref(&self) -> Option<&T> {
		self.downcast_ref()
	}
}

impl<T: Debug + Send + Sync + 'static> CanSized<T> for Arc<dyn Any + Send + Sync> {
	fn into_bin(t: T) -> Self::Bin {
		Arc::new(t)
	}
}



use crate::ArtifactPromise as Ap;
use crate::BuilderEntry;

impl<BCan: CanBase + 'static> CanBase for BuilderEntry<BCan> {
	fn as_ptr(&self) -> *const dyn Any {
		self.deref()
	}
}

impl<BCan: 'static, B: 'static> Can<B> for BuilderEntry<BCan>
		where BCan: Can<B> {

	type Bin = Ap<B, BCan>;
}

impl<BCan: 'static, B: 'static> CanOwned<B> for BuilderEntry<BCan>
		where BCan: CanOwned<B> + Clone {

	fn downcast_can(self) -> Option<Self::Bin> {
		self.builder.clone().downcast_can().map( |bin| {
			Ap {
				builder: bin,
				builder_canned: self.builder,
				_dummy: (),
			}
		})
	}
	fn from_bin(b: Self::Bin) -> Self {
		BuilderEntry::new(b)
	}
	fn bin_as_ptr(b: &Self::Bin) -> *const dyn Any {
		b.deref()
	}
}

impl<BCan: 'static, B: 'static> CanSized<B> for BuilderEntry<BCan>
		where BCan: CanSized<B> + Clone, BCan::Bin: Clone {
	fn into_bin(t: B) -> Self::Bin {
		Ap::new(t)
	}
}



