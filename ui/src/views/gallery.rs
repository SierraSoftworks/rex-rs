//! A specimen page for every control, in debug builds only.
//!
//! This is the review surface for the control library: it renders each control
//! in its states side by side, which is how visual drift gets caught before it
//! reaches a real screen. The standing convention is simple -- **add a specimen
//! when you add or change a control**. An unregistered stylesheet shows up here
//! immediately as an unstyled specimen.

use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    app::Route,
    components::{
        Icon, IdeaControls, IdeaDisplay, MarkdownView, TagEditor,
        controls::{
            Avatar, Button, ButtonGroup, ButtonKind, Form, FormField, IconButton, Select, Spinner,
            Table, TableColumn, Tag, TagKind, TextArea, TextInput, Tooltip,
        },
        use_notifier,
    },
};

/// The specimens, in the order they appear in the sidebar.
const CONTROLS: &[&str] = &[
    "button",
    "tag",
    "tag-editor",
    "text-input",
    "select",
    "form",
    "table",
    "avatar",
    "spinner",
    "tooltip",
    "notification",
    "markdown",
    "idea",
];

#[derive(Properties, PartialEq)]
pub struct GalleryProps {
    /// `None` shows every specimen at once.
    #[prop_or_default]
    pub control: Option<String>,
}

#[function_component(Gallery)]
pub fn gallery(props: &GalleryProps) -> Html {
    let selected = props.control.clone();

    let shown: Vec<&str> = match selected.as_deref() {
        Some(name) => CONTROLS.iter().copied().filter(|c| *c == name).collect(),
        None => CONTROLS.to_vec(),
    };

    html! {
        <div class="gallery">
            <nav class="gallery__nav">
                <Link<Route> to={Route::Gallery}>{ "All" }</Link<Route>>
                { for CONTROLS.iter().map(|control| html! {
                    <Link<Route> key={*control} to={Route::GalleryControl { control: control.to_string() }}>
                        { *control }
                    </Link<Route>>
                }) }
            </nav>

            <div class="gallery__specimens">
                if shown.is_empty() {
                    <p class="muted">{ format!("No specimen is registered for {selected:?}.") }</p>
                }

                { for shown.into_iter().map(specimen) }
            </div>
        </div>
    }
}

fn specimen(name: &str) -> Html {
    html! {
        <section class="gallery__specimen" key={name.to_string()}>
            <h3>{ name }</h3>
            <div class="gallery__stage">{ body(name) }</div>
        </section>
    }
}

fn body(name: &str) -> Html {
    match name {
        "button" => html! { <ButtonSpecimen /> },
        "tag" => html! {
            <>
                <Tag>{ "rust" }</Tag>
                <Tag kind={TagKind::Success}>{ "Done" }</Tag>
                <Tag onclose={Callback::noop()}>{ "closable" }</Tag>
            </>
        },
        "tag-editor" => html! { <TagEditorSpecimen /> },
        "text-input" => html! { <TextInputSpecimen /> },
        "select" => html! { <SelectSpecimen /> },
        "form" => html! {
            <Form>
                <FormField label="Name">
                    <TextInput value="" placeholder="Give your idea a name" />
                </FormField>
                <FormField label="Description" hint="Markdown is supported.">
                    <TextArea value="" placeholder="Describe the idea" />
                </FormField>
            </Form>
        },
        "table" => html! {
            <Table
                columns={vec![
                    TableColumn::new("Name"),
                    TableColumn::new("Tags"),
                    TableColumn::fixed("", "140px"),
                ]}
                row_count={2}>
                <tr>
                    <td>{ "Learn to sail" }</td>
                    <td><Tag>{ "outdoors" }</Tag></td>
                    <td>
                        <IconButton icon={Icon::Check} label="Mark as done" />
                        <IconButton icon={Icon::Delete} kind={ButtonKind::Danger} label="Delete" />
                    </td>
                </tr>
                <tr class="success-row">
                    <td>{ "Bake sourdough" }</td>
                    <td><Tag>{ "cooking" }</Tag></td>
                    <td>
                        <IconButton icon={Icon::Undo} label="Mark as not done" />
                        <IconButton icon={Icon::Delete} kind={ButtonKind::Danger} label="Delete" />
                    </td>
                </tr>
            </Table>
        },
        "avatar" => html! {
            <>
                <Avatar size={50} alt="Nobody in particular" />
                <Avatar size={32} alt="Nobody in particular" />
                <Avatar size={16} alt="Nobody in particular" />
            </>
        },
        "spinner" => html! { <Spinner label="Finding you something to do" /> },
        "tooltip" => html! {
            <Tooltip content="New Idea">
                <Button icon={Icon::Plus}>{ "Hover me" }</Button>
            </Tooltip>
        },
        "notification" => html! { <NotificationSpecimen /> },
        "markdown" => html! {
            <MarkdownView
                class={classes!("markdown")}
                value={"A description with **bold**, `inline code`, and a list:\n\n- one\n- two\n"} />
        },
        "idea" => html! {
            <>
                <IdeaDisplay idea={sample_idea()} />
                <IdeaControls
                    idea={sample_idea()}
                    allow_complete={true}
                    allow_next={true}
                    allow_delete={true} />
            </>
        },
        other => html! { <p class="muted">{ format!("No specimen for {other}.") }</p> },
    }
}

