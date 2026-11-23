//! PhoeCord installer: everything that changes the user's machine, in one auditable place.
//!
//! Planned (docs/ROADMAP.md, milestones 1.4–1.5): find installed clients (Stable / PTB / Canary, later
//! Phoenix), create and trust the local CA, install or remove the loader DLL, import Vencord
//! settings, and undo all of it. A GUI front end sits on top of this library; the CLI calls the
//! same functions.

/// Discord release channels the installer knows how to find.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Stable,
    Ptb,
    Canary,
    /// The native Rust/GPUI client. Install paths unknown until it ships.
    Phoenix,
}

impl Channel {
    pub fn name(self) -> &'static str {
        match self {
            Channel::Stable => "stable",
            Channel::Ptb => "ptb",
            Channel::Canary => "canary",
            Channel::Phoenix => "phoenix",
        }
    }
}

pub mod autostart;
pub mod ca;
pub mod discover;
pub mod install;
pub mod launch;
pub mod log;
pub mod shortcut;
pub mod system;
pub mod terms;
pub mod uninstall;
pub mod vencord;
