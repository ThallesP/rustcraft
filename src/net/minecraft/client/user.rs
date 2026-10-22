//! net/minecraft/client/User.java
//!
//! Not ported yet: `getSessionId`, which comes with online-mode login.

use crate::java::util::uuid::UUID;

/// The player the client plays as, and its session's access token.
#[derive(Clone, Debug)]
pub struct User {
    name: String,
    uuid: UUID,
    access_token: String,
    xuid: Option<String>,
    client_id: Option<String>,
}

impl User {
    pub fn new(
        name: String,
        uuid: UUID,
        access_token: String,
        xuid: Option<String>,
        client_id: Option<String>,
    ) -> User {
        User {
            name,
            uuid,
            access_token,
            xuid,
            client_id,
        }
    }

    pub fn get_profile_id(&self) -> UUID {
        self.uuid
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_access_token(&self) -> &str {
        &self.access_token
    }

    pub fn get_client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    pub fn get_xuid(&self) -> Option<&str> {
        self.xuid.as_deref()
    }
}
