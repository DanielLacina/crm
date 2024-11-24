use crate::components::business_components::{
    component::{
        BColumn, BColumnForeignKey, BConstraint, BDataType, BTable, BTableChangeEvents,
        BTableGeneralInfo, BTableIn, BTableInfo, BusinessComponent,
    },
    components::BusinessTables,
};
use crate::components::ui_components::console::events::ConsoleMessage;
use crate::components::ui_components::{
    component::{Event, UIComponent},
    events::Message,
    tables::events::TableInfoMessage,
};
use iced::{
    border,
    border::Radius,
    font::Font,
    widget::{
        button, column, container, row, scrollable, text, text_input, Button, Column, PickList,
        Row, Scrollable, Text, TextInput,
    },
    Alignment, Background, Border, Color, Element, Length, Shadow, Task, Theme, Vector,
};
use std::iter::zip;

#[derive(Debug, Clone)]
pub struct TableInfoUI {
    table_info: BTableInfo,
    table_name_display: String,
    columns_display: Vec<BColumn>,
    pub tables_general_info: Option<Vec<BTableGeneralInfo>>,
    active_foreign_key_table_within_dropdown: Option<String>, // table in foreign key dropdown that has its columns displayed
    active_foreign_key_dropdown_column: Option<usize>, // column index that wants the foreign key dropdown
                                                       // activated
}

impl UIComponent for TableInfoUI {
    type EventType = TableInfoMessage;

    fn update(&mut self, message: Self::EventType) -> Task<Message> {
        match message {
            Self::EventType::AddColumn => {
                let new_column = BColumn::default();
                self.columns_display.push(new_column.clone());
                self.table_info
                    .add_table_change_event(BTableChangeEvents::AddColumn(
                        new_column.name,
                        new_column.datatype,
                    ));
                Task::done(ConsoleMessage::message(ConsoleMessage::LogMessage(
                    self.formated_table_change_events(),
                )))
            }
            Self::EventType::RemoveColumn(index) => {
                if index < self.columns_display.len() {
                    if let Some(column) = self.columns_display.get_mut(index) {
                        self.table_info
                            .add_table_change_event(BTableChangeEvents::RemoveColumn(
                                column.name.clone(),
                            ));
                        self.columns_display.remove(index);
                    }
                }
                Task::done(ConsoleMessage::message(ConsoleMessage::LogMessage(
                    self.formated_table_change_events(),
                )))
            }
            Self::EventType::UpdateColumnName(index, new_column_name) => {
                if let Some(column) = self.columns_display.get_mut(index) {
                    let original_column_name = column.name.clone();
                    column.name = new_column_name.clone();
                    self.table_info
                        .add_table_change_event(BTableChangeEvents::ChangeColumnName(
                            original_column_name,
                            new_column_name,
                        ));
                }

                Task::done(ConsoleMessage::message(ConsoleMessage::LogMessage(
                    self.formated_table_change_events(),
                )))
            }
            Self::EventType::UpdateColumnType(index, new_datatype) => {
                if let Some(column) = self.columns_display.get_mut(index) {
                    column.datatype = new_datatype.clone();
                    self.table_info.add_table_change_event(
                        BTableChangeEvents::ChangeColumnDataType(column.name.clone(), new_datatype),
                    );
                }
                Task::done(ConsoleMessage::message(ConsoleMessage::LogMessage(
                    self.formated_table_change_events(),
                )))
            }
            Self::EventType::UpdateTableName(new_table_name) => {
                self.table_name_display = new_table_name.clone();
                self.table_info
                    .add_table_change_event(BTableChangeEvents::ChangeTableName(new_table_name));
                Task::done(ConsoleMessage::message(ConsoleMessage::LogMessage(
                    self.formated_table_change_events(),
                )))
            }
            Self::EventType::SubmitUpdateTable => {
                let mut table_info = self.table_info.clone();
                Task::perform(
                    async move {
                        table_info.alter_table().await;
                        table_info
                    },
                    |updated_table_info| {
                        Self::EventType::message(Self::EventType::UpdateTableInfo(
                            updated_table_info,
                        ))
                    },
                )
            }

            Self::EventType::UpdateTableInfo(updated_table_info) => {
                self.columns_display = updated_table_info.columns_info.clone();
                self.table_name_display = updated_table_info.table_name.clone();
                self.table_info = updated_table_info;
                Task::none()
            }
            Self::EventType::AddForeignKey(
                index,
                referenced_table_name,
                referenced_column_name,
            ) => {
                if let Some(column) = self.columns_display.get_mut(index) {
                    if let Some(existing_index) = column.constraints.iter().position(|constraint| {
                        matches!(
                            constraint,
                            BConstraint::ForeignKey(existing_table_name, existing_column_name)
                        )
                    }) {
                        // Remove the foreign key constraint if it exists
                        column.constraints.remove(existing_index);
                        column.constraints.push(BConstraint::ForeignKey(
                            referenced_table_name.clone(),
                            referenced_column_name.clone(),
                        ));
                    } else {
                        // Add the foreign key constraint if it does not exist
                        column.constraints.push(BConstraint::ForeignKey(
                            referenced_table_name.clone(),
                            referenced_column_name.clone(),
                        ));
                    }
                    self.table_info
                        .add_table_change_event(BTableChangeEvents::AddForeignKey(
                            BColumnForeignKey {
                                column_name: column.name.clone(),
                                referenced_table: referenced_table_name,
                                referenced_column: referenced_column_name,
                            },
                        ));
                }

                self.active_foreign_key_dropdown_column = None;
                self.active_foreign_key_table_within_dropdown = None;
                Task::done(ConsoleMessage::message(ConsoleMessage::LogMessage(
                    self.formated_table_change_events(),
                )))
            }
            Self::EventType::RemoveForeignKey(index) => {
                if let Some(column) = self.columns_display.get_mut(index) {
                    if let Some(existing_index) = column.constraints.iter().position(|constraint| {
                        matches!(
                            constraint,
                            BConstraint::ForeignKey(existing_table_name, existing_column_name)
                        )
                    }) {
                        column.constraints.remove(existing_index);
                    }
                    self.table_info
                        .add_table_change_event(BTableChangeEvents::RemoveForeignKey(
                            column.name.clone(),
                        ));
                }
                self.active_foreign_key_dropdown_column = None;
                self.active_foreign_key_table_within_dropdown = None;

                Task::done(ConsoleMessage::message(ConsoleMessage::LogMessage(
                    self.formated_table_change_events(),
                )))
            }
            Self::EventType::ToggleForeignKeyDropdown(index) => {
                // Toggle the dropdown for the specified column
                if self.active_foreign_key_dropdown_column == Some(index) {
                    self.active_foreign_key_dropdown_column = None;
                } else {
                    self.active_foreign_key_dropdown_column = Some(index);
                }
                Task::none()
            }
            Self::EventType::ToggleForeignKeyTable(_, table_name) => {
                // Toggle the column list for the specified table
                if self.active_foreign_key_table_within_dropdown == Some(table_name.clone()) {
                    self.active_foreign_key_table_within_dropdown = None;
                } else {
                    self.active_foreign_key_table_within_dropdown = Some(table_name);
                }
                Task::none()
            }
        }
    }
}

