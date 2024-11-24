use crate::components::business_components::component::{
    BColumn, BConstraint, BDataType, BTableGeneralInfo, BTableIn, BusinessComponent,
};
use crate::components::ui_components::{
    component::{Event, UIComponent},
    events::Message,
    tables::events::CreateTableFormMessage,
};
use iced::{
    alignment,
    alignment::{Alignment, Vertical},
    border::Radius,
    widget::{
        button, checkbox, column, container, row, scrollable, text, text_input, Button, Checkbox,
        Column, PickList, Row, Text,
    },
    Background, Border, Color, Element, Length, Shadow, Task, Theme, Vector,
};
use std::iter::zip;

#[derive(Debug, Clone)]
pub struct CreateTableFormUI {
    create_table_input: BTableIn,
    pub tables_general_info: Option<Vec<BTableGeneralInfo>>,
    active_foreign_key_table_within_dropdown: Option<String>, // table in foreign key dropdown that has its columns displayed
    active_foreign_key_dropdown_column: Option<usize>, // column index that wants the foreign key dropdown
                                                       // activated
}

impl UIComponent for CreateTableFormUI {
    type EventType = CreateTableFormMessage;

    fn update(&mut self, message: Self::EventType) -> Task<Message> {
        match message {
            Self::EventType::AddColumn => {
                self.create_table_input.columns.push(BColumn::default());
                Task::none()
            }
            Self::EventType::RemoveColumn(index) => {
                if index < self.create_table_input.columns.len() {
                    self.create_table_input.columns.remove(index);
                }
                Task::none()
            }
            Self::EventType::UpdateColumnName(index, input) => {
                if let Some(column) = self.create_table_input.columns.get_mut(index) {
                    column.name = input;
                }
                Task::none()
            }
            Self::EventType::UpdateColumnType(index, input) => {
                if let Some(column) = self.create_table_input.columns.get_mut(index) {
                    column.datatype = input;
                }
                Task::none()
            }
            Self::EventType::SetOrRemovePrimaryKey(index) => {
                if let Some(column) = self.create_table_input.columns.get_mut(index) {
                    if let Some(existing_index) = column
                        .constraints
                        .iter()
                        .position(|constraint| matches!(constraint, BConstraint::PrimaryKey))
                    {
                        column.constraints.remove(existing_index);
                    } else {
                        column.constraints.push(BConstraint::PrimaryKey);
                    }
                }
                Task::none()
            }
            Self::EventType::AddForeignKey(
                index,
                referenced_table_name,
                referenced_column_name,
            ) => {
                if let Some(column) = self.create_table_input.columns.get_mut(index) {
                    if let Some(existing_index) = column.constraints.iter().position(|constraint| {
                        matches!(
                            constraint,
                            BConstraint::ForeignKey(existing_table_name, existing_column_name)
                        )
                    }) {
                        // Remove the foreign key constraint if it exists
                        column.constraints.remove(existing_index);
                        column.constraints.push(BConstraint::ForeignKey(
                            referenced_table_name,
                            referenced_column_name,
                        ));
                    } else {
                        // Add the foreign key constraint if it does not exist
                        column.constraints.push(BConstraint::ForeignKey(
                            referenced_table_name,
                            referenced_column_name,
                        ));
                    }
                }

                self.active_foreign_key_dropdown_column = None;
                self.active_foreign_key_table_within_dropdown = None;
                Task::none()
            }
            Self::EventType::RemoveForeignKey(index) => {
                if let Some(column) = self.create_table_input.columns.get_mut(index) {
                    if let Some(existing_index) = column.constraints.iter().position(|constraint| {
                        matches!(
                            constraint,
                            BConstraint::ForeignKey(existing_table_name, existing_column_name)
                        )
                    }) {
                        column.constraints.remove(existing_index);
                    }
                }
                self.active_foreign_key_dropdown_column = None;
                self.active_foreign_key_table_within_dropdown = None;

                Task::none()
            }
            Self::EventType::UpdateTableName(input) => {
                self.create_table_input.table_name = input;
                Task::none()
            }
            Self::EventType::TableCreated(tables, table_name) => {
                self.create_table_input = BTableIn::default();
                Task::none()
            }
            Self::EventType::SubmitCreateTable(create_table_input) => Task::none(),
            Self::EventType::ShowOrRemoveCreateTableForm => {
                if self.create_table_input.columns.len() == 0 {
                    for _ in 0..1 {
                        self.create_table_input.columns.push(BColumn {
                            name: String::from("id"),
                            datatype: BDataType::INTEGER,
                            constraints: vec![BConstraint::PrimaryKey],
                        });
                    }
                }
                Task::none()
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

impl CreateTableFormUI {
    pub fn new(tables_general_info: Option<Vec<BTableGeneralInfo>>) -> Self {
        Self {
            create_table_input: BTableIn::default(),
            tables_general_info,
            active_foreign_key_dropdown_column: None,
            active_foreign_key_table_within_dropdown: None,
        }
    }

    // ======================== SECTION: Create Table ========================

    pub fn content<'a>(&'a self) -> Element<'a, Message> {
        let mut create_form = Column::new().spacing(20).padding(20);
        create_form = create_form.push(self.create_table_form());

        container(create_form)
            .padding(20)
            .style(|_| container_style())
            .into()
    }

    fn create_table_form<'a>(&'a self) -> Element<'a, Message> {
        let mut form = Column::new().spacing(15).padding(15);
        form = form.push(self.table_name_input());
        form = form.push(self.table_form_columns());

        let add_column_button = button("➕ Add Column")
            .style(|_, _| button_style())
            .on_press(<CreateTableFormUI as UIComponent>::EventType::AddColumn.message())
            .padding(10);
        form = form.push(add_column_button);

        let create_table_button = button("🛠️ Create Table")
            .style(|_, _| create_button_style())
            .on_press(<CreateTableFormUI as UIComponent>::EventType::message(
                <CreateTableFormUI as UIComponent>::EventType::SubmitCreateTable(
                    self.create_table_input.clone(),
                ),
            ))
            .padding(15);

        form.push(
            Row::new()
                .push(
                    container(create_table_button)
                        .width(Length::Fill)
                        .align_x(alignment::Horizontal::Center), // Center the button horizontally
                )
                .width(Length::Fill),
        )
        .into()
    }

