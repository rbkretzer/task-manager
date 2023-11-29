use dioxus::{ html::*, prelude::* };

use super::app_props::AppProps;

#[derive(Clone)]
pub struct Sort {
    pub field: String,
    pub sort_type: SortType,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum SortType {
    Asc,
    Desc,
    Unset,
}

impl Sort {
    pub(crate) fn sorting(self, cx: Scope<AppProps>, to_sort: String) -> Element {
        if self.field.ne(to_sort.as_str()) {
            return render! {span {}};
        }
        let arrow = match self.sort_type {
            SortType::Asc => "arrow_upward",
            SortType::Desc => "arrow_downward",
            SortType::Unset => "",
        };
        render! {
            span {class: "material-icons md-14", arrow }
        }
    }
}