use crate::components::business_components::component::{
    repository_module::BRepository, BColumn, BConstraint, BDataType, BTableChangeEvents,
    BTableData, BTableGeneral, BTableIn, BTableInfo, BTableInsertedData, BusinessComponent,
};

use crate::components::business_components::components::BusinessConsole;
use crate::components::business_components::tables::utils::set_tables_general_info;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;
use tokio::task;

#[derive(Debug, Clone)]
pub struct Tables {
    repository: Arc<BRepository>,
    pub table_info: Arc<BTableInfo>,
    pub table_data: Arc<BTableData>,
    pub tables_general_info: Arc<AsyncMutex<Vec<BTableGeneral>>>,
    console: Arc<BusinessConsole>,
}

impl BusinessComponent for Tables {
    async fn initialize_component(&self) {
        set_tables_general_info(self.repository.clone(), self.tables_general_info.clone()).await;
    }
}

impl Tables {
    pub fn new(repository: Arc<BRepository>, console: Arc<BusinessConsole>) -> Self {
        let tables_general_info = Arc::new(AsyncMutex::new(vec![]));
        let table_data = Arc::new(BTableData::new(
            repository.clone(),
            console.clone(),
            tables_general_info.clone(),
        ));

        Self {
            table_info: Arc::new(BTableInfo::new(
                repository.clone(),
                console.clone(),
                tables_general_info.clone(),
                table_data.clone(),
            )),
            table_data,
            repository,
            tables_general_info,
            console,
        }
    }

    pub async fn add_table(&self, mut table_in: BTableIn) {
        // Check if no column has a primary key constraint
        if !table_in.columns.iter().any(|column| {
            column
                .constraints
                .iter()
                .any(|constraint| matches!(constraint, BConstraint::PrimaryKey))
        }) {
            // Add a default primary key column if none exists
            table_in.columns.push(BColumn {
                name: "id".to_string(),
                datatype: BDataType::INTEGER,
                constraints: vec![BConstraint::PrimaryKey],
            });
        }

        // Create the table and update general info
        self.repository.create_table(&table_in).await;
        set_tables_general_info(self.repository.clone(), self.tables_general_info.clone()).await;
    }

    pub async fn delete_table(&self, table_name: String) {
        self.repository.delete_table(&table_name).await;
        let table_info = self.table_info.clone();
        let table_data = self.table_data.clone();
        task::spawn_blocking(move || {
            let reset_table_info =
                if let Some(current_table_name) = table_info.table_name.blocking_lock().as_ref() {
                    *current_table_name == table_name.to_string()
                } else {
                    false
                };
            if reset_table_info {
                table_info.reset_table_info();
            }

            let reset_table_data = if let Some(table_inserted_data) =
                table_data.table_inserted_data.blocking_lock().as_ref()
            {
                table_inserted_data.table_name == table_name
            } else {
                false
            };
            if reset_table_data {
                table_data.reset_table_data();
            }
        })
        .await;
        set_tables_general_info(self.repository.clone(), self.tables_general_info.clone()).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::business_components::component::repository_module::BRepositoryConsole;
    use crate::components::business_components::tables::test_utils::{
        create_btable_general, create_repository_table_and_console, default_table_in,
        sort_by_table_name,
    };
    use sqlx::PgPool;

    async fn tables_component(pool: PgPool, table_in: &BTableIn) -> Tables {
        let (repository_result, console_result) =
            create_repository_table_and_console(pool, table_in).await;
        Tables::new(repository_result, console_result)
    }

    async fn initialized_tables_component(pool: PgPool, table_in: &BTableIn) -> Tables {
        let tables = tables_component(pool, table_in).await;
        tables.initialize_component().await;
        tables
    }

    #[sqlx::test]
    async fn test_initialize_tables_component(pool: PgPool) {
        let table_in = default_table_in();
        let tables = initialized_tables_component(pool, &table_in).await;

        let mut expected_tables_general_info = vec![create_btable_general(&table_in)];
        let mut tables_general_info = tables.tables_general_info.lock().await.clone();

        // Sort both vectors
        sort_by_table_name(&mut expected_tables_general_info);
        sort_by_table_name(&mut tables_general_info);

        assert_eq!(tables_general_info, expected_tables_general_info);
    }

    #[sqlx::test]
    async fn test_add_table(pool: PgPool) {
        let initial_table_in = default_table_in();
        let tables = initialized_tables_component(pool, &initial_table_in).await;

        let new_table_in = BTableIn {
            table_name: String::from("products"),
            columns: vec![BColumn {
                name: String::from("product_name"),
                datatype: BDataType::TEXT,
                constraints: vec![],
            }],
        };

        // Add a new table
        tables.add_table(new_table_in.clone()).await;

        // Prepare expected results
        let mut new_table = new_table_in;
        new_table.columns.push(BColumn {
            name: String::from("id"),
            datatype: BDataType::INTEGER,
            constraints: vec![BConstraint::PrimaryKey],
        });
        let mut expected_tables_general_info = vec![
            create_btable_general(&initial_table_in),
            create_btable_general(&new_table),
        ];

        let mut tables_general_info = tables.tables_general_info.lock().await.clone();

        // Sort both vectors
        sort_by_table_name(&mut expected_tables_general_info);
        sort_by_table_name(&mut tables_general_info);

        assert_eq!(tables_general_info, expected_tables_general_info);
    }

    #[sqlx::test]
    async fn test_delete_table(pool: PgPool) {
        let table_in = default_table_in();
        let tables = initialized_tables_component(pool, &table_in).await;

        // Delete the initial table
        tables.delete_table(table_in.table_name.clone()).await;

        // Verify no tables exist in `tables_general_info`
        let tables_general_info = tables.tables_general_info.lock().await;
        assert!(tables_general_info.is_empty());
    }
}
