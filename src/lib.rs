
//!
//! DAG Aware Artifact Builder
//! ==========================
//!
//! Rust crate for managing the building of artifacts by builders which are
//! connected in a directed acyclic graph (DAG) like manner.
//!
//! This crate provides essentially a cache which keeps artifacts of builders in
//! order to prevent the same builder to produce multiple equal artifacts.
//! This could be useful if the builders use consumable resources to create their
//! artifacts, the building is a heavyweight procedure, or a given DAG dependency
//! structure among the builders shall be properly preserved among their
//! artifacts.
//!
//! The basic principal on which this crate is build, suggests two levels of
//! abstraction, the builder level and the artifact level. Each builder type has
//! one specific artifact type. The builders are represented by any struct,
//! which implements the [`Builder`] trait, which in turn has an associate type
//! that specifies the artifact type.
//!
//! `Builder`s are supposed to be wrapped in [`ArtifactPromise`]s, which prevents
//! to call its `Builder::build()` method directly. In other respects, the
//! `ArtifactPromise` acts a lot like an `Rc` and thus allows to share one
//! instance among several dependants.
//! This `Rc`-like structure creates naturally a DAG.
//!
//! For building a `Builder`, its `Builder::build()` method is provided with a
//! [`ArtifactResolver`] that allows to resolve depending `ArtifactPromise`s into
//! their respective artifacts, which is, in order to form a DAG, wrapped
//! behind a `Rc`.
//!
//! As entry point serves the [`ArtifactCache`], which allows to resolve any
//! `ArtifactPromise` to its artifact outside of a `Builder`. The
//! `ArtifactCache` is essentially a cache. It can be used to translate any
//! number of `ArtifactPromise`s, sharing their common dependencies.
//! Consequently, resolving the same `ArtifactPromise` using the same
//! `ArtifactCache` results in the same `Rc`ed artifact.
//!
//! When artifacts shall be explicitly recreated, e.g. to form a second
//! independent artifact DAG, `ArtifactCache` has a `clear()` method
//! to reset the cache.
//! Additionally, `ArtifactCache` has an `invalidate()` method to remove a single
//! builder artifact including its dependants (i.e. those artifacts which had
//! used the invalidated one).
//!
//![`Builder`]: trait.Builder.html
//![`ArtifactPromise`]: struct.ArtifactPromise.html
//![`ArtifactResolver`]: struct.ArtifactResolver.html
//![`ArtifactCache`]: struct.ArtifactCache.html
//!
//!
//!
//! ## Example
//!
//! ```rust
//! use std::rc::Rc;
//! use daab::*;
//!
//! // Simple artifact
//! #[derive(Debug)]
//! struct Leaf {
//!     //...
//! }
//! 
//! // Simple builder
//! #[derive(Debug)]
//! struct BuilderLeaf {
//!     // ...
//! }
//! impl BuilderLeaf {
//!     pub fn new() -> Self {
//!         Self {
//!             // ...
//!         }
//!     }
//! }
//! impl Builder for BuilderLeaf {
//!     type Artifact = Leaf;
//!     
//!     fn build(&self, _cache: &mut ArtifactResolver) -> Self::Artifact {
//!         Leaf{
//!             // ...
//!         }
//!     }
//! }
//! 
//! // Composed artifact, linking to a Leaf
//! #[derive(Debug)]
//! struct Node {
//!     leaf: Rc<Leaf>, // Dependency artifact
//!     // ...
//! }
//! 
//! // Composed builder, depending on BuilderLeaf
//! #[derive(Debug)]
//! struct BuilderNode {
//!     builder_leaf: ArtifactPromise<BuilderLeaf>, // Dependency builder
//!     // ...
//! }
//! impl BuilderNode {
//!     pub fn new(builder_leaf: ArtifactPromise<BuilderLeaf>) -> Self {
//!         Self {
//!             builder_leaf,
//!             // ...
//!         }
//!     }
//! }
//! impl Builder for BuilderNode {
//!     type Artifact = Node;
//!     
//!     fn build(&self, cache: &mut ArtifactResolver) -> Self::Artifact {
//!         // Resolve ArtifactPromise to its artifact
//!         let leaf = cache.resolve(&self.builder_leaf);
//!         
//!         Node {
//!             leaf,
//!             // ...
//!         }
//!     }
//! }
//! 
//! // The cache to storing already created artifacts
//! let mut cache = ArtifactCache::new();
//!
//! // Constructing builders
//! let leaf_builder = ArtifactPromise::new(BuilderLeaf::new());
//!
//! let node_builder_1 = ArtifactPromise::new(BuilderNode::new(leaf_builder.clone()));
//! let node_builder_2: ArtifactPromise<_> = BuilderNode::new(leaf_builder.clone()).into();
//!
//! // Using the cache to access the artifacts from the builders
//!
//! // The same builder results in same artifact
//! assert!(Rc::ptr_eq(&cache.get(&node_builder_1), &cache.get(&node_builder_1)));
//!
//! // Different builders result in different artifacts
//! assert!( ! Rc::ptr_eq(&cache.get(&node_builder_1), &cache.get(&node_builder_2)));
//!
//! // Different artifacts may link the same dependent artifact
//! assert!(Rc::ptr_eq(&cache.get(&node_builder_1).leaf, &cache.get(&node_builder_2).leaf));
//! ```
//!
//! ## Debugging
//!
//! `daab` comes with extensive debugging gear. However, in order to
//! keep the production impact as low as possible, the debugging facilityies
//! are capsuled behind the `diagnostics` feature.
//!
//! Of course, the debugging feature is for the user of this crate to
//! debug their graphs. Therefore, it is rather modeled as a
//! diagnostics feature (hence the name). Consequently, the diagnosis
//! is carried out by a [`Doctor`], which is a trait receiving various
//! internal events in order to record them, print them, or otherwise help treating the bug.
//!
//! Care has been taken to keep the `diagnostics` feature broadly applicable as
//! well as keeping the non-`diagnostics` API compatible with the
//! `diagnostics`-API, meaning that a project using not using the
//! `diagnostics` feature can be easily converted to using the
//! `diagnostics`, usually by just replacing `ArtifactCache::new()`
//! by `ArtifactCache::new_with_doctor()`.
//! For this reason the `ArtifactCache` is generic to its doctor, which is
//! important on its creation. The rest of the time the `ArtifactCache`
//! uses `dyn Doctor` as its (fixed) generic argument.
//! To ease conversion between them, all creatable `ArtifactCache`s
//! (i.e. not `ArtifactCache<dyn Doctor>`) implement `DerefMut` to
//! `ArtifactCache<dyn Doctor>` which has all the methods implemented.
//!
//![`Doctor`]: diagnostics/trait.Doctor.html
//!
//!
//! ## Features
//!
//! This crate offers the following features:
//!
//! - **`diagnostics`** enables elaborate graph and cache interaction debugging.
//!   It adds the `new_with_doctor()` function to the `ArtifactCache` and adds
//!   the `diagnostics` module with the `Doctor` trait definition and some
//!   default `Doctor`s.
//!
//! - **`tynm`** enable the optional dependency on the [`tynm`] crate which adds
//!   functionality to abbreviate type names, which are used by some default
//!   `Doctor`s, hence it is only useful in connection with the `diagnostics`
//!   feature.
//!
//![`tynm`]: https://crates.io/crates/tynm
//!

