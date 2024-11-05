use crate::components::business_components::{
    component::{initialize_business_component, BusinessTableOut},
    home::Home,
};
use crate::components::ui_components::{component::UIComponent, events::Message};
use iced::{
    widget::{button, column, container, scrollable, text, text_input, Column, Row, Text},
    Alignment, Element, Length,
};

#[derive(Debug, Clone)]
pub struct HomeUI {
    pub home: Home,
}

impl UIComponent for HomeUI {
    async fn initialize_component(&mut self) {
        let home_business_component =
            initialize_business_component::<Home>(self.home.clone()).await;
        self.home = home_business_component;
    }
}

impl HomeUI {
    pub fn new(home: Home) -> Self {
        Self { home }
    }

    fn tables<'a>(&'a self) -> Element<'a, Message> {
        let tables_container = if let Some(tables) = &self.home.tables {
            let mut tables_column = Column::new()
                .height(Length::Fill)
                .width(Length::Fill)
                .padding(10);

            for table in tables {
                tables_column = tables_column.push(text(&table.table_name));
            }
            container(tables_column).height(250).width(300) // Wrap tables_column here
        } else {
            container(text("Loading"))
                .height(Length::Fill)
                .width(Length::Fill)
                .padding(10)
        };

        let mut tables_display = Column::new();
        tables_display = tables_display.push(tables_container); // Populate tables_display with tables_container

        container(tables_display).into()
    }

    fn title<'a>(&'a self) -> Element<'a, Message> {
        if let Some(title) = &self.home.title {
            container(text(title)).into()
        } else {
            container(text("Loading")).into()
        }
    }

    pub fn content<'a>(&'a self) -> Element<'a, Message> {
        let mut row = Row::new();
        row = row.push(self.tables());
        row = row.push(self.title());
        container(row).into()
    }
}