use bevy::prelude::Resource;

/// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DisplayQuality {
    /// Low quality
    Low,
    /// Medium quality
    Medium,
    /// High quality
    High,
}

/// Volume management
#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Volume(pub u8);