// prevents compilation with broken Deref impl causing nasty stack overflows.
#![deny(unconditional_recursion)]

#![warn(missing_docs)]


use std::rc::Rc;
use std::collections::HashMap;
use std::collections::HashSet;
use std::any::Any;
use std::hash::Hash;
use std::hash::Hasher;
use std::fmt::Debug;
use std::borrow::Borrow;

#[cfg(feature = "diagnostics")]
use std::ops::Deref;

#[cfg(feature = "diagnostics")]
use std::ops::DerefMut;


#[cfg(feature = "diagnostics")]
pub mod diagnostics;


#[cfg(feature = "diagnostics")]
use diagnostics::Doctor;

#[cfg(feature = "diagnostics")]
use diagnostics::ArtifactHandle;

#[cfg(feature = "diagnostics")]
use diagnostics::BuilderHandle;

#[cfg(feature = "diagnostics")]
use diagnostics::NoopDoctor as DefDoctor;


/// Represents a builder for an artifact.
///
/// Each builder is supposed to contain all direct dependencies possibly other
/// builders.
/// In the `build()` function, the builder can access `cache` in order to
/// resolve depending builders (as `ArtifactPromise`) in order to create their
/// artifact.
///
pub trait Builder: Debug {
	/// The artifact type as produced by this builder.
	///
	type Artifact : Debug;
	
