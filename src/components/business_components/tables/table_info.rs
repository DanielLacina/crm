use crate::components::business_components::component::{
    repository_module::BRepository, BColumn, BColumnForeignKey, BColumnsInfo, BConstraint,
    BDataType, BTableChangeEvents, BTableGeneralInfo, BTableIn, BusinessComponent,
};
use crate::components::business_components::components::BusinessConsole;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Debug, Clone)]
pub struct TableInfo {
    repository: Arc<BRepository>,
    pub table_name: String,
    pub columns_info: Vec<BColumn>,
    pub tables_general_info: Option<Arc<AsyncMutex<Vec<BTableGeneralInfo>>>>,
    table_change_events: Vec<BTableChangeEvents>,
    console: Arc<Mutex<BusinessConsole>>,
}

impl BusinessComponent for TableInfo {
    async fn initialize_component(&mut self) {
        self.set_table_info().await;
    }
}

impl TableInfo {
    pub fn new(
        repository: Arc<BRepository>,
        console: Arc<Mutex<BusinessConsole>>,
        tables_general_info: Option<Arc<AsyncMutex<Vec<BTableGeneralInfo>>>>,
        table_name: String,
    ) -> Self {
        Self {
            repository,
            table_name,
            columns_info: vec![],
            table_change_events: vec![],
            console,
            tables_general_info,
        }
    }

    pub fn get_table_change_events(&self) -> Vec<BTableChangeEvents> {
        self.table_change_events.clone()
    }

    async fn set_table_info(&mut self) {
        let columns_info = self
            .repository
            .get_columns_info(&self.table_name)
            .await
            .unwrap();
        let columns_info_with_enums = columns_info
            .into_iter()
            .map(|column_info| BColumn::to_column(column_info))
            .collect();
        self.columns_info = columns_info_with_enums;
    }

    pub fn add_table_change_event(&mut self, table_change_event: BTableChangeEvents) {
        match table_change_event {
            BTableChangeEvents::ChangeTableName(new_table_name) => {
                self.handle_change_table_name(new_table_name);
            }
            BTableChangeEvents::ChangeColumnDataType(column_name, data_type) => {
                self.handle_change_column_datatype(column_name, data_type);
            }
            BTableChangeEvents::ChangeColumnName(column_name, new_column_name) => {
                self.handle_change_column_name(column_name, new_column_name);
            }
            BTableChangeEvents::RemoveColumn(column_name) => {
                self.handle_remove_column(column_name);
            }
            BTableChangeEvents::AddColumn(column_name, data_type) => {
                self.handle_add_column(column_name, data_type);
            }
            BTableChangeEvents::AddForeignKey(column_foreign_key) => {
                self.handle_add_foreign_key(column_foreign_key);
            }
            BTableChangeEvents::RemoveForeignKey(column_name) => {
                self.handle_remove_foreign_key(column_name);
            }
            BTableChangeEvents::AddPrimaryKey(column_name) => {
                self.handle_add_primary_key(column_name);
            }
            BTableChangeEvents::RemovePrimaryKey(column_name) => {
                self.handle_remove_primary_key(column_name);
            }
        }
        let mut locked_console = self.console.lock().unwrap();
        locked_console.write(format!("{:?}", self.table_change_events));
    }

    fn handle_add_column(&mut self, column_name: String, data_type: BDataType) {
        if let Some(existing_event_index) = self.find_existing_remove_column_event(&column_name) {
            if let BTableChangeEvents::RemoveColumn(original_column_name) =
                &self.table_change_events[existing_event_index]
            {
                if let Some(original_column) = self
                    .columns_info
                    .iter()
                    .find(|&column| column.name == *original_column_name)
                {
                    if data_type == original_column.datatype {
                        self.table_change_events.remove(existing_event_index);
                    } else {
                        self.table_change_events.remove(existing_event_index);
                        self.table_change_events
                            .push(BTableChangeEvents::ChangeColumnDataType(
                                column_name,
                                data_type,
                            ));
                    }
                }
            }
        } else {
            self.table_change_events
                .push(BTableChangeEvents::AddColumn(column_name, data_type));
        }
    }

