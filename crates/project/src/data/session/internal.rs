use super::Module;
use crate::{data::DataUuid, user::User, DataModel, InternalData, InternalProject};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use uuid::Uuid;

/// The internal representation of a data session.
#[derive(Clone, Debug)]
pub struct InternalDataSession<M: Module> {
    /// Persistent data for this session.
    ///
    /// Synced with other sessions and the project.
    pub persistent: M::PersistentData,
    /// Persistent user-specific data for this session.
    pub persistent_user: M::PersistentUserData,
    /// Non-persistent user-specific data for this session.
    pub session_data: M::SessionData,
    /// Non-persistent data shared among users for this session.
    pub shared_data: M::SharedData,
    /// A weak reference to the [`crate::Project`] to which this data belongs.
    pub _project_ref: Weak<RefCell<InternalProject>>,
    // TODO: delete this and project_ref field -> move to Session
    /// A weak reference to the internal representation of this data section.
    pub data_model_ref: Weak<RefCell<InternalData<M>>>,
    /// The unique identifier of the document.
    pub _data_uuid: DataUuid,
    /// The unique identifier of this session.
    pub session_uuid: Uuid,
}

impl<M: Module> Drop for InternalDataSession<M> {
    fn drop(&mut self) {
        // Remove the session from the project, if it still exists.
        if let Some(project_data) = self.data_model_ref.upgrade() {
            let mut project_data = project_data.borrow_mut();
            project_data
                .sessions
                .retain(|(uuid, _)| *uuid != self.session_uuid);

            // If this was the last session, remove the shared session data.
            if project_data.sessions.is_empty() {
                project_data.shared_data = None;
            }
        }
    }
}

impl<M: Module> InternalDataSession<M> {
    // TODO: write doc
    #[must_use]
    pub fn new(
        data_model: &DataModel<M>,
        project: &Rc<RefCell<InternalProject>>,
        data_uuid: DataUuid,
        user: User,
    ) -> Rc<RefCell<Self>> {
        let mut data = data_model.0.borrow_mut();

        // We are the first session, so we need to create the shared data.
        if data.shared_data.is_none() {
            data.shared_data = Some(M::SharedData::default());
        }

        // We need to register a new session with the user
        let session_uuid = Uuid::new_v4();
        data.session_to_user.insert(session_uuid, user);

        // Now construct the session.
        let shared_data = data.shared_data.clone().unwrap();

        let session = Self {
            persistent: data.persistent_data.clone(),
            persistent_user: data.user_data.clone(),
            shared_data,
            session_data: M::SessionData::default(),
            _project_ref: Rc::downgrade(project),
            _data_uuid: data_uuid,
            session_uuid,
            data_model_ref: Rc::downgrade(&data_model.0),
        };
        let session = Rc::new(RefCell::new(session));
        data.sessions.push((session_uuid, Rc::downgrade(&session)));
        session
    }
}