	/// Produces an artifact using the given `ArtifactResolver` for resolving
	/// dependencies.
	///
	fn build(&self, cache: &mut ArtifactResolver) -> Self::Artifact;
}



/// Encapsulates a builder as promise for its artifact from the `ArtifactCache`.
///
/// This struct is essentially a wrapper around `Rc<B>`, but it provides a
/// `Hash` and `Eq` implementation based no the identity of the `Rc`s inner value.
///
/// All clones of an `ArtifactPromise` are considered identical.
///
/// An `ArtifactPromise` can be either resolved using the `ArtifactCache::get()`
/// or `ArtifactResolver::resolve()`.
///
#[derive(Debug)]
pub struct ArtifactPromise<B: ?Sized> {
	builder: Rc<B>,
	id: BuilderId,
}

impl<B: Builder + 'static> ArtifactPromise<B> {
	/// Crates a new promise for the given builder.
	///
	pub fn new(builder: B) -> Self {
		let builder = Rc::new(builder);
		let id = (&builder).into();
		
		Self {
			builder,
			id,
		}
	}
	
	/// Changes the generic type of self to `dyn Any`.
	///
	fn into_any(self) -> ArtifactPromise<dyn Any>
			where B: 'static {
		
		ArtifactPromise {
			builder: self.builder,
			id: self.id,
		}
	}
}

impl<B: ?Sized> Borrow<BuilderId> for ArtifactPromise<B> {
	fn borrow(&self) -> &BuilderId {
		&self.id
	}
}

impl<B: ?Sized> Clone for ArtifactPromise<B> {
	fn clone(&self) -> Self {
		ArtifactPromise {
			builder: self.builder.clone(),
			id: self.id,
		}
	}
}

impl Hash for ArtifactPromise<dyn Any> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}

impl PartialEq for ArtifactPromise<dyn Any> {
	fn eq(&self, other: &Self) -> bool {
		self.id.eq(&other.id)
	}
}

impl Eq for ArtifactPromise<dyn Any> {
}

impl<B: Builder + 'static> From<B> for ArtifactPromise<B> {
	fn from(b: B) -> Self {
		Self::new(b)
	}
}



/// Resolves any `ArtifactPromise` used to resolve the dependencies of builders.
///
/// This struct records each resolution in order to keep track of dependencies.
/// This is used for correct cache invalidation.
///
pub struct ArtifactResolver<'a> {
	user: &'a BuilderEntry,
	cache: &'a mut ArtifactCache,
	#[cfg(feature = "diagnostics")]
	diag_builder: &'a BuilderHandle,
}

impl<'a> ArtifactResolver<'a> {
	/// Resolves the given `ArtifactPromise` into its `Artifact`.
	///
	pub fn resolve<B: Builder + 'static>(&mut self, promise: &ArtifactPromise<B>) -> Rc<B::Artifact> {
		#[cfg(feature = "diagnostics")]
		{
			self.cache.do_resolve(self.user, self.diag_builder, promise)
		}
		#[cfg(not(feature = "diagnostics"))]
		{
			self.cache.do_resolve(self.user, promise)
		}
	}
}


/// Id to differentiate builder instances across types.
///
/// Notice, this type simply wraps `*const` to the builder `Rc`s.
/// Consequentially, a `BuilderId`s validity is limited to the life time of
/// the respective `Builder`.
///
#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct BuilderId(*const dyn Any);

impl<B: Builder + 'static> From<&Rc<B>> for BuilderId {
	fn from(rc: &Rc<B>) -> Self {
		BuilderId(rc.as_ref() as &dyn Any as *const dyn Any)
	}
}


#[derive(Clone, Debug)]
struct ArtifactEntry {
	value: Rc<dyn Any>,
}

impl ArtifactEntry {
	fn new<T: Any + Debug>(value: Rc<T>) -> Self {
		ArtifactEntry {
			value,
		}
	}
}


#[derive(Clone, Debug)]
struct BuilderEntry {
	value: ArtifactPromise<dyn Any>,
	id: BuilderId,
}

