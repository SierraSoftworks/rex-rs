use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    pub label: AttrValue,
    /// A fixed width, for the column of row actions.
    pub width: Option<AttrValue>,
}

impl TableColumn {
    pub fn new(label: impl Into<AttrValue>) -> Self {
        Self {
            label: label.into(),
            width: None,
        }
    }

    pub fn fixed(label: impl Into<AttrValue>, width: impl Into<AttrValue>) -> Self {
        Self {
            label: label.into(),
            width: Some(width.into()),
        }
    }
}

/// A table that only knows about its header.
///
/// Rows are supplied as children, so each view lays out its own cells rather
/// than squeezing them through a generic row abstraction that would have to
/// grow a special case for every column.
#[derive(Properties, PartialEq)]
pub struct TableProps {
    pub columns: Vec<TableColumn>,
    /// Rendered in place of the rows when there are none.
    #[prop_or_default]
    pub empty: Option<AttrValue>,
    #[prop_or_default]
    pub row_count: usize,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Table)]
pub fn table(props: &TableProps) -> Html {
    html! {
        <table class="table">
            <thead>
                <tr>
                    { for props.columns.iter().map(|column| html! {
                        <th style={column.width.as_ref().map(|w| format!("width: {w};"))}>
                            { &column.label }
                        </th>
                    }) }
                </tr>
            </thead>
            <tbody>
                if props.row_count == 0 && props.empty.is_some() {
                    <tr class="table__empty">
                        <td colspan={props.columns.len().to_string()}>
                            { props.empty.clone().unwrap_or_default() }
                        </td>
                    </tr>
                } else {
                    { for props.children.iter() }
                }
            </tbody>
        </table>
    }
}