    fn handle_change_table_name(&mut self, table_name: String) {
        if let Some(existing_event_index) = self.find_existing_change_table_name_event() {
            if table_name == self.table_name {
                self.table_change_events.remove(existing_event_index);
            } else {
                self.table_change_events.remove(existing_event_index);
                self.table_change_events
                    .push(BTableChangeEvents::ChangeTableName(table_name));
            }
        } else {
            self.table_change_events
                .push(BTableChangeEvents::ChangeTableName(table_name));
        }
    }

    fn handle_change_column_datatype(&mut self, column_name: String, data_type: BDataType) {
        if let Some(existing_event_index) =
            self.find_existing_change_data_type_column_event(&column_name)
        {
            if let Some(column) = self
                .columns_info
                .iter()
                .find(|&column| column.name == column_name)
            {
                if column.datatype == data_type {
                    self.table_change_events.remove(existing_event_index);
                } else {
                    self.table_change_events.remove(existing_event_index);
                    self.table_change_events
                        .push(BTableChangeEvents::ChangeColumnDataType(
                            column_name,
                            data_type,
                        ));
                }
            } else {
                self.table_change_events.remove(existing_event_index);
                self.table_change_events
                    .push(BTableChangeEvents::ChangeColumnDataType(
                        column_name,
                        data_type,
                    ));
            }
        } else if let Some(existing_event_index) = self.find_existing_add_column_event(&column_name)
        {
            if let BTableChangeEvents::AddColumn(_, added_column_data_type) =
                &self.table_change_events[existing_event_index]
            {
                if *added_column_data_type != data_type {
                    self.table_change_events.remove(existing_event_index);
                    self.table_change_events
                        .push(BTableChangeEvents::AddColumn(column_name, data_type));
                }
            }
        } else {
            self.table_change_events
                .push(BTableChangeEvents::ChangeColumnDataType(
                    column_name,
                    data_type,
                ));
        }
    }

    fn handle_change_column_name(&mut self, column_name: String, new_column_name: String) {
        if column_name == new_column_name {
            return;
        }
        self.rename_existing_datatype_change_event(&column_name, &new_column_name);
        if let Some(existing_event_index) = self.find_existing_rename_column_event(&column_name) {
            self.update_existing_rename_event(existing_event_index, new_column_name.clone());
        } else if let Some(existing_event_index) = self.find_existing_add_column_event(&column_name)
        {
            self.update_existing_add_column_event(
                existing_event_index,
                column_name.clone(),
                new_column_name.clone(),
            );
        } else {
            self.table_change_events
                .push(BTableChangeEvents::ChangeColumnName(
                    column_name.clone(),
                    new_column_name.clone(),
                ));
        }

        if let Some(existing_event_index) = self.find_existing_add_primary_key_event(&column_name) {
            self.table_change_events.remove(existing_event_index);
            self.table_change_events
                .push(BTableChangeEvents::AddPrimaryKey(new_column_name.clone()));
        }
    }

    fn handle_remove_column(&mut self, column_name: String) {
        if let Some(existing_event_index) = self.find_existing_add_primary_key_event(&column_name) {
            self.table_change_events.remove(existing_event_index);
        }
        if let Some(existing_event_index) = self.find_existing_add_foreign_key_event(&column_name) {
            self.table_change_events.remove(existing_event_index);
        }
        if let Some(existing_event_index) = self.find_existing_add_column_event(&column_name) {
            self.table_change_events.remove(existing_event_index);
        } else if let Some(existing_event_index) =
            self.find_existing_change_data_type_column_event(&column_name)
        {
            self.table_change_events.remove(existing_event_index);
            self.table_change_events
                .push(BTableChangeEvents::RemoveColumn(column_name));
        } else if let Some(existing_event_index) =
            self.find_existing_rename_column_event(&column_name)
        {
            if let BTableChangeEvents::ChangeColumnName(
                original_column_name,
                modified_column_name,
            ) = self.table_change_events[existing_event_index].clone()
            {
                self.table_change_events.remove(existing_event_index);
                self.table_change_events
                    .push(BTableChangeEvents::RemoveColumn(original_column_name));
            }
        } else {
            self.table_change_events
                .push(BTableChangeEvents::RemoveColumn(column_name));
        }
    }