impl TableInfoUI {
    pub fn new(
        table_info: BTableInfo,
        tables_general_info: Option<Vec<BTableGeneralInfo>>,
    ) -> Self {
        Self {
            table_info: table_info.clone(),
            table_name_display: table_info.table_name,
            columns_display: table_info.columns_info,
            tables_general_info,
            active_foreign_key_dropdown_column: None,
            active_foreign_key_table_within_dropdown: None,
        }
    }

    pub fn get_table_name(&self) -> String {
        self.table_info.table_name.clone()
    }

    fn formated_table_change_events(&self) -> String {
        format!("{:?}", self.table_info.get_table_change_events())
    }

    pub fn content<'a>(&'a self) -> Element<'a, Message> {
        let mut table_info_column = Column::new().spacing(20).padding(20);

        table_info_column = table_info_column
            .push(self.build_table_name_input())
            .push(self.build_column_headers())
            .push(self.separator_line())
            .push(self.scrollable_columns_info())
            .push(self.add_column_button())
            .push(self.update_table_button());

        container(table_info_column)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(|_| container_style())
            .into()
    }

    // ============== Smaller Reusable Methods ==============

    fn build_table_name_input(&self) -> TextInput<'_, Message> {
        text_input("📝 Table Name", &self.table_name_display)
            .on_input(|value| TableInfoMessage::UpdateTableName(value).message())
            .size(30)
            .padding(10)
            .width(Length::Fill)
            .style(|_, _| text_input_style())
    }

    fn build_column_headers(&self) -> Row<'_, Message> {
        Row::new()
            .spacing(20)
            .push(
                text("📋 Column Name")
                    .size(20)
                    .color(Color::WHITE)
                    .width(Length::FillPortion(2)),
            )
            .push(
                text("🔧 Data Type")
                    .size(20)
                    .color(Color::WHITE)
                    .width(Length::FillPortion(1)),
            )
    }

    fn separator_line(&self) -> Element<'_, Message> {
        text("------------------------------")
            .color(Color::from_rgb(0.6, 0.6, 0.6))
            .size(16)
            .into()
    }

    fn scrollable_columns_info(&self) -> Element<'_, Message> {
        let columns_info_column = self.build_columns_info();
        scrollable(container(columns_info_column.spacing(10)).padding(10))
            .height(Length::FillPortion(3))
            .into()
    }

    fn build_columns_info(&self) -> Column<'_, Message> {
        self.columns_display.iter().enumerate().fold(
            Column::new().spacing(10),
            |columns_info_column, (index, column_info)| {
                columns_info_column.push(self.build_column_row(index, column_info))
            },
        )
    }

    fn build_column_row<'a>(&'a self, index: usize, column_info: &'a BColumn) -> Row<'a, Message> {
        Row::new()
            .spacing(20)
            .push(self.column_name_input(index, &column_info.name))
            .push(self.data_type_picker(index, &column_info.datatype))
            .push(self.foreign_key_button(index))
            .push(self.remove_column_button(index))
            .width(Length::Fill)
    }

    fn column_name_input<'a>(&'a self, index: usize, name: &str) -> TextInput<'a, Message> {
        text_input("Column Name", name)
            .on_input(move |value| TableInfoMessage::UpdateColumnName(index, value).message())
            .width(Length::FillPortion(2))
            .padding(5)
            .style(|_, _| text_input_style())
    }

    fn data_type_picker<'a>(&'a self, index: usize, datatype: &BDataType) -> Element<'a, Message> {
        PickList::new(
            vec![BDataType::TEXT, BDataType::INTEGER, BDataType::TIMESTAMP],
            Some(datatype.clone()),
            move |value| TableInfoMessage::UpdateColumnType(index, value).message(),
        )
        .width(Length::FillPortion(1))
        .padding(5)
        .into()
    }

    fn foreign_key_button<'a>(&'a self, index: usize) -> Element<'a, Message> {
        let button_text = self.foreign_key_button_text(index);
        let button = button(text(button_text))
            .style(|_, _| button_style())
            .on_press(TableInfoMessage::ToggleForeignKeyDropdown(index).message());

        if self.active_foreign_key_dropdown_column == Some(index) {
            Column::new()
                .push(button)
                .push(self.render_foreign_key_dropdown(index))
                .spacing(5)
                .into()
        } else {
            button.into()
        }
    }

    fn foreign_key_button_text(&self, index: usize) -> String {
        if let Some(column) = self.columns_display.get(index) {
            column
                .constraints
                .iter()
                .find_map(|constraint| {
                    if let BConstraint::ForeignKey(table, column) = constraint {
                        Some(format!("{}.{}", table, column))
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Set Foreign Key".to_string())
        } else {
            "Set Foreign Key".to_string()
        }
    }

    fn render_foreign_key_dropdown<'a>(&'a self, index: usize) -> Element<'a, Message> {
        if let Some(tables) = &self.tables_general_info {
            let dropdown = tables.iter().fold(
                Column::new()
                    .spacing(10)
                    .padding(10)
                    .push(self.remove_foreign_key_button(index)),
                |dropdown, table| dropdown.push(self.foreign_key_table_row(index, table)),
            );

            scrollable(container(dropdown).padding(10).style(|_| dropdown_style()))
                .height(Length::Shrink)
                .width(150)
                .into()
        } else {
            container(text("No Tables Available"))
                .height(Length::Shrink)
                .width(150)
                .style(|_| dropdown_style())
                .into()
        }
    }

    fn foreign_key_table_row<'a>(
        &'a self,
        index: usize,
        table: &'a BTableGeneralInfo,
    ) -> Element<'a, Message> {
        let table_button = button(text(&table.table_name))
            .style(|_, _| table_button_style())
            .on_press(
                TableInfoMessage::ToggleForeignKeyTable(index, table.table_name.clone()).message(),
            );

        if self.active_foreign_key_table_within_dropdown == Some(table.table_name.clone()) {
            Column::new()
                .push(table_button)
                .push(self.column_picklist(index, table))
                .spacing(5)
                .into()
        } else {
            table_button.into()
        }
    }

    fn column_picklist<'a>(
        &'a self,
        index: usize,
        table: &'a BTableGeneralInfo,
    ) -> Element<'a, Message> {
        let options: Vec<String> = zip(&table.column_names, &table.data_types)
            .filter(|(_, datatype)| {
                datatype.to_lowercase()
                    == self.columns_display[index]
                        .datatype
                        .to_string()
                        .to_lowercase()
            })
            .map(|(name, _)| name.clone())
            .collect();

        let selected: Option<String> = None;
        PickList::new(options, selected, move |column| {
            TableInfoMessage::AddForeignKey(index, table.table_name.clone(), column).message()
        })
        .into()
    }

    fn remove_foreign_key_button(&self, index: usize) -> Button<'_, Message> {
        button("Remove Foreign Key")
            .style(|_, _| delete_button_style())
            .on_press(TableInfoMessage::RemoveForeignKey(index).message())
    }

    fn add_column_button(&self) -> Button<'_, Message> {
        button("➕ Add Column")
            .style(|_, _| button_style())
            .padding(10)
            .on_press(TableInfoMessage::AddColumn.message())
    }

    fn remove_column_button<'a>(&'a self, index: usize) -> Button<'a, Message> {
        button("🗑️ Remove")
            .style(|_, _| delete_button_style())
            .padding(10)
            .on_press(TableInfoMessage::RemoveColumn(index).message())
    }

    fn update_table_button(&self) -> Button<'_, Message> {
        button("🛠️ Update Table")
            .style(|_, _| button_style())
            .padding(10)
            .on_press(TableInfoMessage::SubmitUpdateTable.message())
    }
}
fn container_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.12, 0.15, 0.20))), // Darker background for a CRM feel
        border: Border {
            color: Color::from_rgb(0.1, 0.4, 0.6),
            width: 1.5,
            radius: Radius::from(8.0),
        },
        text_color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
        shadow: Shadow {
            color: Color::from_rgb(0.0, 0.0, 0.0),
            offset: Vector::new(0.0, 3.0),
            blur_radius: 7.0,
        },
    }
}

