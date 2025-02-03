use crate::{
    module_data::ModuleId, ChangeBuilder, DataId, DataView, DocumentId, DocumentView, Path,
    PlannedData, PlannedDocument, ProjectSource, ProjectView,
};
use module::{DataSection, Module};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Represents specific events related to accessing project data.
#[derive(Debug, Clone)]
enum AccessEvent {
    OpenDocument(DocumentId),
    OpenDataById(DataId),
    OpenDataByType(ModuleId),
    OpenDocumentDataById(DocumentId, DataId),
    OpenDocumentDataByType(DocumentId, ModuleId),
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

    /// Freezes the current state of tracked read accesses, producing an immutable
    /// [`CacheValidator`] that can be used to later check if the same accesses
    /// on a different [`ProjectView`] would yield the same results.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The internal mutex is poisoned (indicates another thread panicked while holding the lock)
    #[must_use]
    pub fn freeze(&self) -> CacheValidator {
        CacheValidator(std::mem::take(
            &mut *self.0.lock().expect("failed to acquire lock"),
        ))
    }
}

/// An immutable record of read accesses performed on a [`TrackedProjectView`].
///
/// This is used to determine if a cached result derived from that view is still valid.
#[derive(Debug, Clone)]
pub struct CacheValidator(Vec<AccessEvent>);

impl CacheValidator {
    /// Checks if the accesses in this [`CacheValidator`] yield the same data on two [`ProjectView`]s.
    ///
    /// This effectively determines if a cache based on these accesses is still valid when comparing
    /// a previous project state (`old_view`) to a potentially updated one (`new_view`).
    ///
    /// # Returns
    /// `true` if the cache is still valid, and `false` otherwise.
    #[must_use]
    #[expect(clippy::too_many_lines)]
    pub fn is_cache_valid(&self, old_view: &ProjectView, new_view: &ProjectView) -> bool {
        if old_view.uuid != new_view.uuid {
            return false;
        }
        for e in &self.0 {
            match e {
                AccessEvent::OpenDocument(document_id) => {
                    if old_view.documents.get(document_id) != new_view.documents.get(document_id) {
                        return false;
                    }
                }
                AccessEvent::OpenDataById(data_id) => {
                    if old_view.data.get(data_id).map(|d| d.module)
                        != new_view.data.get(data_id).map(|d| d.module)
                    {
                        return false;
                    }
                }
                AccessEvent::OpenDataByType(module_id) => {
                    let old_data_ids = old_view
                        .data
                        .iter()
                        .filter(|d| d.1.module == *module_id)
                        .map(|d| d.0);
                    let new_data_ids = new_view
                        .data
                        .iter()
                        .filter(|d| d.1.module == *module_id)
                        .map(|d| d.0);
                    if !old_data_ids.eq(new_data_ids) {
                        return false;
                    }
                }
                AccessEvent::OpenDocumentDataById(document_id, data_id) => {
                    match (
                        old_view.documents.get(document_id),
                        new_view.documents.get(document_id),
                    ) {
                        (None, None) => {}
                        (None, Some(_)) | (Some(_), None) => return false,
                        (Some(old_doc), Some(new_doc)) => {
                            if old_doc.data.iter().any(|d| d == data_id)
                                != new_doc.data.iter().any(|d| d == data_id)
                            {
                                return false;
                            }
                        }
                    }
                }
                AccessEvent::OpenDocumentDataByType(document_id, module_id) => {
                    match (
                        old_view.documents.get(document_id),
                        new_view.documents.get(document_id),
                    ) {
                        (None, None) => {}
                        (None, Some(_)) | (Some(_), None) => return false,
                        (Some(old_doc), Some(new_doc)) => {
                            let old_data_ids = old_doc
                                .data
                                .iter()
                                .map(|d| (d, old_view.data.get(d)))
                                .filter_map(|d| d.1.map(|ed| (d.0, ed)))
                                .filter(|d| d.1.module == *module_id)
                                .map(|d| d.0);
                            let new_data_ids = new_doc
                                .data
                                .iter()
                                .map(|d| (d, new_view.data.get(d)))
                                .filter_map(|d| d.1.map(|ed| (d.0, ed)))
                                .filter(|d| d.1.module == *module_id)
                                .map(|d| d.0);
                            if !old_data_ids.eq(new_data_ids) {
                                return false;
                            }
                        }
                    }
                }
                AccessEvent::AccessPesistent(data_id) => {
                    match (old_view.data.get(data_id), new_view.data.get(data_id)) {
                        (None, None) => {}
                        (None, Some(_)) | (Some(_), None) => return false,
                        (Some(old), Some(new)) => {
                            if old.module != new.module {
                                return false;
                            }
                            if !old.data.persistent_eq(new.data.as_ref()) {
                                return false;
                            }
                        }
                    }
                }
                AccessEvent::AccessPesistentUser(data_id) => {
                    match (old_view.data.get(data_id), new_view.data.get(data_id)) {
                        (None, None) => {}
                        (None, Some(_)) | (Some(_), None) => return false,
                        (Some(old), Some(new)) => {
                            if old.module != new.module {
                                return false;
                            }
                            if !old.data.persistent_user_eq(new.data.as_ref()) {
                                return false;
                            }
                        }
                    }
                }
                AccessEvent::AccessShared(data_id) => {
                    match (old_view.data.get(data_id), new_view.data.get(data_id)) {
                        (None, None) => {}
                        (None, Some(_)) | (Some(_), None) => return false,
                        (Some(old), Some(new)) => {
                            if old.module != new.module {
                                return false;
                            }
                            if !old.data.shared_eq(new.data.as_ref()) {
                                return false;
                            }
                        }
                    }
                }
                AccessEvent::AccessSession(data_id) => {
                    match (old_view.data.get(data_id), new_view.data.get(data_id)) {
                        (None, None) => {}
                        (None, Some(_)) | (Some(_), None) => return false,
                        (Some(old), Some(new)) => {
                            if old.module != new.module {
                                return false;
                            }
                            if !old.data.session_eq(new.data.as_ref()) {
                                return false;
                            }
                        }
                    }
                }
            }
        }

        true
    }

    /// Checks if the associated [`TrackedProjectView`] had any read accesses.
    ///
    /// # Returns
    /// `true` if at least one property about the `project` was queried, and `false` if not.
    #[must_use]
    pub fn was_accessed(&self) -> bool {
        !self.0.is_empty()
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
        self.1
            .track(AccessEvent::OpenDataByType(ModuleId::from_module::<M>()));
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
        self.1.track(AccessEvent::OpenDocumentDataById(
            self.0.clone().into(),
            data_id,
        ));
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
        self.1.track(AccessEvent::OpenDocumentDataByType(
            self.0.clone().into(),
            ModuleId::from_module::<M>(),
        ));
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
    pub fn move_to_document(&self, new_owner: &crate::TrackedDocumentView, cb: &mut ChangeBuilder) {
        self.0.move_to_document(&new_owner.0, cb);
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
