[![Crates.io](https://img.shields.io/crates/v/daab.svg)](https://crates.io/crates/daab)


DAG Aware Artifact Builder
==========================

Rust crate for managing the building of artifacts by builders which are
connected in a directed acyclic graph (DAG) like manner.

This crate provides essentially a cache which keeps artifacts of builders in
order to prevent the same builder to produce multiple equal artifacts.
This could be useful if the builders use consumable resources to create their
artifacts, the building is a heavyweight procedure, or a given DAG dependency
structure among the builders shall be properly preserved among their
artifacts.

The basic principal on which this crate is build, suggests two levels of
abstraction, the builder level and the artifact level. Each builder type has
one specific artifact type. The builders are represented by any struct,
which implements the `Builder` trait, which in turn has an associate type
that specifies the artifact type.

`Builder`s are supposed to be wrapped in `ArtifactPromise`s, which prevents
to call its `Builder::build()` method directly. In other respects, the
`ArtifactPromise` acts a lot like an `Rc` and thus allows to share one
instance among several dependants.
This `Rc`-like structure creates naturally a DAG.

For building a `Builder`s artifact, its `Builder::build()` method is
provided with a `ArtifactResolver` that allows to resolve depending
`ArtifactPromise`s into their respective artifacts, which is,
in order to form a DAG, wrapped behind a `Rc`.

As entry point serves the `ArtifactCache`, which allows outside of a
`Builder` to resolve any `ArtifactPromise` to its artifact. The
`ArtifactCache` is essentially a cache for artifacts. It can be used to
translate any number of `ArtifactPromise`s to their respective artifact,
while sharing their common dependencies.
Consequently, resolving the same `ArtifactPromise` using the same
`ArtifactCache` results in the same `Rc`ed artifact.
However, using different `ArtifactCache`s results in different artifacts.

The `ArtifactCache` has a `clear()` method to reset the cache.
This could be useful to free the resources kept by all artifacts and
builders, which are cached in it, or when artifacts shall be explicitly
recreated, e.g. to form a second independent artifact DAG.
Additionally, `ArtifactCache` has an `invalidate()` method to remove a single
builder and artifact including its dependants (i.e. those artifacts which had
used the invalidated one).

Minimal Rust version: **1.40**



### Example

```rust
use std::rc::Rc;
use daab::*;

// Simple artifact
#[derive(Debug)]
struct Leaf {
    //...
}

// Simple builder
#[derive(Debug)]
struct BuilderLeaf {
    // ...
}
impl BuilderLeaf {
    pub fn new() -> Self {
        Self {
            // ...
        }
    }
}
impl Builder for BuilderLeaf {
    type Artifact = Leaf;

    fn build(&self, _cache: &mut ArtifactResolver) -> Self::Artifact {
        Leaf{
            // ...
        }
    }
}

// Composed artifact, linking to a Leaf
#[derive(Debug)]
struct Node {
    leaf: Rc<Leaf>, // Dependency artifact
    // ...
}

// Composed builder, depending on BuilderLeaf
#[derive(Debug)]
struct BuilderNode {
    builder_leaf: ArtifactPromise<BuilderLeaf>, // Dependency builder
    // ...
}
impl BuilderNode {
    pub fn new(builder_leaf: ArtifactPromise<BuilderLeaf>) -> Self {
        Self {
            builder_leaf,
            // ...
        }
    }
}
impl Builder for BuilderNode {
    type Artifact = Node;

    fn build(&self, cache: &mut ArtifactResolver) -> Self::Artifact {
        // Resolve ArtifactPromise to its artifact
        let leaf = cache.resolve(&self.builder_leaf);

        Node {
            leaf,
            // ...
        }
    }
}

// The cache to storing already created artifacts
let mut cache = ArtifactCache::new();

// Constructing builders
let leaf_builder = ArtifactPromise::new(BuilderLeaf::new());

let node_builder_1 = ArtifactPromise::new(BuilderNode::new(leaf_builder.clone()));
let node_builder_2: ArtifactPromise<_> = BuilderNode::new(leaf_builder.clone()).into();

// Using the cache to access the artifacts from the builders

// The same builder results in same artifact
assert!(Rc::ptr_eq(&cache.get(&node_builder_1), &cache.get(&node_builder_1)));

// Different builders result in different artifacts
assert!( ! Rc::ptr_eq(&cache.get(&node_builder_1), &cache.get(&node_builder_2)));

// Different artifacts may link the same dependent artifact
assert!(Rc::ptr_eq(&cache.get(&node_builder_1).leaf, &cache.get(&node_builder_2).leaf));
```



### Debugging

`daab` comes with extensive debugging gear. However, in order to
keep the production impact as low as possible, the debugging facilities
are capsuled behind the **`diagnostics`** feature.

Of course, the debugging feature is for the user of this crate to
debug their graphs. Therefore, it is rather modelled as a
diagnostics feature (hence the name). The diagnosis
is carried out by a `Doctor`, which is a trait receiving various
internal events in order to record them, print them, or otherwise help
treating the bug.

Care has been taken to keep the **`diagnostics`** feature broadly applicable
as well as keeping the non-`diagnostics` API compatible with the
`diagnostics`-API, meaning that a project not using the
`diagnostics` feature can be easily converted to using
`diagnostics`, usually by just replacing `ArtifactCache::new()`
with `ArtifactCache::new_with_doctor()`.
In order to store the `Doctor` the `ArtifactCache` is generic to a doctor,
which is important on its creation and for storing it by value.
The rest of the time the `ArtifactCache` uses `dyn Doctor` as its default
generic argument.
To ease conversion between them, all creatable `ArtifactCache`s
(i.e. not `ArtifactCache<dyn Doctor>`) implement `DerefMut` to
`&mut ArtifactCache<dyn Doctor>` which has all the important methods
implemented.




### Features

This crate offers the following features:

- **`diagnostics`** enables elaborate graph and cache interaction debugging.
  It adds the `new_with_doctor()` function to the `ArtifactCache` and adds
  the `diagnostics` module with the `Doctor` trait definition and some
  default `Doctor`s.

- **`tynm`** enable the optional dependency on the [`tynm`] crate which adds
  functionality to abbreviate type names, which are used by some default
  `Doctor`s, hence it is only useful in connection with the `diagnostics`
  feature.

[`tynm`]: https://crates.io/crates/tynm


## License

Licensed under Apache License, Version 2.0 ([LICENSE](LICENSE) or https://www.apache.org/licenses/LICENSE-2.0).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.
