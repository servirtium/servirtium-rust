use crate::InteractionData;
use std::fmt::Debug;

pub trait InteractionManager: Debug {
    fn load_interactions(
        &self,
    ) -> Result<Vec<InteractionData>, Box<dyn std::error::Error + Send + Sync>>;
    fn save_interactions(
        &self,
        interactions: &[InteractionData],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn check_data_unchanged(
        &self,
        interactions: &[InteractionData],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
