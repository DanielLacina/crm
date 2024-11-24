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
        Column, PickList, Row, Text, TextInput,
    },
    Background, Border, Color, Element, Length, Shadow, Task, Theme, Vector,
};
use std::iter::zip;
use std::sync::Arc;

pub trait ForeignKeyDropdownEvents {
    fn add_foreign_key(
        index: usize,
        referenced_table_name: String,
        referenced_column_name: String,
    ) -> Message;
    fn remove_foreign_key(index: usize) -> Message;
    fn toggle_foreign_key_table(index: usize, table_name: String) -> Message;
}

#[derive(Debug, Clone)]
pub struct ForeignKeyDropDownUI {
    pub tables_general_info: Option<Vec<BTableGeneralInfo>>,
    pub active_foreign_key_table_within_dropdown: Option<String>,
    pub column: BColumn,
    pub events: Arc<dyn ForeignKeyDropdownEvents>,
    pub index: usize,
}

impl ForeignKeyDropDownUI {
    pub fn new(
        column: BColumn,
        tables_general_info: Option<Vec<BTableGeneralInfo>>,
        events: Arc<dyn ForeignKeyDropdownEvents>,
        active_foreign_key_table_within_dropdown: Option<String>,
        index: usize,
    ) -> Self {
        Self {
            column,
            tables_general_info,
            events,
            active_foreign_key_table_within_dropdown,
            index,
        }
    }

    pub fn content<'a>(&'a self) -> Element<'a, Message> {
        if let Some(tables) = &self.tables_general_info {
            let dropdown = tables.iter().fold(
                Column::new()
                    .spacing(10)
                    .padding(10)
                    .push(self.remove_foreign_key_button()),
                |dropdown, table| dropdown.push(self.foreign_key_table_row(table)),
            );

            scrollable(container(dropdown).padding(10).style(|_| dropdown_style()))
                .height(Length::Shrink)
                .width(150)
                .into()
        } else {
            container(text("No tables available"))
                .height(Length::Shrink)
                .width(Length::FillPortion(2))
                .style(|_| dropdown_style())
                .into()
        }
    }

    fn foreign_key_table_row<'a>(&'a self, table: &'a BTableGeneralInfo) -> Element<'a, Message> {
        let table_button = button(text(&table.table_name))
            .style(|_, _| table_button_style())
            .on_press(
                self.events
                    .toggle_foreign_key_table(table.table_name.clone()),
            );

        if self.active_foreign_key_table_within_dropdown == Some(table.table_name.clone()) {
            Column::new()
                .push(table_button)
                .push(self.foreign_key_column_picklist(table))
                .spacing(5)
                .into()
        } else {
            table_button.into()
        }
    }

    fn foreign_key_column_picklist<'a>(
        &'a self,
        table: &'a BTableGeneralInfo,
    ) -> Element<'a, Message> {
        let options: Vec<String> = zip(&table.column_names, &table.data_types)
            .filter(|(_, datatype)| {
                datatype.to_lowercase() == self.column.datatype.to_string().to_lowercase()
            })
            .map(|(name, _)| name.clone())
            .collect();
        let selected: Option<String> = None;
        PickList::new(options, selected, move |column_name| {
            self.events
                .add_foreign_key(table.table_name.clone(), column_name.clone())
        })
        .into()
    }

    fn remove_foreign_key_button(&self) -> Button<'_, Message> {
        button("Remove Foreign Key")
            .style(|_, _| delete_button_style())
            .on_press(self.events.remove_foreign_key())
    }
}

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
