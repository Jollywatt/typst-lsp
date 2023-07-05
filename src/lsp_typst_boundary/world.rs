use comemo::Prehashed;
use tokio::sync::OwnedRwLockReadGuard;
use tracing::{error, warn};
use typst::diag::{EcoString, FileResult};
use typst::eval::{Datetime, Library};
use typst::file::{FileId, PackageSpec};
use typst::font::{Font, FontBook};
use typst::util::Bytes;
use typst::World;

use crate::workspace::resource_manager::ResourceManager;
use crate::workspace::source::Source;
use crate::workspace::source_manager::SourceManager;
use crate::workspace::Workspace;

use super::clock::Now;
use super::TypstSource;

/// Short-lived struct to implement [`World`] for [`Workspace`]. It wraps a `Workspace` with a main
/// file and exists for the lifetime of a Typst invocation.
pub struct WorkspaceWorld {
    workspace: OwnedRwLockReadGuard<Workspace>,
    main: FileId,
    /// Current time. Will be cached lazily for consistency throughout a compilation.
    now: Now,
}

impl WorkspaceWorld {
    pub fn new(workspace: OwnedRwLockReadGuard<Workspace>, main: FileId) -> Self {
        Self {
            workspace,
            main,
            now: Now::new(),
        }
    }

    pub fn get_workspace(&self) -> &OwnedRwLockReadGuard<Workspace> {
        &self.workspace
    }

    pub fn get_main(&self) -> &Source {
        self.get_workspace()
            .sources
            .get_source_by_id(self.main)
            .expect("main should be cached and so won't cause errors")
    }
}

impl World for WorkspaceWorld {
    fn library(&self) -> &Prehashed<Library> {
        let workspace = self.get_workspace();
        &workspace.typst_stdlib
    }

    fn book(&self) -> &Prehashed<FontBook> {
        self.get_workspace().fonts.book()
    }

    fn main(&self) -> TypstSource {
        match self.source(self.main) {
            Ok(main) => main,
            Err(err) => {
                error!(
                    "this is a bug: failed to get main with id {} with error {err}",
                    self.main
                );
                warn!("returning fake main file");
                TypstSource::detached("")
            }
        }
    }

    fn source(&self, id: FileId) -> FileResult<TypstSource> {
        self.get_workspace().sources().source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.get_workspace().resources().resource(id)
    }

    fn font(&self, id: usize) -> Option<Font> {
        let mut resources = self.get_workspace().resources.write();
        self.get_workspace().fonts.font(id, &mut resources)
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        self.now.with_typst_offset(offset)
    }

    fn packages(&self) -> &[(PackageSpec, Option<EcoString>)] {
        // TODO: implement packages
        &[]
    }
}
