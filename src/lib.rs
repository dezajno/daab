

use std::rc::Rc;
use std::collections::HashMap;
use std::collections::HashSet;
use std::any::Any;
use std::hash::Hash;
use std::hash::Hasher;


/// Represents a builder for an artifact.
///
/// Each builder is supposed to contain all direct depenencies possibly other
/// builders.
/// In the `build()` function, the builder can access the cache inorder to
/// resolve depending builders to their artifact.
///
pub trait Builder {
    type Artifact;
    
    fn build(&self, cache: &mut ArtifactResolver) -> Self::Artifact;
}


/// Encapsulates a builder are handle for its artifact from the ArtifactCache.
///
/// This struct is essentially a wrapper around `Rc<B>`, but it provides a
/// `Hash` and `Eq` implementation based no the identity of the Rcs inner value.
///
/// All clones of an `ArtifactPromise` are considered identical.
///
#[derive(Debug)]
pub struct ArtifactPromise<B: ?Sized> {
	builder: Rc<B>,
}

impl<B> ArtifactPromise<B> {
	/// Crates a new promise for the given builder.
	///
	pub fn new(builder: B) -> Self {
		Self {
			builder: Rc::new(builder),
		}
	}
	
	fn into_any(self) -> ArtifactPromise<dyn Any>
			where B: 'static {
		ArtifactPromise {
			builder: self.builder,
		}
	}
}

impl<B: ?Sized> ArtifactPromise<B> {
	/// Returns the pointer to the inner value.
	///
	fn as_ptr(&self) -> *const B {
		self.builder.as_ref() as &B as *const B
	}
}

impl<B: ?Sized> Clone for ArtifactPromise<B> {
	fn clone(&self) -> Self {
		ArtifactPromise {
			builder: self.builder.clone(),
		}
	}
}

impl<B: ?Sized> Hash for ArtifactPromise<B> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.as_ptr().hash(state);
	}
}

impl<B: ?Sized> PartialEq for ArtifactPromise<B> {
	fn eq(&self, other: &Self) -> bool {
		self.as_ptr().eq(&other.as_ptr())
	}
}

impl<B: ?Sized> Eq for ArtifactPromise<B> {
}


/// Resolves any `ArtifactPromise` used to resolve the dependencies of builders.
///
/// This struct records each resolution in order to keep track of dependencies.
/// This is used for correct cache invalidation.
///
pub struct ArtifactResolver<'a> {
	user: ArtifactPromise<dyn Any>,
	cache: &'a mut ArtifactCache,
}

impl<'a> ArtifactResolver<'a> {
	/// Resolves the given `ArtifactPromise` into its `Artifact`.
	///
	pub fn resolve<B: Builder + 'static>(&mut self, cap: &ArtifactPromise<B>) -> Rc<B::Artifact> {
		self.cache.do_resolve(&self.user, cap)
	}
}



/// Central structure to prevent dependency duplication on building.
///
pub struct ArtifactCache {
	/// Maps Builder-Capsules to their Artifact value
	cache: HashMap<ArtifactPromise<dyn Any>, Rc<dyn Any>>,
	
	/// Tracks the direct promise dependants of each promise
	dependants: HashMap<ArtifactPromise<dyn Any>, HashSet<ArtifactPromise<dyn Any>>>,
}

impl Default for ArtifactCache {
	fn default() -> Self {
		ArtifactCache::new()
	}
}

impl ArtifactCache {
	
	/// Creates new empty cache
	///
	pub fn new() -> Self {
		Self {
			cache: HashMap::new(),
			dependants: HashMap::new(),
		}
	}
	
	/// Resolves artifact of cap and records dependency between user and cap.
	///
	fn do_resolve<B: Builder + 'static>(&mut self, user: &ArtifactPromise<dyn Any>, cap: &ArtifactPromise<B>) -> Rc<B::Artifact> {
		
		let deps = self.get_dependants(&cap.clone().into_any());
		if !deps.contains(user) {
			deps.insert(user.clone());
		}
		
