use crate::{
    ChangeBuilder, DataId, DataView, DocumentId, DocumentView, Path, PlannedData, PlannedDocument,
    ProjectSource, ProjectView,
};
use module::{DataSection, Module};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

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

#[derive(Clone, Debug)]
pub struct AccessRecorder(Arc<Mutex<Vec<AccessEvent>>>);

impl AccessRecorder {
    fn track(&self, access: AccessEvent) {
        self.0.lock().unwrap().push(access);
    }
}

#[derive(Clone, Debug)]
pub struct TrackedProjectView(Arc<ProjectView>, AccessRecorder);

impl ProjectSource for TrackedProjectView {
    fn uuid(&self) -> Uuid {
        self.0.uuid
    }
}

impl TrackedProjectView {
    #[must_use]
    pub fn new(pv: Arc<ProjectView>) -> (Self, AccessRecorder) {
        let accesses = AccessRecorder(Arc::new(Mutex::new(Vec::new())));
        (Self(pv, accesses.clone()), accesses)
    }

    #[must_use]
    pub fn open_document(&self, document_id: DocumentId) -> Option<TrackedDocumentView> {
        self.1.track(AccessEvent::OpenDocument(document_id));
        self.0
            .open_document(document_id)
            .map(|d| TrackedDocumentView(d, self.1.clone()))
    }

    #[must_use]
    pub fn create_document<'b, 'c>(
        &'b self,
        cb: &'c mut ChangeBuilder,
        path: Path,
    ) -> PlannedDocument<'b, 'c> {
        self.0.create_document(cb, path)
    }

    pub fn create_data<M: Module>(&self, cb: &mut ChangeBuilder) -> DataId {
        self.0.create_data::<M>(cb)
    }

    #[must_use]
    pub fn open_data_by_id<M: Module>(&self, data_id: DataId) -> Option<TrackedDataView<M>> {
        self.1.track(AccessEvent::OpenDataById(data_id));
        self.0
            .open_data_by_id(data_id)
            .map(|d| TrackedDataView(d, self.1.clone()))
    }

    pub fn open_data_by_type<M: Module>(&self) -> impl Iterator<Item = TrackedDataView<M>> + '_ {
        self.0
            .open_data_by_type()
            .map(|d| TrackedDataView(d, self.1.clone()))
    }
}

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
    #[must_use]
    pub fn open_data_by_id<M: Module>(&self, data_id: DataId) -> Option<TrackedDataView<M>> {
        self.0
            .open_data_by_id(data_id)
            .map(|d| TrackedDataView(d, self.1.clone()))
    }

    pub fn open_data_by_type<M: Module>(&self) -> impl Iterator<Item = TrackedDataView<M>> + '_ {
        self.0
            .open_data_by_type()
            .map(|d| TrackedDataView(d, self.1.clone()))
    }

    #[must_use]
    pub fn create_data<'b, 'c, M: Module>(
        &'b self,
        cb: &'c mut ChangeBuilder,
    ) -> PlannedData<'b, 'c, M> {
        self.0.create_data(cb)
    }

    pub fn delete(&self, cb: &mut ChangeBuilder) {
        self.0.delete(cb);
    }
}

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
    pub fn apply_persistent(
        &self,
        args: <M::PersistentData as DataSection>::Args,
        cb: &mut ChangeBuilder,
    ) {
        self.0.apply_persistent(args, cb);
    }

    pub fn apply_persistent_user(
        &self,
        args: <M::PersistentUserData as DataSection>::Args,
        cb: &mut ChangeBuilder,
    ) {
        self.0.apply_persistent_user(args, cb);
    }

    pub fn apply_session(
        &self,
        args: <M::SessionData as DataSection>::Args,
        cb: &mut ChangeBuilder,
    ) {
        self.0.apply_session(args, cb);
    }

    pub fn apply_shared(&self, args: <M::SharedData as DataSection>::Args, cb: &mut ChangeBuilder) {
        self.0.apply_shared(args, cb);
    }

    pub fn delete(&self, cb: &mut ChangeBuilder) {
        self.0.delete(cb);
    }

    pub fn move_to_document(&self, new_owner: &crate::DocumentView, cb: &mut ChangeBuilder) {
        self.0.move_to_document(new_owner, cb);
    }

    pub fn make_orphan(&self, cb: &mut ChangeBuilder) {
        self.0.make_orphan(cb);
    }

    #[must_use]
    pub fn persistent(&self) -> &'a &<M as Module>::PersistentData {
        self.1.track(AccessEvent::AccessPesistent(self.0.id));
        &self.0.persistent
    }

    #[must_use]
    pub fn persistent_user(&self) -> &'a &<M as Module>::PersistentUserData {
        self.1.track(AccessEvent::AccessPesistentUser(self.0.id));
        &self.0.persistent_user
    }

    #[must_use]
    pub fn shared_data(&self) -> &'a &<M as Module>::SharedData {
        self.1.track(AccessEvent::AccessShared(self.0.id));
        &self.0.shared_data
    }

    #[must_use]
    pub fn session_data(&self) -> &'a &<M as Module>::SessionData {
        self.1.track(AccessEvent::AccessSession(self.0.id));
        &self.0.session_data
    }
}
