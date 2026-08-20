use rex_api::{CollectionV3, IdeaV3, RoleAssignmentV3};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    api,
    app::Route,
    components::{
        Icon, Protected,
        controls::{ButtonKind, IconButton, Spinner, Table, TableColumn, Tag},
        use_notifier,
    },
};

#[derive(Properties, PartialEq)]
pub struct ManageProps {
    pub collection: String,
}

/// Everything in one collection: its ideas, and who can see them.
#[function_component(Manage)]
pub fn manage(props: &ManageProps) -> Html {
    html! {
        <Protected>
            <ManageContent collection={props.collection.clone()} />
        </Protected>
    }
}

#[function_component(ManageContent)]
fn manage_content(props: &ManageProps) -> Html {
    let notifier = use_notifier();
    let navigator = use_navigator().expect("a router");

    let collection = use_state(|| None::<CollectionV3>);
    let ideas = use_state(Vec::<IdeaV3>::new);
    let members = use_state(Vec::<RoleAssignmentV3>::new);
    let loading = use_state(|| true);

    {
        let collection = collection.clone();
        let ideas = ideas.clone();
        let members = members.clone();
        let loading = loading.clone();
        let notifier = notifier.clone();
        let cid = props.collection.clone();

        use_effect_with(cid.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::collection(&cid).await {
                    Ok(loaded) => collection.set(Some(loaded)),
                    Err(err) => notifier.error(&err),
                }

                match api::ideas(&cid).await {
                    Ok(loaded) => ideas.set(loaded),
                    Err(err) => notifier.error(&err),
                }

                // Only an owner may list the members, so a plain 403 here is
                // the expected answer for everybody else rather than a fault.
                match api::role_assignments(&cid).await {
                    Ok(loaded) => members.set(loaded),
                    Err(err) if err.code == 403 => members.set(Vec::new()),
                    Err(err) => notifier.error(&err),
                }

                loading.set(false);
            });
        });
    }

    let set_completed = {
        let ideas = ideas.clone();
        let notifier = notifier.clone();

        Callback::from(move |(id, completed): (String, bool)| {
            let Some(current) = ideas.iter().find(|i| i.id.as_deref() == Some(&id)).cloned() else {
                return;
            };

            let ideas = ideas.clone();
            let notifier = notifier.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let updated = IdeaV3 {
                    completed: Some(completed),
                    ..current
                };

                match api::store_idea(&updated).await {
                    Ok(saved) => {
                        let replaced = ideas
                            .iter()
                            .map(|idea| {
                                if idea.id == saved.id {
                                    saved.clone()
                                } else {
                                    idea.clone()
                                }
                            })
                            .collect();

                        ideas.set(replaced);
                    }
                    Err(err) => notifier.error(&err),
                }
            });
        })
    };

    let delete_idea = {
        let ideas = ideas.clone();
        let notifier = notifier.clone();
        let cid = props.collection.clone();

        Callback::from(move |id: String| {
            let ideas = ideas.clone();
            let notifier = notifier.clone();
            let cid = cid.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match api::remove_idea(&cid, &id).await {
                    Ok(()) => {
                        let remaining = ideas
                            .iter()
                            .filter(|idea| idea.id.as_deref() != Some(id.as_str()))
                            .cloned()
                            .collect();

                        ideas.set(remaining);
                    }
                    Err(err) => notifier.error(&err),
                }
            });
        })
    };

    let remove_member = {
        let members = members.clone();
        let notifier = notifier.clone();
        let cid = props.collection.clone();

        Callback::from(move |user: String| {
            let members = members.clone();
            let notifier = notifier.clone();
            let cid = cid.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match api::remove_role_assignment(&cid, &user).await {
                    Ok(()) => {
                        let remaining = members
                            .iter()
                            .filter(|member| member.user_id.as_deref() != Some(user.as_str()))
                            .cloned()
                            .collect();

                        members.set(remaining);
                    }
                    Err(err) => notifier.error(&err),
                }
            });
        })
    };

    if *loading {
        return html! {
            <div class="fill">
                <div class="center">
                    <Spinner label="Loading the collection" />
                </div>
            </div>
        };
    }

    let Some(loaded) = (*collection).clone() else {
        return html! {
            <div class="fill">
                <div class="center">
                    { "Oops, it looks like the collection you're looking for could not be found." }
                </div>
            </div>
        };
    };

    let invite = {
        let navigator = navigator.clone();
        let cid = props.collection.clone();

        Callback::from(move |_: MouseEvent| navigator.push(&Route::Invite { cid: cid.clone() }))
    };

    html! {
        <section class="manage">
            <h1 class="manage__name">{ &loaded.name }</h1>

            <div class="manage__columns">
                <div class="manage__ideas">
                    <h3>{ "Ideas" }</h3>

                    <Table
                        columns={vec![
                            TableColumn::new("Name"),
                            TableColumn::new("Tags"),
                            TableColumn::fixed("", "140px"),
                        ]}
                        row_count={ideas.len()}
                        empty="This collection has no ideas yet.">
                        { for ideas.iter().map(|idea| {
                            let id = idea.id.clone().unwrap_or_default();
                            let completed = idea.completed.unwrap_or(false);

                            let mut tags: Vec<String> =
                                idea.tags.clone().unwrap_or_default().into_iter().collect();
                            tags.sort();

                            let toggle = {
                                let set_completed = set_completed.clone();
                                let id = id.clone();
                                Callback::from(move |_: MouseEvent| {
                                    set_completed.emit((id.clone(), !completed))
                                })
                            };

                            let remove = {
                                let delete_idea = delete_idea.clone();
                                let id = id.clone();
                                Callback::from(move |_: MouseEvent| delete_idea.emit(id.clone()))
                            };

                            html! {
                                <tr key={id.clone()} class={classes!(completed.then_some("success-row"))}>
                                    <td>
                                        <Link<Route> to={Route::Idea {
                                            cid: loaded.id.clone().unwrap_or_default(),
                                            iid: id.clone(),
                                        }}>
                                            { &idea.name }
                                        </Link<Route>>
                                    </td>
                                    <td>
                                        { for tags.into_iter().map(|tag| html! {
                                            <Tag key={tag.clone()}>{ Html::from(tag.clone()) }</Tag>
                                        }) }
                                    </td>
                                    <td class="manage__actions">
                                        <IconButton
                                            icon={if completed { Icon::Undo } else { Icon::Check }}
                                            label={if completed { "Mark as not done" } else { "Mark as done" }}
                                            onclick={toggle} />
                                        <IconButton
                                            icon={Icon::Delete}
                                            kind={ButtonKind::Danger}
                                            label="Delete idea"
                                            onclick={remove} />
                                    </td>
                                </tr>
                            }
                        }) }
                    </Table>
                </div>

                <div class="manage__members">
                    <h3>
                        { "Users" }
                        <IconButton
                            icon={Icon::Plus}
                            kind={ButtonKind::Primary}
                            label="Invite someone"
                            onclick={invite} />
                    </h3>

                    <Table
                        columns={vec![
                            TableColumn::new("User"),
                            TableColumn::new("Role"),
                            TableColumn::fixed("", "80px"),
                        ]}
                        row_count={members.len()}
                        empty="Only you can see this collection.">
                        { for members.iter().map(|member| {
                            let user = member.user_id.clone().unwrap_or_default();

                            let remove = {
                                let remove_member = remove_member.clone();
                                let user = user.clone();
                                Callback::from(move |_: MouseEvent| remove_member.emit(user.clone()))
                            };

                            html! {
                                <tr key={user.clone()}>
                                    <td class="manage__user-id">{ &user }</td>
                                    <td>{ &member.role }</td>
                                    <td class="manage__actions">
                                        <IconButton
                                            icon={Icon::Delete}
                                            kind={ButtonKind::Danger}
                                            label="Remove from collection"
                                            onclick={remove} />
                                    </td>
                                </tr>
                            }
                        }) }
                    </Table>
                </div>
            </div>
        </section>
    }
}
