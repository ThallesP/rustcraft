//! net/minecraft/client/multiplayer/chat/ChatListener.java
//!
//! Where the server's chat goes. Only system messages are ported (`handleSystemMessage`): a chat
//! line, or the action bar's overlay message. The chat delay option (`chatDelay`, 0 by default) and
//! its queue, player chat and its signatures, disguised chat, blocked players' messages
//! (`hideMatchedNames`, which needs the social manager's blocked list) and narration aren't ported:
//! a message goes to the chat as it comes, which is what a delay of 0 does.

/// `queueSize()`: nothing waits with no delay.
pub fn queue_size() -> i64 {
    0
}