    fn handle_add_primary_key(&mut self, column_name: String) {
        if let Some(existing_event_index) =
            self.find_existing_remove_primary_key_event(&column_name)
        {
            self.table_change_events.remove(existing_event_index);
        } else {
            self.table_change_events
                .push(BTableChangeEvents::AddPrimaryKey(column_name));
        }
    }

    fn handle_remove_primary_key(&mut self, column_name: String) {
        if let Some(existing_event_index) = self.find_existing_add_primary_key_event(&column_name) {
            self.table_change_events.remove(existing_event_index);
        } else {
            self.table_change_events
                .push(BTableChangeEvents::RemovePrimaryKey(column_name));
        }
    }

    fn handle_add_foreign_key(&mut self, column_foreign_key: BColumnForeignKey) {
        // only one foreign key allowed
        if let Some(existing_event_index) =
            self.find_existing_add_foreign_key_event(&column_foreign_key.column_name)
        {
            self.table_change_events.remove(existing_event_index);
            self.table_change_events
                .push(BTableChangeEvents::AddForeignKey(column_foreign_key));
        } else if let Some(existing_event_index) =
            self.find_existing_remove_foreign_key_event(&column_foreign_key.column_name)
        {
            self.table_change_events.remove(existing_event_index);
        } else {
            self.table_change_events
                .push(BTableChangeEvents::AddForeignKey(column_foreign_key));
        }
    }

    fn handle_remove_foreign_key(&mut self, column_name: String) {
        if let Some(existing_event_index) = self.find_existing_add_foreign_key_event(&column_name) {
            self.table_change_events.remove(existing_event_index);
        } else {
            self.table_change_events
                .push(BTableChangeEvents::RemoveForeignKey(column_name));
        }
    }

    fn update_existing_rename_event(&mut self, event_index: usize, new_column_name: String) {
        if let BTableChangeEvents::ChangeColumnName(original_column_name, _) =
            self.table_change_events[event_index].clone()
        {
            if original_column_name != new_column_name {
                self.table_change_events
                    .push(BTableChangeEvents::ChangeColumnName(
                        original_column_name,
                        new_column_name,
                    ));
            }
        }
        self.table_change_events.remove(event_index);
    }

    fn update_existing_add_column_event(
        &mut self,
        event_index: usize,
        column_name: String,
        new_column_name: String,
    ) {
        if let BTableChangeEvents::AddColumn(_, added_data_type) =
            self.table_change_events[event_index].clone()
        {
            self.table_change_events.remove(event_index);
            self.handle_add_column(new_column_name, added_data_type);
        }
    }

    fn rename_existing_datatype_change_event(&mut self, column_name: &str, new_column_name: &str) {
        if let Some(event_index) = self.find_existing_change_data_type_column_event(column_name) {
            if let BTableChangeEvents::ChangeColumnDataType(original_column_name, data_type) =
                self.table_change_events[event_index].clone()
            {
                self.table_change_events.remove(event_index);
                self.table_change_events
                    .push(BTableChangeEvents::ChangeColumnDataType(
                        new_column_name.to_string(),
                        data_type,
                    ));
            }
        }
    }

    fn find_existing_remove_primary_key_event(&self, column_name: &str) -> Option<usize> {
        self.table_change_events.iter().position(|event| {
            matches!(event, BTableChangeEvents::RemovePrimaryKey(existing_column_name)
                if existing_column_name == column_name)
        })
    }

    fn find_existing_add_primary_key_event(&self, column_name: &str) -> Option<usize> {
        self.table_change_events.iter().position(|event| {
            matches!(event, BTableChangeEvents::AddPrimaryKey(existing_column_name)
                if existing_column_name == column_name)
        })
    }

