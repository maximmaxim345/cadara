use crate::{
    ChangeBuilder, DataId, DataView, DocumentId, DocumentView, Path, PlannedData, PlannedDocument,
    ProjectSource, ProjectView,
};
use module::{DataSection, Module};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Represents specific events related to accessing project data.
#[expect(dead_code)]
#[derive(Debug, Clone)]
enum AccessEvent {
    OpenDocument(DocumentId),
    OpenDataById(DataId),
    AccessPesistent(DataId),
    AccessPesistentUser(DataId),
    AccessShared(DataId),
    AccessSession(DataId),
}

/// A log of read accesses on a [`TrackedProjectView`] for caching purposes.
#[derive(Clone, Debug)]
pub struct AccessRecorder(Arc<Mutex<Vec<AccessEvent>>>);

impl AccessRecorder {
    /// Tracks an [`AccessEvent`] by appending it to the log.
    ///
    /// # Arguments
    /// * `access` - The [`AccessEvent`] to track.
    fn track(&self, access: AccessEvent) {
        self.0.lock().unwrap().push(access);
    }
}

/// A wrapper around [`ProjectView`] that provides access tracking functionality.
///
/// [`TrackedProjectView`] is a wrapper around [`ProjectView`] that tracks all read
/// actions on that [`TrackedProjectView`] related [`TrackedDocumentView`] and
/// [`TrackedDataView`]s.
///
/// This allows implicit dependency tracking of [`ProjectView`]s for caching purposes.
///
/// Create a new [`TrackedProjectView`] and a [`AccessRecorder`] with [`TrackedProjectView::new`].
#[derive(Clone, Debug)]
pub struct TrackedProjectView(Arc<ProjectView>, AccessRecorder);

impl ProjectSource for TrackedProjectView {
    fn uuid(&self) -> Uuid {
        self.0.uuid
    }
}

impl TrackedProjectView {
    /// Creates a new [`TrackedProjectView`] from a [`ProjectView`].
    ///
    /// # Arguments
    /// * `pv` - The [`ProjectView`] to wrap.
    ///
    /// # Returns
    /// A tuple containing the new [`TrackedProjectView`] and its associated [`AccessRecorder`].
    #[must_use]
    pub fn new(pv: Arc<ProjectView>) -> (Self, AccessRecorder) {
        let accesses = AccessRecorder(Arc::new(Mutex::new(Vec::new())));
        (Self(pv, accesses.clone()), accesses)
    }

    /// Opens a read only [`TrackedDocumentView`].
    ///
    /// # Arguments
    /// * `document_id` - The unique identifier of the document to open
    ///
    /// # Returns
    /// An `Option` containing a [`TrackedDocumentView`] if the document was found, or `None` otherwise.
    #[must_use]
    pub fn open_document(&self, document_id: DocumentId) -> Option<TrackedDocumentView> {
        self.1.track(AccessEvent::OpenDocument(document_id));
        self.0
            .open_document(document_id)
            .map(|d| TrackedDocumentView(d, self.1.clone()))
    }

    /// Plans the creation of a new empty document.
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Returns
    /// The unique identifier of the document recorded to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    #[must_use]
    pub fn create_document<'b, 'c>(
        &'b self,
        cb: &'c mut ChangeBuilder,
        path: Path,
    ) -> PlannedDocument<'b, 'c> {
        self.0.create_document(cb, path)
    }

    /// Plans the creation of a new empty data section with type `M`.
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Returns
    ///
    /// The unique identifier of the data recorded to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn create_data<M: Module>(&self, cb: &mut ChangeBuilder) -> DataId {
        self.0.create_data::<M>(cb)
    }

    /// Opens a read only [`TrackedDataView`].
    ///
    /// # Arguments
    /// * `data_id` - The unique identifier of the document to open
    ///
    /// # Type Parameters
    /// * `M` - The [`Module`] expected to describe the data
    ///
    /// # Returns
    /// An `Option` containing a [`TrackedDataView`] if the document was found and is of type `M`, or `None` otherwise.
    #[must_use]
    pub fn open_data_by_id<M: Module>(&self, data_id: DataId) -> Option<TrackedDataView<M>> {
        self.1.track(AccessEvent::OpenDataById(data_id));
        self.0
            .open_data_by_id(data_id)
            .map(|d| TrackedDataView(d, self.1.clone()))
    }

    /// Opens read only [`TrackedDataView`]s to all data with the type `M`.
    ///
    /// # Type Parameters
    /// * `M` - The [`Module`] to filter by
    ///
    /// # Returns
    /// An iterator yielding [`TrackedDataView`]s of type `M` found in this document.
    pub fn open_data_by_type<M: Module>(&self) -> impl Iterator<Item = TrackedDataView<M>> + '_ {
        self.0
            .open_data_by_type()
            .map(|d| TrackedDataView(d, self.1.clone()))
    }
}

/// A wrapper around [`DocumentView`] that provides access tracking functionality.
#[derive(Clone, Debug)]
pub struct TrackedDocumentView<'a>(DocumentView<'a>, AccessRecorder);

impl ProjectSource for TrackedDocumentView<'_> {
    fn uuid(&self) -> Uuid {
        self.0.uuid
    }
}

impl From<TrackedDocumentView<'_>> for DocumentId {
    fn from(dv: TrackedDocumentView<'_>) -> Self {
        dv.0.id
    }
}