		self.get(cap)
	}
	
	/// Returns the vector of dependants of cap
	///
	fn get_dependants(&mut self, cap: &ArtifactPromise<dyn Any>) -> &mut HashSet<ArtifactPromise<dyn Any>> {
		if !self.dependants.contains_key(cap) {
			self.dependants.insert(cap.clone(), HashSet::new());
		}
		
		self.dependants.get_mut(cap).unwrap()
	}
	
	/// Get the stored artifact if it exists.
	///
	fn lookup<B: Builder + 'static>(&self, builder: &ArtifactPromise<B>) -> Option<Rc<B::Artifact>>
			where <B as Builder>::Artifact: 'static {
		
		// Get the artifact from the hash map ensuring integrity
		self.cache.get(&ArtifactPromise {
			builder: builder.clone().builder,
		}).map(
			|rc| {
				// Ensure value type
				rc.clone().downcast()
					.expect("Cached Builder Artifact is of invalid type")
			}
		)
	}
	
	/// Store given artifact for given builder.
	///
	fn insert<B: Builder + 'static>(&mut self, builder: ArtifactPromise<B>, artifact: Rc<B::Artifact>) {
		
		// Insert artifact
		self.cache.insert(
			ArtifactPromise {
				builder: builder.clone().builder,
			},
			artifact
		);
		
	}
	
	/// Gets the artifact of the given builder.
	///
	/// This method looksup whether the artifact for the given builder is still
	/// present in the cache, or it will use the builder to build and store the
	/// artifact.
	///
	/// Notice the given builder will be stored keept to prevent it from
	/// deallocating. `clear()` must be called inorder to free those Rcs.
	///
	pub fn get<B: Builder + 'static>(&mut self, builder: &ArtifactPromise<B>) -> Rc<B::Artifact>
			where <B as Builder>::Artifact: 'static {
		
		if let Some(rc) = self.lookup(builder) {
			rc
			
		} else {
			let rc = Rc::new(builder.builder.build(&mut ArtifactResolver {
				user: ArtifactPromise {
					builder: builder.clone().builder,
				},
				cache: self,
			}));
			
			self.insert(builder.clone(), rc.clone());
			
			rc
		}
	}
	
	/// Clears the entire cache including all hold builder Rcs.
	///
	pub fn clear(&mut self) {
		self.cache.clear();
		self.dependants.clear();
	}
	
	fn invalidate_any(&mut self, any_promise: &ArtifactPromise<dyn Any>) {
		if let Some(set) = self.dependants.remove(any_promise) {
			for dep in set {
				self.invalidate_any(&dep);
			}
		}
		
		self.cache.remove(any_promise);
	}
	
	/// Clears the entire cache including all hold builder Rcs.
	///
	pub fn invalidate<B: Builder + 'static>(&mut self, cap: &ArtifactPromise<B>) {
		let any_promise = cap.clone().into_any();
		
		self.invalidate_any(&any_promise);
	}
}





// -----------

#[cfg(test)]
mod tests {
	use super::*;
	
	use std::rc::Rc;
	use std::sync::atomic::Ordering;
	use std::sync::atomic::AtomicU32;
	
	
	// Dummy counter to differentiate the leaf instances
	static COUNTER: AtomicU32 = AtomicU32::new(0);

	#[derive(Debug, PartialEq, Eq)]
	struct Leaf {
		id: u32,
	}

	#[derive(Debug)]
	struct BuilderLeaf {
		// empty
	}

	impl BuilderLeaf {
		pub fn new() -> Self {
		    Self {
		    	// empty
		    }
		}
	}

	impl Builder for BuilderLeaf {
		type Artifact = Leaf;
		
		fn build(&self, _cache: &mut ArtifactResolver) -> Self::Artifact {
		    Leaf{
				id: COUNTER.fetch_add(1, Ordering::SeqCst),
			}
		}
	}


	#[derive(Debug, PartialEq, Eq)]
	struct SimpleNode {
		id: u32,
		leaf: Rc<Leaf>,
	}

	#[derive(Debug)]
	struct BuilderSimpleNode {
		leaf: ArtifactPromise<BuilderLeaf>,
	}

	impl BuilderSimpleNode {
		pub fn new(leaf: ArtifactPromise<BuilderLeaf>) -> Self {
		    Self {
		        leaf,
		    }
		}
	}

	impl Builder for BuilderSimpleNode {
		type Artifact = SimpleNode;
		
		fn build(&self, cache: &mut ArtifactResolver) -> Self::Artifact {
			let leaf = cache.resolve(&self.leaf);
		    
		    SimpleNode{
		    	id: COUNTER.fetch_add(1, Ordering::SeqCst),
		    	leaf
		    }
		}
	}

	#[derive(Debug, PartialEq, Eq)]
	enum LeafOrNodes {
		Leaf(Rc<Leaf>),
		Nodes {
			left: Rc<ComplexNode>,
			right: Rc<ComplexNode>
		},
	}

	#[derive(Debug)]
	enum BuilderLeafOrNodes {
		Leaf(ArtifactPromise<BuilderLeaf>),
		Nodes {
			left: ArtifactPromise<BuilderComplexNode>,
			right: ArtifactPromise<BuilderComplexNode>
		},
	}
	
