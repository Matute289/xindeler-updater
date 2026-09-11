use crate::profiles::Profile;

pub mod default;

/// An action requested by the current view
#[derive(Debug, Clone)]
pub enum Action {
    UpdateProfile(Profile),
}