impl BuilderEntry {
	fn new<T: Builder + Debug + 'static>(value: ArtifactPromise<T>) -> Self {
		let id = value.id;
		
		BuilderEntry {
			value: value.into_any(),
			id,
		}
	}
}

impl Hash for BuilderEntry {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.value.hash(state);
	}
}

impl PartialEq for BuilderEntry {
	fn eq(&self, other: &Self) -> bool {
		self.value.eq(&other.value)
	}
}

impl Eq for BuilderEntry {
}

impl Borrow<BuilderId> for BuilderEntry {
	fn borrow(&self) -> &BuilderId {
		&self.id
	}
}



/// Central structure to prevent dependency duplication on building.
///
/// Notice the debugging version (activating the **`diagnostics`** feature) of this
/// struct contains a debugging `Doctor`, which
/// allows run-time inspection of various events. Therefore, `ArtifactCache`
/// is generic to some `Doctor`.
/// Thus, the `new()` function returns actually a `ArtifactCache<NoopDoctor>`
/// and `new_with_doctor()` returns some `ArtifactCache<T>` those
/// can be store in variables.
///
/// However, since most of the code does not care about the concrete
/// `Doctor` the default generic is `dyn Doctor`.
/// To ease conversion between them, all creatable `ArtifactCache`s
/// (i.e. not `ArtifactCache<dyn Doctor>`) implement `DerefMut` to
/// `ArtifactCache<dyn Doctor>` which has all the methods implemented.
///
pub struct ArtifactCache< #[cfg(feature = "diagnostics")] T: ?Sized = dyn Doctor> {
	/// Maps Builder-Capsules to their Artifact value
	cache: HashMap<ArtifactPromise<dyn Any>, ArtifactEntry>,
	
	/// Tracks the direct promise dependants of each promise
	dependants: HashMap<BuilderId, HashSet<BuilderId>>,
	
	/// The doctor for error diagnostics.
	#[cfg(feature = "diagnostics")]
	doctor: T,
}

#[cfg(feature = "diagnostics")]
impl Default for ArtifactCache<DefDoctor> {
	fn default() -> Self {
		ArtifactCache::new()
	}
}

#[cfg(not(feature = "diagnostics"))]
impl Default for ArtifactCache {
	fn default() -> Self {
		ArtifactCache::new()
	}
}

#[cfg(not(feature = "diagnostics"))]
impl ArtifactCache {
	/// Creates a new empty cache.
	///
	pub fn new() -> Self {
		Self {
			cache: HashMap::new(),
			dependants: HashMap::new(),
		}
	}
}


#[cfg(feature = "diagnostics")]
impl ArtifactCache<DefDoctor> {
	/// Creates a new empty cache with a dummy doctor.
	///
	pub fn new() -> Self {
		Self {
			cache: HashMap::new(),
			dependants: HashMap::new(),
			
			doctor: DefDoctor::default(),
		}
	}
}

#[cfg(feature = "diagnostics")]
impl<T: Doctor + 'static> Deref for ArtifactCache<T> {
	type Target = ArtifactCache;

	fn deref(&self) -> &Self::Target {
		self
	}
}

#[cfg(feature = "diagnostics")]
impl<T: Doctor + 'static> DerefMut for ArtifactCache<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self
	}
}


#[cfg(feature = "diagnostics")]
impl<T: Doctor + 'static> ArtifactCache<T> {
	
	/// Creates new empty cache with given doctor for drop-in inspection.
	///
	/// **Notice: This function is only available if the `diagnostics` feature has been activated**.
	///
	pub fn new_with_doctor(doctor: T) -> Self {
		Self {
			cache: HashMap::new(),
			dependants: HashMap::new(),
			
			doctor,
		}
	}
	
	/// Returns a reference of the inner doctor.
	///
	/// **Notice: This function is only available if the `diagnostics` feature has been activated**.
	///
	pub fn get_doctor(&mut self) -> &mut T {
		&mut self.doctor
	}
	
	/// Consumes the `ArtifactCache` and returns the inner doctor.
	///
	/// **Notice: This function is only available if the `diagnostics` feature has been activated**.
	///
	pub fn into_doctor(self) -> T {
		self.doctor
	}
}

impl ArtifactCache {
	
