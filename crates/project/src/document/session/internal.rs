use super::Module;
use crate::{user::User, InternalDocumentModel, InternalProject, SharedDocumentModel};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use uuid::Uuid;

/// The internal representation of a document session.
#[derive(Clone, Debug)]
pub struct InternalDocumentSession<M: Module> {
    /// Persistent document data for this session.
    ///
    /// Synced with other sessions and the project.
    pub document_data: M::DocumentData,
    /// Persistent user-specific data for this session.
    pub user_data: M::UserData,
    /// Non-persistent user-specific data for this session.
    pub session_data: M::SessionData,
    /// Non-persistent data shared among users for this session.
    pub shared_data: M::SharedData,
    /// A weak reference to the `Project` to which this document belongs.
    pub _project_ref: Weak<RefCell<InternalProject>>,
    // TODO: delete this and project_ref field -> move to Session
    /// A weak reference to the internal representation of this document.
    pub document_model_ref: Weak<RefCell<InternalDocumentModel<M>>>,
    /// The unique identifier of the document.
    pub _document_uuid: Uuid,
    /// The unique identifier of this session.
    pub session_uuid: Uuid,
}

impl<M: Module> Drop for InternalDocumentSession<M> {
    fn drop(&mut self) {
        // Remove the session from the project, if it still exists.
        if let Some(project_document) = self.document_model_ref.upgrade() {
            let mut project_document = project_document.borrow_mut();
            project_document
                .sessions
                .retain(|(uuid, _)| *uuid != self.session_uuid);

            // If this was the last session, remove the shared session data.
            if project_document.sessions.is_empty() {
                project_document.shared_data = None;
            }
        }
    }
}

impl<M: Module> InternalDocumentSession<M> {
    // TODO: write doc
    #[must_use]
    pub fn new(
        doc_model: &SharedDocumentModel<M>,
        project: &Rc<RefCell<InternalProject>>,
        document_uuid: Uuid,
        user: User,
    ) -> Rc<RefCell<Self>> {
        let mut doc = doc_model.0.borrow_mut();

        // We are the first session, so we need to create the shared data.
        if doc.shared_data.is_none() {
            doc.shared_data = Some(M::SharedData::default());
        }

        // We need to register a new session with the user
        let session_uuid = Uuid::new_v4();
        doc.session_to_user.insert(session_uuid, user);

        // Now construct the session.
        let shared_data = doc.shared_data.clone().unwrap();

        let session = Self {
            document_data: doc.document_data.clone(),
            user_data: doc.user_data.clone(),
            shared_data,
            session_data: M::SessionData::default(),
            _project_ref: Rc::downgrade(project),
            _document_uuid: document_uuid,
            session_uuid,
            document_model_ref: Rc::downgrade(&doc_model.0),
        };
        let session = Rc::new(RefCell::new(session));
        doc.sessions.push((session_uuid, Rc::downgrade(&session)));
        session
    }
}