fn button_style() -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb(0.0, 0.6, 0.9))), // CRM blue button
        border: Border {
            color: Color::from_rgb(0.0, 0.4, 0.7),
            width: 2.0,
            radius: Radius::from(5.0),
        },
        text_color: Color::WHITE,
        shadow: Shadow {
            color: Color::from_rgb(0.0, 0.0, 0.0),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 5.0,
        },
    }
}

fn text_input_style() -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::from_rgb(0.18, 0.22, 0.28)), // Darker input background
        border: Border {
            width: 2.0,
            color: Color::from_rgb(0.0, 0.6, 0.9),
            radius: Radius::from(6.0),
        },
        placeholder: Color::from_rgb(0.6, 0.6, 0.6),
        value: Color::WHITE,
        selection: Color::from_rgb(0.0, 0.6, 0.9),
        icon: Color::from_rgb(0.8, 0.8, 0.8),
    }
}

fn constraints_container_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
        border: Border {
            color: Color::from_rgb(0.85, 0.85, 0.85),
            width: 1.0,
            radius: Radius::from(5.0),
        },
        text_color: Some(Color::BLACK),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 2.0,
        },
    }
}
fn table_button_style() -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb(0.2, 0.4, 0.8))), // Blue background
        border: Border {
            color: Color::from_rgb(0.1, 0.3, 0.6), // Darker blue border
            width: 2.0,
            radius: Radius::from(6.0),
        },
        text_color: Color::WHITE, // White text for contrast
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5), // Slight shadow for depth
            offset: Vector::new(0.0, 2.0),
            blur_radius: 10.0,
        },
    }
}

fn column_button_style() -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb(0.4, 0.8, 0.2))), // Green background
        border: Border {
            color: Color::from_rgb(0.3, 0.6, 0.1), // Darker green border
            width: 1.5,
            radius: Radius::from(5.0),
        },
        text_color: Color::BLACK, // Black text for contrast
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3), // Subtle shadow
            offset: Vector::new(0.0, 1.0),
            blur_radius: 5.0,
        },
    }
}

fn dropdown_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))), // Dark background
        border: Border {
            color: Color::from_rgb(0.0, 0.6, 0.6), // Aqua border color
            width: 2.0,
            radius: Radius::from(5.0),
        },
        text_color: Some(Color::WHITE), // White text color
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5), // Slight shadow for depth
            offset: Vector::new(0.0, 2.0),
            blur_radius: 10.0,
        },
    }
}

fn delete_button_style() -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))), // Soft red background
        border: Border {
            color: Color::from_rgb(0.6, 0.1, 0.1), // Dark red border
            width: 2.0,
            radius: Radius::from(5.0),
        },
        text_color: Color::WHITE, // White text for contrast
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::new(0.0, 3.0),
            blur_radius: 5.0,
        },
    }
}