	/// Resolves the artifact of `promise` and records dependency between `user`
	/// and `promise`.
	///
	fn do_resolve<B: Builder + 'static>(&mut self,
			user: &BuilderEntry,
			#[cfg(feature = "diagnostics")]
			diag_builder: &BuilderHandle,
			promise: &ArtifactPromise<B>) -> Rc<B::Artifact> {
		
		let deps = self.get_dependants(&promise.clone().into_any());
		if !deps.contains(user.borrow()) {
			deps.insert(user.id);
		}
		
		#[cfg(feature = "diagnostics")]
		self.doctor.resolve(diag_builder, &BuilderHandle::new(promise.clone()));
		
		self.get(promise)
	}
	
	/// Returns the vector of dependants of promise
	///
	fn get_dependants(&mut self, promise: &ArtifactPromise<dyn Any>) -> &mut HashSet<BuilderId> {
		if !self.dependants.contains_key(promise.borrow()) {
			self.dependants.insert(*promise.borrow(), HashSet::new());
		}
		
		self.dependants.get_mut(promise.borrow()).unwrap()
	}
	
	/// Get the stored artifact if it exists.
	///
	fn lookup<B: Builder + 'static>(&self, builder: &ArtifactPromise<B>) -> Option<Rc<B::Artifact>>
			where <B as Builder>::Artifact: 'static {
		
		// Get the artifact from the hash map ensuring integrity
		self.cache.get(&builder.id).map(
			|ent| {
				// Ensure value type
				ent.value.clone().downcast()
					.expect("Cached Builder Artifact is of invalid type")
			}
		)
	}
	
	/// Store given artifact for given builder.
	///
	fn insert(&mut self, builder: BuilderEntry, artifact: ArtifactEntry) {
		
		// Insert artifact
		self.cache.insert(
			builder.value,
			artifact,
		);
		
	}
	
	/// Gets the artifact of the given builder.
	///
	/// This method looks up whether the artifact for the given builder is still
	/// present in the cache, or it will use the builder to build and store the
	/// artifact.
	///
	/// Notice the given promise will be stored kept to prevent it from
	/// deallocating. `clear()` or `invalidate()` must be called in order to
	/// free those `Rc`s if required.
	///
	pub fn get<B: Builder + 'static>(&mut self, promise: &ArtifactPromise<B>) -> Rc<B::Artifact>
			where <B as Builder>::Artifact: 'static {
		
		if let Some(rc) = self.lookup(promise) {
			rc
			
		} else {
			let ent = BuilderEntry::new(promise.clone());
			
			#[cfg(feature = "diagnostics")]
			let diag_builder = BuilderHandle::new(promise.clone());
			
			let rc = Rc::new(promise.builder.build(&mut ArtifactResolver {
				user: &ent,
				cache: self,
				#[cfg(feature = "diagnostics")]
				diag_builder: &diag_builder,
			}));
		
			#[cfg(feature = "diagnostics")]
			self.doctor.build(&diag_builder, &ArtifactHandle::new(rc.clone()));
			
			self.insert(ent, ArtifactEntry::new( rc.clone() ));
			
			rc
		}
	}
	
	/// Clears the entire cache including all kept builder and artifact `Rc`s.
	///
	pub fn clear(&mut self) {
		self.cache.clear();
		self.dependants.clear();
		
		#[cfg(feature = "diagnostics")]
		self.doctor.clear();
	}
	
	/// Auxiliary invalidation function using a `BuilderId`.
	///
	fn invalidate_any(&mut self, builder: BuilderId) {
		if let Some(set) = self.dependants.remove(&builder) {
			for dep in set {
				self.invalidate_any(dep);
			}
		}
		
		self.cache.remove(&builder);
	}
	
	/// Clears cached artifact of the given builder and all depending artifacts.
	///
	/// Depending artifacts are all artifacts which used the former during
	/// its building. The dependencies are automatically tracked using the
	/// `ArtifactResolver` struct.
	///
	pub fn invalidate<B: Builder + 'static>(&mut self, promise: &ArtifactPromise<B>) {
		let any_promise = promise.clone().into_any();
		
		self.invalidate_any(any_promise.id);
		
		#[cfg(feature = "diagnostics")]
		self.doctor.invalidate(&BuilderHandle::new(promise.clone()));
	}
}





// -----------

#[cfg(test)]
mod test;