fn sample_idea() -> rex_api::IdeaV3 {
    rex_api::IdeaV3 {
        id: Some("00000000000000000000000000000001".into()),
        collection: Some("00000000000000000000000000000000".into()),
        name: "Learn to sail".into(),
        description: "Find a club on the coast and book an introductory weekend.".into(),
        tags: Some(
            ["outdoors", "someday"]
                .iter()
                .map(|t| t.to_string())
                .collect(),
        ),
        completed: Some(false),
    }
}

#[function_component(ButtonSpecimen)]
fn button_specimen() -> Html {
    html! {
        <>
            <div>
                <Button>{ "Default" }</Button>
                <Button kind={ButtonKind::Primary}>{ "Primary" }</Button>
                <Button kind={ButtonKind::Danger}>{ "Danger" }</Button>
                <Button disabled={true}>{ "Disabled" }</Button>
            </div>
            <div>
                <Button icon={Icon::Check}>{ "With an icon" }</Button>
                <Button small={true}>{ "Small" }</Button>
                <IconButton icon={Icon::Delete} label="Delete" />
            </div>
            <ButtonGroup>
                <Button kind={ButtonKind::Primary} icon={Icon::Document}>{ "Open" }</Button>
                <Button icon={Icon::DocumentAdd}>{ "New Idea" }</Button>
                <Button icon={Icon::DocumentChecked}>{ "Manage" }</Button>
            </ButtonGroup>
        </>
    }
}

#[function_component(TagEditorSpecimen)]
fn tag_editor_specimen() -> Html {
    let tags = use_state(|| vec![String::from("outdoors"), String::from("someday")]);

    html! {
        <TagEditor
            tags={(*tags).clone()}
            onchange={{
                let tags = tags.clone();
                Callback::from(move |updated| tags.set(updated))
            }} />
    }
}

#[function_component(TextInputSpecimen)]
fn text_input_specimen() -> Html {
    let value = use_state(String::new);

    html! {
        <>
            <TextInput
                value={(*value).clone()}
                placeholder="Type something"
                onchange={{
                    let value = value.clone();
                    Callback::from(move |next| value.set(next))
                }} />
            <TextArea value={(*value).clone()} placeholder="And something longer" />
        </>
    }
}

#[function_component(SelectSpecimen)]
fn select_specimen() -> Html {
    let value = use_state(|| String::from("Viewer"));

    html! {
        <Select
            value={(*value).clone()}
            options={vec![AttrValue::from("Owner"), AttrValue::from("Contributor"), AttrValue::from("Viewer")]}
            onchange={{
                let value = value.clone();
                Callback::from(move |next| value.set(next))
            }} />
    }
}

#[function_component(NotificationSpecimen)]
fn notification_specimen() -> Html {
    let notifier = use_notifier();

    let error = {
        let notifier = notifier.clone();
        Callback::from(move |_: MouseEvent| {
            notifier.message("403 Forbidden", "You do not have permission to do that.")
        })
    };

    let success = {
        let notifier = notifier.clone();
        Callback::from(move |_: MouseEvent| notifier.success("That worked."))
    };

    html! {
        <>
            <Button onclick={error}>{ "Raise an error" }</Button>
            <Button onclick={success}>{ "Raise a success" }</Button>
        </>
    }
}
