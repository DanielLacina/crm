use crate::components::business_components::component::repository_module::BRepository;
use crate::components::business_components::{home::Home, tables::tables::Tables};
use std::sync::Arc;

pub type BusinessHome = Home;
pub type BusinessTables = Tables;

#[derive(Debug, Clone)]
pub struct BusinessComponents {
    pub home: BusinessHome,
    pub tables: BusinessTables,
}

impl BusinessComponents {
    pub async fn new() -> Self {
        let repository = Arc::new(BRepository::new(None).await);
        Self {
            home: BusinessHome::new(repository.clone()),
            tables: BusinessTables::new(repository.clone()),
        }
    }
}