    fn table_name_input<'a>(&'a self) -> Element<'a, Message> {
        text_input("Enter Table Name", &self.create_table_input.table_name)
            .on_input(|value| {
                <CreateTableFormUI as UIComponent>::EventType::message(
                    <CreateTableFormUI as UIComponent>::EventType::UpdateTableName(value),
                )
            })
            .width(Length::Fill)
            .padding(10)
            .style(|_, _| text_input_style())
            .into()
    }

    fn table_form_columns<'a>(&'a self) -> Element<'a, Message> {
        let mut columns_list = Column::new().spacing(10);
        for (index, column) in self.create_table_input.columns.iter().enumerate() {
            columns_list = columns_list.push(self.column_input_row(index, column));
        }
        scrollable(columns_list)
            .height(Length::Fill)
            .direction(scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::new(),
                horizontal: scrollable::Scrollbar::new(),
            })
            .into()
    }

    fn column_input_row<'a>(&'a self, index: usize, column: &'a BColumn) -> Element<'a, Message> {
        // Column name input
        let name_input = text_input("Column Name", &column.name)
            .on_input(move |value| {
                <CreateTableFormUI as UIComponent>::EventType::message(
                    <CreateTableFormUI as UIComponent>::EventType::UpdateColumnName(index, value),
                )
            })
            .width(200)
            .style(|_, _| text_input_style());

        // Data type picker
        let datatype_input = PickList::new(
            vec![BDataType::TEXT, BDataType::INTEGER, BDataType::TIMESTAMP],
            Some(&column.datatype),
            move |value| {
                <CreateTableFormUI as UIComponent>::EventType::message(
                    <CreateTableFormUI as UIComponent>::EventType::UpdateColumnType(index, value),
                )
            },
        )
        .width(150);

        // Primary key checkbox
        let primary_key_checkbox = checkbox(
            "Primary Key",
            column.constraints.contains(&BConstraint::PrimaryKey),
        )
        .on_toggle(move |_| {
            <CreateTableFormUI as UIComponent>::EventType::message(
                <CreateTableFormUI as UIComponent>::EventType::SetOrRemovePrimaryKey(index),
            )
        });

        // Foreign key dropdown
        let foreign_key_dropdown = self.render_foreign_key_button(index);
        let remove_button = button("Remove")
            .style(|_, _| delete_button_style())
            .on_press(<CreateTableFormUI as UIComponent>::EventType::message(
                <CreateTableFormUI as UIComponent>::EventType::RemoveColumn(index),
            ))
            .padding(10);

        // Construct the row layout
        row![
            name_input,
            datatype_input,
            primary_key_checkbox,
            foreign_key_dropdown,
            remove_button
        ]
        .spacing(10)
        .align_y(Vertical::Center)
        .into()
    }
    fn render_foreign_key_button<'a>(&'a self, index: usize) -> Element<'a, Message> {
        // Button to show the foreign key tables
        let button_text = if let Some(column_info) = self.create_table_input.columns.get(index) {
            if let Some(foreign_key_constraint) = column_info
                .constraints
                .iter()
                .find(|constraint| matches!(constraint, BConstraint::ForeignKey(_, _)))
            {
                if let BConstraint::ForeignKey(referenced_table_name, referenced_column_name) =
                    foreign_key_constraint
                {
                    text(format!(
                        "{}.{}",
                        referenced_table_name, referenced_column_name
                    ))
                } else {
                    text("Set Foreign Key")
                }
            } else {
                text("Set Foreign Key")
            }
        } else {
            text("Set Foreign Key")
        };
        let button = button(button_text).style(|_, _| button_style()).on_press(
            <CreateTableFormUI as UIComponent>::EventType::message(
                <CreateTableFormUI as UIComponent>::EventType::ToggleForeignKeyDropdown(index),
            ),
        );

        // Check if the current column's foreign key dropdown is active
        if self.active_foreign_key_dropdown_column == Some(index) {
            // Render the foreign key dropdown
            let foreign_key_dropdown = self.render_foreign_key_dropdown(index);
            Column::new()
                .push(button)
                .push(foreign_key_dropdown)
                .spacing(5)
                .into()
        } else {
            // Render just the button
            button.into()
        }
    }
    fn render_foreign_key_dropdown<'a>(&'a self, index: usize) -> Element<'a, Message> {
        if let Some(tables) = &self.tables_general_info {
            // Initialize a column for the dropdown
            let mut dropdown = Column::new().spacing(10).padding(10);
            let remove_foreign_key_button = button(text("Remove"))
                .style(|_, _| delete_button_style())
                .on_press(<CreateTableFormUI as UIComponent>::EventType::message(
                    <CreateTableFormUI as UIComponent>::EventType::RemoveForeignKey(index),
                ));
            dropdown = dropdown.push(remove_foreign_key_button);

            for table in tables {
                let table_name = table.table_name.clone();

                // Create a button for the table name
                let table_button = button(text(table_name.clone()))
                    .style(|_, _| table_button_style())
                    .on_press(<CreateTableFormUI as UIComponent>::EventType::message(
                        <CreateTableFormUI as UIComponent>::EventType::ToggleForeignKeyTable(
                            index,
                            table_name.clone(),
                        ),
                    ));

                // Check if this table is expanded
                let expanded_table = if matches!(self.active_foreign_key_table_within_dropdown, Some(ref name) if name == &table_name)
                {
                    // Create a PickList for the columns in the table
                    let selected: Option<String> = None;
                    let column_names_to_reference_by_datatype: Vec<String> =
                        zip(table.column_names.clone(), table.data_types.clone())
                            .filter(|(column_name, data_type)| {
                                *data_type.to_lowercase()
                                    == self.create_table_input.columns[index]
                                        .datatype
                                        .to_string()
                                        .to_lowercase()
                            })
                            .map(|(column_name, data_type)| column_name)
                            .collect();
                    let column_picklist = PickList::new(
                        column_names_to_reference_by_datatype,
                        selected,
                        move |column_name| {
                            <CreateTableFormUI as UIComponent>::EventType::message(
                                <CreateTableFormUI as UIComponent>::EventType::AddForeignKey(
                                    index,
                                    table_name.clone(),
                                    column_name,
                                ),
                            )
                        },
                    )
                    .width(150);

                    // Combine table button and column picklist in a column
                    Column::new()
                        .push(table_button)
                        .push(column_picklist)
                        .spacing(5)
                } else {
                    // Only show the table button if not expanded
                    Column::new().push(table_button)
                };

                // Add the expanded or non-expanded table to the dropdown
                dropdown = dropdown.push(expanded_table);
            }

            // Wrap the dropdown in a scrollable container
            scrollable(container(dropdown.padding(10)).style(|_| dropdown_style()))
                .height(Length::Shrink)
                .width(150)
                .into()
        } else {
            // If no tables are available, show a placeholder
            container(text("No tables available"))
                .height(Length::Shrink)
                .width(Length::FillPortion(2))
                .style(|_| dropdown_style())
                .into()
        }
    }
}