impl TrackedDocumentView<'_> {
    /// Opens a read only [`TrackedDataView`] to data contained in this document.
    ///
    /// # Arguments
    /// * `data_id` - The unique identifier of the document to open
    ///
    /// # Type Parameters
    /// * `M` - The [`Module`] expected to describe the data
    ///
    /// # Returns
    /// An `Option` containing a [`TrackedDataView`] if the document was found in this document and is of type `M`, or `None` otherwise.
    #[must_use]
    pub fn open_data_by_id<M: Module>(&self, data_id: DataId) -> Option<TrackedDataView<M>> {
        self.0
            .open_data_by_id(data_id)
            .map(|d| TrackedDataView(d, self.1.clone()))
    }

    /// Opens read only [`TrackedDataView`]s to all data with the type `M`.
    ///
    /// # Type Parameters
    /// * `M` - The [`Module`] to filter by
    ///
    /// # Returns
    /// An iterator yielding [`TrackedDataView`]s of type `M` found in this document.
    pub fn open_data_by_type<M: Module>(&self) -> impl Iterator<Item = TrackedDataView<M>> + '_ {
        self.0
            .open_data_by_type()
            .map(|d| TrackedDataView(d, self.1.clone()))
    }

    /// Plans the creation of a new empty data section with type `M`
    ///
    /// The new data section will be contained in this document
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Returns
    /// The unique identifier of the data recorded to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    #[must_use]
    pub fn create_data<'b, 'c, M: Module>(
        &'b self,
        cb: &'c mut ChangeBuilder,
    ) -> PlannedData<'b, 'c, M> {
        self.0.create_data(cb)
    }

    /// Plans the deletion of this document and all its contained data
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn delete(&self, cb: &mut ChangeBuilder) {
        self.0.delete(cb);
    }
}

/// A wrapper around [`DataView`] that provides access tracking functionality.
#[derive(Clone, Debug)]
pub struct TrackedDataView<'a, M: Module>(DataView<'a, M>, AccessRecorder);

impl<M: Module> ProjectSource for TrackedDataView<'_, M> {
    fn uuid(&self) -> Uuid {
        self.0.uuid
    }
}

impl<M: Module> From<TrackedDataView<'_, M>> for DataId {
    fn from(dv: TrackedDataView<'_, M>) -> Self {
        dv.0.id
    }
}

impl<'a, M: Module> TrackedDataView<'a, M> {
    /// Plans to apply a transaction to [`Module::PersistentData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn apply_persistent(
        &self,
        args: <M::PersistentData as DataSection>::Args,
        cb: &mut ChangeBuilder,
    ) {
        self.0.apply_persistent(args, cb);
    }

    /// Plans to apply a transaction to [`Module::PersistentUserData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn apply_persistent_user(
        &self,
        args: <M::PersistentUserData as DataSection>::Args,
        cb: &mut ChangeBuilder,
    ) {
        self.0.apply_persistent_user(args, cb);
    }

    /// Plans to apply a transaction to [`Module::SessionData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn apply_session(
        &self,
        args: <M::SessionData as DataSection>::Args,
        cb: &mut ChangeBuilder,
    ) {
        self.0.apply_session(args, cb);
    }

    /// Plans to apply a transaction to [`Module::SharedData`].
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `args` - Arguments of the transaction.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn apply_shared(&self, args: <M::SharedData as DataSection>::Args, cb: &mut ChangeBuilder) {
        self.0.apply_shared(args, cb);
    }

    /// Plans the deletion of this data
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn delete(&self, cb: &mut ChangeBuilder) {
        self.0.delete(cb);
    }

    /// Plans to move this data section to another document.
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Arguments
    ///
    /// * `new_owner` - The document to move the data to.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn move_to_document(&self, new_owner: &crate::DocumentView, cb: &mut ChangeBuilder) {
        self.0.move_to_document(new_owner, cb);
    }

    /// Plans to make this data section an orphan (not owned by any document).
    ///
    /// This will not modify the [`crate::Project`], just record this change to `cb`.
    ///
    /// # Panics
    /// If a [`ChangeBuilder`] of a different [`crate::Project`] was passed.
    pub fn make_orphan(&self, cb: &mut ChangeBuilder) {
        self.0.make_orphan(cb);
    }

    /// Accesses the persistent data section, shared by all users.
    ///
    /// # Returns
    /// A reference to [`DataView::persistent`].
    #[must_use]
    pub fn persistent(&self) -> &'a &<M as Module>::PersistentData {
        self.1.track(AccessEvent::AccessPesistent(self.0.id));
        &self.0.persistent
    }

    /// Accesses the persistent user-specific data section.
    ///
    /// # Returns
    /// A reference to [`DataView::persistent_user`].
    #[must_use]
    pub fn persistent_user(&self) -> &'a &<M as Module>::PersistentUserData {
        self.1.track(AccessEvent::AccessPesistentUser(self.0.id));
        &self.0.persistent_user
    }

    /// Accesses the non-persistent data section, that is shared among other users.
    ///
    /// # Returns
    /// A reference to [`DataView::shared_data`].
    #[must_use]
    pub fn shared_data(&self) -> &'a &<M as Module>::SharedData {
        self.1.track(AccessEvent::AccessShared(self.0.id));
        &self.0.shared_data
    }

    /// Accesses the non-persistent user-specific data section.
    ///
    /// # Returns
    /// A reference to [`DataView::session_data`].
    #[must_use]
    pub fn session_data(&self) -> &'a &<M as Module>::SessionData {
        self.1.track(AccessEvent::AccessSession(self.0.id));
        &self.0.session_data
    }
}
