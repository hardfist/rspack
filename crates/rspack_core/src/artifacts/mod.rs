use std::ops::{Deref, DerefMut};

use rayon::iter::{FromParallelIterator, IntoParallelIterator};
use rspack_collections::{IdentifierMap, IdentifierSet, UkeyMap};
use rspack_error::Diagnostic;

use crate::incremental::{Incremental, IncrementalPasses};
use crate::{
  chunk_graph_chunk::ChunkId, ChunkHashesResult, ChunkRenderResult, ChunkUkey, ModuleId,
  RuntimeGlobals,
};

mod cgm_hash_artifact;
mod cgm_runtime_requirement_artifact;
mod code_generation_results;
mod side_effects_do_optimize_artifact;

pub use cgm_hash_artifact::*;
pub use cgm_runtime_requirement_artifact::*;
pub use code_generation_results::*;
pub use side_effects_do_optimize_artifact::*;

pub trait ArtifactExt {
  const PASS: IncrementalPasses;
  fn reset(&mut self);
}

impl<T: ArtifactExt + ?Sized> ArtifactExt for Box<T> {
  const PASS: IncrementalPasses = T::PASS;
  fn reset(&mut self) {
    (**self).reset();
  }
}

pub fn reset_artifact_if_passes_disabled<T: ArtifactExt>(
  incremental: &Incremental,
  artifact: &mut T,
) {
  if !incremental.passes_enabled(T::PASS) {
    artifact.reset();
  }
}

macro_rules! define_artifact {
  (
    $(#[$meta:meta])*
    $name:ident($inner:ty) => $pass:ident
  ) => {
    $(#[$meta])*
    #[derive(Debug, Default, Clone)]
    pub struct $name($inner);

    impl $name {
      pub fn new(inner: $inner) -> Self {
        Self(inner)
      }

      pub fn into_inner(self) -> $inner {
        self.0
      }
    }

    impl Deref for $name {
      type Target = $inner;
      fn deref(&self) -> &Self::Target {
        &self.0
      }
    }

    impl DerefMut for $name {
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
      }
    }

    impl ArtifactExt for $name {
      const PASS: IncrementalPasses = IncrementalPasses::$pass;
      fn reset(&mut self) {
        self.0.clear();
      }
    }
  };
}

define_artifact!(AsyncModulesArtifact(IdentifierSet) => INFER_ASYNC_MODULES);
define_artifact!(ImportedByDeferModulesArtifact(IdentifierSet) => INFER_ASYNC_MODULES);
define_artifact!(ModuleIdsArtifact(IdentifierMap<ModuleId>) => MODULE_IDS);
define_artifact!(CgcRuntimeRequirementsArtifact(UkeyMap<ChunkUkey, RuntimeGlobals>) => CHUNKS_RUNTIME_REQUIREMENTS);
define_artifact!(ChunkIdsArtifact(UkeyMap<ChunkUkey, ChunkId>) => CHUNK_IDS);
define_artifact!(ChunkHashesArtifact(UkeyMap<ChunkUkey, ChunkHashesResult>) => CHUNKS_HASHES);

// ChunkRenderArtifact needs IntoIterator for use in for loops
#[derive(Debug, Default, Clone)]
pub struct ChunkRenderArtifact(UkeyMap<ChunkUkey, ChunkRenderResult>);

impl ChunkRenderArtifact {
  pub fn new(inner: UkeyMap<ChunkUkey, ChunkRenderResult>) -> Self {
    Self(inner)
  }

  pub fn into_inner(self) -> UkeyMap<ChunkUkey, ChunkRenderResult> {
    self.0
  }
}

impl Deref for ChunkRenderArtifact {
  type Target = UkeyMap<ChunkUkey, ChunkRenderResult>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ChunkRenderArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl ArtifactExt for ChunkRenderArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::CHUNKS_RENDER;
  fn reset(&mut self) {
    self.0.clear();
  }
}

impl IntoIterator for ChunkRenderArtifact {
  type Item = (ChunkUkey, ChunkRenderResult);
  type IntoIter = <UkeyMap<ChunkUkey, ChunkRenderResult> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

// DependenciesDiagnosticsArtifact needs special handling for FromParallelIterator and IntoIterator
#[derive(Debug, Default, Clone)]
pub struct DependenciesDiagnosticsArtifact(IdentifierMap<Vec<Diagnostic>>);

impl DependenciesDiagnosticsArtifact {
  pub fn new(inner: IdentifierMap<Vec<Diagnostic>>) -> Self {
    Self(inner)
  }

  pub fn into_inner(self) -> IdentifierMap<Vec<Diagnostic>> {
    self.0
  }
}

impl Deref for DependenciesDiagnosticsArtifact {
  type Target = IdentifierMap<Vec<Diagnostic>>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for DependenciesDiagnosticsArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl ArtifactExt for DependenciesDiagnosticsArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::DEPENDENCIES_DIAGNOSTICS;
  fn reset(&mut self) {
    self.0.clear();
  }
}

impl FromParallelIterator<(rspack_collections::Identifier, Vec<Diagnostic>)>
  for DependenciesDiagnosticsArtifact
{
  fn from_par_iter<I>(par_iter: I) -> Self
  where
    I: IntoParallelIterator<Item = (rspack_collections::Identifier, Vec<Diagnostic>)>,
  {
    Self(IdentifierMap::from_par_iter(par_iter))
  }
}

impl IntoIterator for DependenciesDiagnosticsArtifact {
  type Item = (rspack_collections::Identifier, Vec<Diagnostic>);
  type IntoIter = <IdentifierMap<Vec<Diagnostic>> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

// SideEffectsOptimizeArtifact needs special handling for FromParallelIterator and IntoIterator
#[derive(Debug, Default, Clone)]
pub struct SideEffectsOptimizeArtifact(UkeyMap<crate::DependencyId, Option<SideEffectsDoOptimize>>);

impl SideEffectsOptimizeArtifact {
  pub fn new(inner: UkeyMap<crate::DependencyId, Option<SideEffectsDoOptimize>>) -> Self {
    Self(inner)
  }

  pub fn into_inner(self) -> UkeyMap<crate::DependencyId, Option<SideEffectsDoOptimize>> {
    self.0
  }
}

impl Deref for SideEffectsOptimizeArtifact {
  type Target = UkeyMap<crate::DependencyId, Option<SideEffectsDoOptimize>>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for SideEffectsOptimizeArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl ArtifactExt for SideEffectsOptimizeArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::SIDE_EFFECTS;
  fn reset(&mut self) {
    self.0.clear();
  }
}

impl FromParallelIterator<(crate::DependencyId, Option<SideEffectsDoOptimize>)>
  for SideEffectsOptimizeArtifact
{
  fn from_par_iter<I>(par_iter: I) -> Self
  where
    I: IntoParallelIterator<Item = (crate::DependencyId, Option<SideEffectsDoOptimize>)>,
  {
    Self(UkeyMap::from_par_iter(par_iter))
  }
}

impl IntoIterator for SideEffectsOptimizeArtifact {
  type Item = (crate::DependencyId, Option<SideEffectsDoOptimize>);
  type IntoIter = <UkeyMap<crate::DependencyId, Option<SideEffectsDoOptimize>> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
