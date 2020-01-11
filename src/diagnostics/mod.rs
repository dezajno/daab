
//!
//! # Extensive debugging analysis module.
//!
//! **Notice: This module is only available if the `diagnostics` feature has been activated**.
//!
//! This module contains the types used in debugging the [`ArtifactCache`].
//! The most important one is [`Doctor`] which conducts a diagnosis on a
//! `ArtifactCache` if constructed via [`ArtifactCache::new_with_doctor()`].
//!
//! `Doctor` has methods for various events happening in the `ArtifactCache`
//! receiving the relevant builder or artifact as argument
//! (as [`ArtifactHandle`] or [`BuilderHandle`] respectivly).
//! See the respective method of the `Doctor` for details.
//!
//! Additionally, to the generic `Doctor` trait, there are several pre-implemented
//! Doctors such as: [`VisgraphDoc`] or [`TextualDoc`].
//!
//![`ArtifactCache`]: ../struct.ArtifactCache.html
//![`Doctor`]: trait.Doctor.html
//![`ArtifactCache::new_with_doctor()`]: ../struct.ArtifactCache.html#method.new_with_doctor
//![`ArtifactHandle`]: struct.ArtifactHandle.html
//![`BuilderHandle`]: struct.BuilderHandle.html
//![`VisgraphDoc`]: struct.VisgraphDoc.html
//![`TextualDoc`]: struct.TextualDoc.html
//!


use std::any::Any;
use std::hash::Hash;
use std::hash::Hasher;
use std::rc::Rc;
use std::fmt::Debug;

use super::ArtifactPromise;
use super::Builder;


mod visgraph;

pub use visgraph::VisgraphDocOptions;
pub use visgraph::VisgraphDoc;

mod textual;

pub use textual::TextualDocOptions;
pub use textual::TextualDoc;



/// Debugger for the [`ArtifactCache`].
///
/// **Notice: This trait is only available if the `diagnostics` feature has been activated**.
///
/// The Doctor conducts diagnoses on the `ArtifactCache`, if it is passed
/// with [`ArtifactCache::new_with_doctor()`]. The `ArtifactCache` will
/// call the methods of this trait whenever the respective event happens.
/// It will be supplied with relevant object(s), such as `Builder`s and artifacts.
/// For details on each event see the respective method.
///
/// Each method as a default implementation to ease implementing specialized `Doctor`s which don't need all the events. Each default implementation just dose nothing, i.e. are no-ops.
///
///[`ArtifactCache`]: ../struct.ArtifactCache.html
///[`ArtifactCache::new_with_doctor()`]: ../struct.ArtifactCache.html#method.new_with_doctor
///
pub trait Doctor {
	/// One `Builder` resolves another `Builder`.
	///
	/// This methods means that `builder` appearently depends on `used`.
	///
	fn resolve(&mut self, _builder: &BuilderHandle, _used: &BuilderHandle) {
		// NOOP
	}
	
	/// One `Builder` builds its artifact.
	///
	/// This method is called each time `builder` is invoked to build
	/// its `artifact`. Notice, this function is only called when a fresh
	/// artifact is actually constructed, i.e. first time it is resolved
	/// or when it is resolved after a reset or invalidation.
	///
	fn build(&mut self, _builder: &BuilderHandle, _artifact: &ArtifactHandle) {
		// NOOP
	}
	
	/// The entire cache is cleared.
	///
	fn clear(&mut self) {
		// NOOP
	}
	
	/// The given `Builder` is invalidate.
	///
	/// This method is only called if invalidation is call directly with
	/// `builder` as its argument.
	///
	/// **Notice:** All dependants of `builder` are invalidated as well, but
	/// this function will not be called for the dependant invalidations. If
	/// deep invalidation tracking is desirable, the dependencies have to be
	/// tracked via the `resolve` call back.
	///
	/// **Notice:** This invalidation might result in clearing the entire cache,
	/// but `clear` will not be called in such a case.
	///
	fn invalidate(&mut self, _builder: &BuilderHandle) {
		// NOOP
	}
}


/// Encapsulates a generic artifact with some debugging information.
///
/// This struct encapsulates a artifact as `Rc<dyn Any>` which is fairly usless,
/// unless one wants to cast or test it against a concrete type.
/// Thus this struct also contains the strinified type name of that value
/// as well as the `Debug` string of the value.
/// Also notice, that different values can be differentiated by the allocation
/// pointer thus the implementation of `Hash` and `Eq`.
///
#[derive(Clone, Debug)]
pub struct ArtifactHandle {
	/// The actual artifact value.
	pub value: Rc<dyn Any>,
	
	/// The type name of the artifact as of `std::any::type_name`.
	pub type_name: &'static str,
	
	/// The value of the artifact as of `std::fmt::Debug`.
	pub dbg_text: String,
}

impl ArtifactHandle {
	/// Constructs a new artifact handle with the given value.
	///
	pub fn new<T: Any + Debug>(value: Rc<T>) -> Self {
		let dbg_text = format!("{:#?}", &value);
		
		ArtifactHandle {
			value,
			type_name: std::any::type_name::<T>(),
			dbg_text,
		}
	}
}

impl Hash for ArtifactHandle {
	fn hash<H: Hasher>(&self, state: &mut H) {
		(self.value.as_ref() as *const dyn Any).hash(state);
	}
}

impl PartialEq for ArtifactHandle {
	fn eq(&self, other: &Self) -> bool {
		(self.value.as_ref() as *const dyn Any)
			.eq(&(other.value.as_ref() as *const dyn Any))
	}
}

impl Eq for ArtifactHandle {
}


/// Encapsulates a generic builder with some debugging information.
///
/// This struct encapsulates a builder as `ArtifactPromise<dyn Any>` which is fairly usless,
/// unless one wants to cast or test it against a concrete type.
/// Thus this struct also contains the strinified type name of that value
/// as well as the `Debug` string of the value.
/// Also notice, that different builders can be differentiated by the allocation
/// pointer thus the implementation of `Hash` and `Eq`.
///
#[derive(Clone, Debug)]
pub struct BuilderHandle {
	/// The actual builder as promise.
	pub value: ArtifactPromise<dyn Any>,
	
	/// The type name of the builder as of `std::any::type_name`.
	pub type_name: &'static str,
	
	/// The value of the builder as of `std::fmt::Debug`.
	pub dbg_text: String,
}

impl BuilderHandle {
	/// Constructs a new builder handle with the given value.
	///
	pub fn new<T: Builder + Debug + 'static>(value: ArtifactPromise<T>) -> Self {
		let dbg_text = format!("{:#?}", &value.builder);
		
		BuilderHandle {
			value: value.into_any(),
			type_name: std::any::type_name::<T>(),
			dbg_text,
		}
	}
}

impl Hash for BuilderHandle {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.value.hash(state);
	}
}

impl PartialEq for BuilderHandle {
	fn eq(&self, other: &Self) -> bool {
		self.value.eq(&other.value)
	}
}

impl Eq for BuilderHandle {
}


/// Default no-op `Doctor`.
///
/// **Notice: This struct is only available if the `diagnostics` feature has been activated**.
///
/// A no-op implementation of the `Doctor` i.e. a `Doctor` that does nothing. It is used as default `Doctor`,
/// i.e. if no actual `Doctor` is specified.
///
pub struct NoopDoctor;

impl Doctor for NoopDoctor {
	// Use default impl
}

impl Default for NoopDoctor {
	fn default() -> Self {
		NoopDoctor
	}
}