	impl BuilderLeafOrNodes {
		fn build(&self, cache: &mut ArtifactResolver) -> LeafOrNodes {
			match self {
				Self::Leaf(l) => {
					LeafOrNodes::Leaf(cache.resolve(l))
				},
				Self::Nodes{left, right} => {
					LeafOrNodes::Nodes{
						left: cache.resolve(left),
						right: cache.resolve(right),
					}
				},
			}
		}
	}

	#[derive(Debug, PartialEq, Eq)]
	struct ComplexNode {
		id: u32,
		inner: LeafOrNodes,
	}
	
	impl ComplexNode {
		pub fn leaf(&self) -> Option<&Rc<Leaf>> {
			if let LeafOrNodes::Leaf(ref l) = self.inner {
				Some(l)
			} else {
				None
			}
		}
		
		pub fn left(&self) -> Option<&Rc<ComplexNode>> {
			if let LeafOrNodes::Nodes{ref left, ..} = self.inner {
				Some(left)
			} else {
				None
			}
		}
		
		pub fn right(&self) -> Option<&Rc<ComplexNode>> {
			if let LeafOrNodes::Nodes{ref right, ..} = self.inner {
				Some(right)
			} else {
				None
			}
		}
	}

	#[derive(Debug)]
	struct BuilderComplexNode {
		inner: BuilderLeafOrNodes,
	}

	impl BuilderComplexNode {
		pub fn new_leaf(leaf: ArtifactPromise<BuilderLeaf>) -> Self {
		    Self {
		        inner: BuilderLeafOrNodes::Leaf(leaf),
		    }
		}
		
		pub fn new_nodes(left: ArtifactPromise<BuilderComplexNode>, right: ArtifactPromise<BuilderComplexNode>) -> Self {
		    Self {
		        inner: BuilderLeafOrNodes::Nodes{left, right},
		    }
		}
	}

	impl Builder for BuilderComplexNode {
		type Artifact = ComplexNode;
		
		fn build(&self, cache: &mut ArtifactResolver) -> Self::Artifact {
		    ComplexNode{
		    	id: COUNTER.fetch_add(1, Ordering::SeqCst),
		    	inner: self.inner.build(cache),
		    }
		}
	}
    
    #[test]
	fn test_leaf() {
		let mut cache = ArtifactCache::new();
		
		let leaf1 = ArtifactPromise::new(BuilderLeaf::new());
		let leaf2 = ArtifactPromise::new(BuilderLeaf::new());
		
		//println!("BuilderLeaf: {:?}; {:?}", leaf1, leaf2);
		
		// Ensure same builder results in same artifact
		assert_eq!(cache.get(&leaf1), cache.get(&leaf1));
		
		// Ensure different builder result in  different artifacts
		assert_ne!(cache.get(&leaf1), cache.get(&leaf2));
	}
    
    #[test]
	fn test_node() {
		let mut cache = ArtifactCache::new();
		
		let leaf1 = ArtifactPromise::new(BuilderLeaf::new());
		let leaf2 = ArtifactPromise::new(BuilderLeaf::new());
		
		let node1 = ArtifactPromise::new(BuilderSimpleNode::new(leaf1.clone()));
		let node2 = ArtifactPromise::new(BuilderSimpleNode::new(leaf2.clone()));
		let node3 = ArtifactPromise::new(BuilderSimpleNode::new(leaf2.clone()));
		
		// Ensure same builder results in same artifact
		assert_eq!(cache.get(&node1), cache.get(&node1));
		
		// Ensure different builder result in  different artifacts
		assert_ne!(cache.get(&node2), cache.get(&node3));
		
		// Enusre that different artifacts may link the same dependent artifact
		assert_eq!(cache.get(&node2).leaf, cache.get(&node3).leaf);
		
	}
    
    #[test]
	fn test_complex() {
		let mut cache = ArtifactCache::new();
		
		let leaf1 = ArtifactPromise::new(BuilderLeaf::new());
		let leaf2 = ArtifactPromise::new(BuilderLeaf::new());
		
		let nodef1 = ArtifactPromise::new(BuilderComplexNode::new_leaf(leaf1.clone()));
		let nodef2 = ArtifactPromise::new(BuilderComplexNode::new_leaf(leaf2.clone()));
		let nodef3 = ArtifactPromise::new(BuilderComplexNode::new_leaf(leaf2.clone()));
		
		let noden1 = ArtifactPromise::new(BuilderComplexNode::new_nodes(nodef1.clone(), nodef2.clone()));
		let noden2 = ArtifactPromise::new(BuilderComplexNode::new_nodes(nodef3.clone(), noden1.clone()));
		let noden3 = ArtifactPromise::new(BuilderComplexNode::new_nodes(noden2.clone(), noden2.clone()));
		
		// Ensure same builder results in same artifact
		assert_eq!(cache.get(&noden3), cache.get(&noden3));
		
		// Ensure different builder result in  different artifacts
		assert_ne!(cache.get(&noden1), cache.get(&noden2));
		
		let artifact_leaf = cache.get(&leaf1);
		let artifact_node = cache.get(&noden1);
		let artifact_root = cache.get(&noden3);
		
		assert_eq!(artifact_root.left(), artifact_root.right());
		
		assert_eq!(artifact_root.left().unwrap().right(), Some(&artifact_node));
		assert_eq!(artifact_node.left().unwrap().leaf(), Some(&artifact_leaf));
		
	}
    