    fn find_existing_add_foreign_key_event(&self, column_name: &str) -> Option<usize> {
        self.table_change_events.iter().position(|event| {
            matches!(event, BTableChangeEvents::AddForeignKey(existing_column_foreign_key)
                if  existing_column_foreign_key.column_name == column_name)
        })
    }

    fn find_existing_remove_foreign_key_event(&self, column_name: &str) -> Option<usize> {
        self.table_change_events.iter().position(|event| {
            matches!(event, BTableChangeEvents::RemoveForeignKey(existing_column_name)
                if existing_column_name == column_name)
        })
    }

    fn find_existing_rename_column_event(&self, column_name: &str) -> Option<usize> {
        self.table_change_events.iter().position(|event| {
            matches!(event, BTableChangeEvents::ChangeColumnName(_, modified_column_name)
                if modified_column_name == column_name)
        })
    }

    fn find_existing_remove_column_event(&self, column_name: &str) -> Option<usize> {
        self.table_change_events.iter().position(|event| {
            matches!(event, BTableChangeEvents::RemoveColumn(existing_column_name)
                if existing_column_name == column_name)
        })
    }

    fn find_existing_add_column_event(&self, column_name: &str) -> Option<usize> {
        self.table_change_events.iter().position(|event| {
            matches!(event, BTableChangeEvents::AddColumn(existing_column_name, _)
                if existing_column_name == column_name)
        })
    }

    fn find_existing_change_data_type_column_event(&self, column_name: &str) -> Option<usize> {
        self.table_change_events.iter().position(|event| {
            matches!(event, BTableChangeEvents::ChangeColumnDataType(existing_column_name, _)
                if existing_column_name == column_name)
        })
    }

    fn find_existing_change_table_name_event(&self) -> Option<usize> {
        self.table_change_events
            .iter()
            .position(|event| matches!(event, BTableChangeEvents::ChangeTableName(_)))
    }

