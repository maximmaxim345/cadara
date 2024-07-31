use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a user within the `CADara` application.
///
/// Each user is uniquely identified by a [`Uuid`]. The [`User`] struct is used to
/// represent different types of users and their permissions within the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct User {
    /// The unique identifier for the user.
    pub uuid: Uuid,
}

impl User {
    /// Creates a new user with a randomly generated UUID.
    ///
    /// This method is typically used for creating a standard user with unique credentials.
    ///
    /// # Examples
    ///
    /// ```
    /// # use project::user::User;
    /// # use uuid::Uuid;
    /// let user = User::new();
    /// println!("New user UUID: {}", user.uuid);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
        }
    }

    /// Creates a user reserved for local host operations.
    ///
    /// This user is identified by a "nil" UUID, which is a special UUID with all bits set to zero.
    /// It is intended for locally saved projects.
    ///
    /// # Examples
    ///
    /// ```
    /// # use project::user::User;
    /// # use uuid::Uuid;
    /// let local_user = User::local();
    /// assert_eq!(local_user.uuid, Uuid::from_u128(0));
    /// ```
    #[must_use]
    pub const fn local() -> Self {
        Self {
            uuid: Uuid::from_u128(0),
        }
    }

    /// Creates a read-only local user.
    ///
    /// This user is intended for operations that should not modify data, such as viewing or auditing.
    /// The UUID for the read-only user is predefined and distinct from other user UUIDs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use project::user::User;
    /// # use uuid::Uuid;
    /// let read_only_user = User::local_read_only();
    /// assert_eq!(read_only_user.uuid, uuid::Uuid::from_u128(1));
    /// ```
    #[must_use]
    pub const fn local_read_only() -> Self {
        Self {
            uuid: Uuid::from_u128(1), // Replace with the actual UUID for the read-only user.
        }
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}