    #[test]
	fn test_clear() {
		let mut cache = ArtifactCache::new();
		
		let leaf1 = ArtifactPromise::new(BuilderLeaf::new());
		
		let artifact1 = cache.get(&leaf1);
		
		cache.clear();
		
		let artifact2 = cache.get(&leaf1);
		
		// Ensure artifacts differ after clear
		assert_ne!(artifact1, artifact2);
		
	}
    
    #[test]
	fn test_complex_clear() {
		let mut cache = ArtifactCache::new();
		
		let leaf1 = ArtifactPromise::new(BuilderLeaf::new());
		let leaf2 = ArtifactPromise::new(BuilderLeaf::new());
		
		let nodef1 = ArtifactPromise::new(BuilderComplexNode::new_leaf(leaf1.clone()));
		let nodef2 = ArtifactPromise::new(BuilderComplexNode::new_leaf(leaf2.clone()));
		let nodef3 = ArtifactPromise::new(BuilderComplexNode::new_leaf(leaf2.clone()));
		
		let noden1 = ArtifactPromise::new(BuilderComplexNode::new_nodes(nodef1.clone(), nodef2.clone()));
		let noden2 = ArtifactPromise::new(BuilderComplexNode::new_nodes(nodef3.clone(), noden1.clone()));
		let noden3 = ArtifactPromise::new(BuilderComplexNode::new_nodes(noden2.clone(), noden2.clone()));
		
		let artifact_leaf = cache.get(&leaf1);
		let artifact_node = cache.get(&noden1);
		let artifact_root = cache.get(&noden3);
		
		cache.clear();
		
		let artifact_leaf_2 = cache.get(&leaf1);
		let artifact_node_2 = cache.get(&noden1);
		let artifact_root_2 = cache.get(&noden3);
		
		assert_ne!(artifact_leaf, artifact_leaf_2);
		assert_ne!(artifact_node, artifact_node_2);
		assert_ne!(artifact_root, artifact_root_2);
		
	}
    
    #[test]
	fn test_invalidate() {
		let mut cache = ArtifactCache::new();
		
		let leaf1 = ArtifactPromise::new(BuilderLeaf::new());
		
		let artifact1 = cache.get(&leaf1);
		
		cache.invalidate(&leaf1);
		
		let artifact2 = cache.get(&leaf1);
		
		// Ensure artifacts differ after clear
		assert_ne!(artifact1, artifact2);
		
	}
    
    #[test]
	fn test_complex_invalidate() {
		let mut cache = ArtifactCache::new();
		
		let leaf1 = ArtifactPromise::new(BuilderLeaf::new());
		let leaf2 = ArtifactPromise::new(BuilderLeaf::new());
		
		let nodef1 = ArtifactPromise::new(BuilderComplexNode::new_leaf(leaf1.clone()));
		let nodef2 = ArtifactPromise::new(BuilderComplexNode::new_leaf(leaf2.clone()));
		let nodef3 = ArtifactPromise::new(BuilderComplexNode::new_leaf(leaf2.clone()));
		
		let noden1 = ArtifactPromise::new(BuilderComplexNode::new_nodes(nodef1.clone(), nodef2.clone()));
		let noden2 = ArtifactPromise::new(BuilderComplexNode::new_nodes(nodef3.clone(), noden1.clone()));
		let noden3 = ArtifactPromise::new(BuilderComplexNode::new_nodes(noden2.clone(), noden2.clone()));
		
		let artifact_leaf = cache.get(&leaf1);
		let artifact_node = cache.get(&noden1);
		let artifact_root = cache.get(&noden3);
		
		cache.invalidate(&noden1);
		
		let artifact_leaf_2 = cache.get(&leaf1);
		let artifact_node_2 = cache.get(&noden1);
		let artifact_root_2 = cache.get(&noden3);
		
		assert_eq!(artifact_leaf, artifact_leaf_2);
		assert_ne!(artifact_node, artifact_node_2);
		assert_ne!(artifact_root, artifact_root_2);
		
	}
}