    pub async fn set_general_tables_info(&mut self) {
        if let Some(ref tables) = self.tables_general_info {
            let mut locked_tables = tables.lock().await;
            *locked_tables = self.repository.get_general_tables_info().await.unwrap();
        } else {
            self.tables_general_info = Some(Arc::new(AsyncMutex::new(
                self.repository.get_general_tables_info().await.unwrap(),
            )));
        }
    }
    pub async fn alter_table(&mut self) {
        if !self.table_change_events.is_empty() {
            let primary_key_column_names: Vec<String> = self
                .columns_info
                .iter()
                .filter(|&column| {
                    column
                        .constraints
                        .iter()
                        .any(|constraint| matches!(constraint, BConstraint::PrimaryKey))
                })
                .map(|column| column.name.clone())
                .collect();
            let res = self
                .repository
                .alter_table(
                    &self.table_name,
                    &self.table_change_events,
                    &primary_key_column_names,
                )
                .await;
            println!("Alter table result: {:?}", res);
        }

        for event in &self.table_change_events {
            if let BTableChangeEvents::ChangeTableName(updated_table_name) = event {
                self.table_name = updated_table_name.clone();
            }
        }

        self.table_change_events.clear();
        self.set_table_info().await;
        self.set_general_tables_info().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    /// Helper function to create a `TableInfo` instance.
    async fn create_table_info(
        pool: PgPool,
        table_in: &BTableIn,
        tables_general_info: Option<Arc<AsyncMutex<Vec<BTableGeneralInfo>>>>,
    ) -> TableInfo {
        let repository = Arc::new(BRepository::new(Some(pool.clone())).await);
        let console = Arc::new(Mutex::new(BusinessConsole::new()));
        repository.create_table(table_in).await;

        let mut table_info = TableInfo::new(
            repository.clone(),
            console.clone(),
            tables_general_info.clone(),
            table_in.table_name.clone(),
        );

        table_info.set_table_info().await;
        table_info.set_general_tables_info().await; // Initialize tables_general_info
        table_info
    }

    fn default_table_in() -> BTableIn {
        BTableIn {
            table_name: String::from("users"),
            columns: vec![
                BColumn {
                    name: String::from("id"),
                    datatype: BDataType::INTEGER,
                    constraints: vec![BConstraint::PrimaryKey],
                },
                BColumn {
                    name: String::from("name"),
                    datatype: BDataType::TEXT,
                    constraints: vec![],
                },
            ],
        }
    }

    async fn initialized_table_info(
        pool: PgPool,
        table_in: &BTableIn,
        tables_general_info: Option<Arc<AsyncMutex<Vec<BTableGeneralInfo>>>>,
    ) -> TableInfo {
        let mut table_info = create_table_info(pool, table_in, tables_general_info).await;
        table_info.initialize_component().await;
        table_info
    }

    #[sqlx::test]
    async fn test_initialize_component(pool: PgPool) {
        let table_in = default_table_in();
        let tables_general_info = Some(Arc::new(AsyncMutex::new(Vec::new())));

        let mut table_info = initialized_table_info(pool, &table_in, tables_general_info).await;

        let mut expected_columns = vec![
            BColumn {
                name: String::from("id"),
                datatype: BDataType::INTEGER,
                constraints: vec![BConstraint::PrimaryKey],
            },
            BColumn {
                name: String::from("name"),
                datatype: BDataType::TEXT,
                constraints: vec![],
            },
        ];

        expected_columns.sort_by(|a, b| a.name.cmp(&b.name));
        table_info.columns_info.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(table_info.table_name, table_in.table_name);
        assert_eq!(table_info.columns_info, expected_columns);
    }

    #[sqlx::test]
    async fn test_alter_table(pool: PgPool) {
        // Initialize shared components
        let repository = Arc::new(BRepository::new(Some(pool.clone())).await);
        let console = Arc::new(Mutex::new(BusinessConsole::new()));
        let tables_general_info = Some(Arc::new(AsyncMutex::new(Vec::new())));

        // Create the remote table "registrations"
        let remote_table = BTableIn {
            table_name: String::from("registrations"),
            columns: vec![BColumn {
                name: String::from("id"),
                constraints: vec![BConstraint::PrimaryKey],
                datatype: BDataType::INTEGER,
            }],
        };
        repository.create_table(&remote_table).await;

        // Create the initial table "users"
        let table_in = default_table_in();
        repository.create_table(&table_in).await;

        // Initialize TableInfo for "users"
        let mut table_info = TableInfo::new(
            repository.clone(),
            console.clone(),
            tables_general_info.clone(),
            table_in.table_name.clone(),
        );
        table_info.set_table_info().await;
        table_info.set_general_tables_info().await;
        table_info.initialize_component().await;

        // Define a series of table change events
        let table_change_events = vec![
            BTableChangeEvents::AddColumn(String::from("email"), BDataType::TEXT),
            BTableChangeEvents::ChangeColumnName(String::from("name"), String::from("username")),
            BTableChangeEvents::ChangeColumnDataType(String::from("username"), BDataType::INTEGER),
            BTableChangeEvents::AddColumn(String::from("age"), BDataType::INTEGER),
            BTableChangeEvents::RemoveColumn(String::from("age")),
            BTableChangeEvents::ChangeTableName(String::from("customers")),
            BTableChangeEvents::AddColumn(String::from("created_at"), BDataType::TIMESTAMP),
            BTableChangeEvents::ChangeColumnName(
                String::from("created_at"),
                String::from("registration_date"),
            ),
            BTableChangeEvents::RemoveColumn(String::from("registration_date")),
            BTableChangeEvents::AddColumn(String::from("is_active"), BDataType::BOOLEAN),
            BTableChangeEvents::ChangeColumnName(
                String::from("is_active"),
                String::from("active_status"),
            ),
            BTableChangeEvents::AddColumn(String::from("last_login"), BDataType::TIMESTAMP),
            BTableChangeEvents::ChangeColumnDataType(
                String::from("last_login"),
                BDataType::TIMESTAMP,
            ),
            BTableChangeEvents::AddColumn(String::from("country"), BDataType::TEXT),
            BTableChangeEvents::AddPrimaryKey(String::from("country")),
            BTableChangeEvents::ChangeColumnName(String::from("country"), String::from("region")),
            BTableChangeEvents::ChangeTableName(String::from("clients")),
            BTableChangeEvents::AddColumn(String::from("phone_number"), BDataType::TEXT),
            BTableChangeEvents::AddPrimaryKey(String::from("phone_number")),
            BTableChangeEvents::RemoveColumn(String::from("phone_number")),
            BTableChangeEvents::ChangeColumnName(String::from("username"), String::from("name")),
            BTableChangeEvents::AddColumn(String::from("registration_id"), BDataType::INTEGER),
            BTableChangeEvents::AddForeignKey(BColumnForeignKey {
                column_name: String::from("registration_id"),
                referenced_table: String::from("registrations"),
                referenced_column: String::from("id"),
            }),
            BTableChangeEvents::RemoveForeignKey(String::from("registration_id")),
            BTableChangeEvents::AddForeignKey(BColumnForeignKey {
                column_name: String::from("registration_id"),
                referenced_table: String::from("registrations"),
                referenced_column: String::from("id"),
            }),
        ];

        // Apply the table change events
        for event in table_change_events {
            table_info.add_table_change_event(event);
        }

        // Expected events after processing
        let expected_events = vec![
            BTableChangeEvents::AddColumn(String::from("email"), BDataType::TEXT),
            BTableChangeEvents::AddColumn(String::from("active_status"), BDataType::BOOLEAN),
            BTableChangeEvents::AddColumn(String::from("last_login"), BDataType::TIMESTAMP),
            BTableChangeEvents::AddColumn(String::from("region"), BDataType::TEXT),
            BTableChangeEvents::AddPrimaryKey(String::from("region")),
            BTableChangeEvents::ChangeTableName(String::from("clients")),
            BTableChangeEvents::ChangeColumnDataType(String::from("name"), BDataType::INTEGER),
            BTableChangeEvents::AddColumn(String::from("registration_id"), BDataType::INTEGER),
            BTableChangeEvents::AddForeignKey(BColumnForeignKey {
                column_name: String::from("registration_id"),
                referenced_table: String::from("registrations"),
                referenced_column: String::from("id"),
            }),
        ];

        // Verify that the processed events match expected events
        assert_eq!(table_info.table_change_events, expected_events);

        // Apply the alterations to the table
        table_info.alter_table().await;

        // Expected columns after alteration
        let mut expected_columns = vec![
            BColumn {
                name: String::from("id"),
                datatype: BDataType::INTEGER,
                constraints: vec![BConstraint::PrimaryKey],
            },
            BColumn {
                name: String::from("name"),
                datatype: BDataType::INTEGER,
                constraints: vec![],
            },
            BColumn {
                name: String::from("email"),
                datatype: BDataType::TEXT,
                constraints: vec![],
            },
            BColumn {
                name: String::from("active_status"),
                datatype: BDataType::BOOLEAN,
                constraints: vec![],
            },
            BColumn {
                name: String::from("last_login"),
                datatype: BDataType::TIMESTAMP,
                constraints: vec![],
            },
            BColumn {
                name: String::from("region"),
                datatype: BDataType::TEXT,
                constraints: vec![BConstraint::PrimaryKey],
            },
            BColumn {
                name: String::from("registration_id"),
                datatype: BDataType::INTEGER,
                constraints: vec![BConstraint::ForeignKey(
                    String::from("registrations"),
                    String::from("id"),
                )],
            },
        ];

        expected_columns.sort_by(|a, b| a.name.cmp(&b.name));
        table_info.columns_info.sort_by(|a, b| a.name.cmp(&b.name));

        let expected_table_name = String::from("clients");

        // Verify the final state
        assert!(table_info.table_change_events.is_empty());
        assert_eq!(table_info.columns_info, expected_columns);
        assert_eq!(table_info.table_name, expected_table_name);
    }
}