// ======================== STYLES ========================
fn container_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))), // Background color
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.5,
            radius: Radius::from(5.0),
        },
        text_color: Some(Color::WHITE), // Text color for the content inside the container
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::new(0.0, 2.0),
            blur_radius: 5.0,
        },
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
fn button_style() -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb(0.0, 0.75, 0.65))),
        border: Border {
            color: Color::from_rgb(0.0, 0.6, 0.5),
            width: 2.0,
            radius: Radius::from(5.0),
        },
        text_color: Color::WHITE,
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::new(0.0, 3.0),
            blur_radius: 5.0,
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

fn create_button_style() -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgb(0.0, 0.5, 0.9))),
        border: Border {
            color: Color::from_rgb(0.0, 0.4, 0.7),
            width: 2.0,
            radius: Radius::from(8.0),
        },
        text_color: Color::WHITE,
        shadow: Shadow {
            color: Color::BLACK,
            offset: Vector::new(0.0, 3.0),
            blur_radius: 7.0,
        },
    }
}

fn text_input_style() -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::from_rgb(0.2, 0.2, 0.2)), // Darker input background
        border: Border {
            width: 1.5,
            color: Color::from_rgb(0.0, 0.74, 0.84),
            radius: Radius::from(5.0),
        },
        placeholder: Color::from_rgb(0.6, 0.6, 0.6), // Color for placeholder text
        value: Color::WHITE,                         // Color for input text
        selection: Color::from_rgb(0.0, 0.74, 0.84), // Color for selected text
        icon: Color::from_rgb(0.8, 0.8, 0.8),        // Color for any input icons
    }
}
