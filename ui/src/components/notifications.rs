use std::rc::Rc;

use rex_api::ApiError;
use yew::prelude::*;

/// A message shown to the user, usually because something failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub id: usize,
    pub title: String,
    pub message: String,
    pub kind: NotificationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationKind {
    Error,
    Success,
}

/// The handle views use to raise a message.
///
/// Every mutation in the app reports its failures through this. The previous
/// interface omitted most of its error handling, so a failed save simply did
/// nothing visible -- which is worse than an error, because it looks like it
/// worked.
#[derive(Clone, PartialEq)]
pub struct Notifier {
    push: Callback<(NotificationKind, String, String)>,
}

impl Notifier {
    pub fn error(&self, err: &ApiError) {
        self.push.emit((
            NotificationKind::Error,
            format!("{} {}", err.code, err.error),
            err.message.clone(),
        ));
    }

    pub fn message(&self, title: &str, message: &str) {
        self.push.emit((
            NotificationKind::Error,
            title.to_string(),
            message.to_string(),
        ));
    }

    pub fn success(&self, message: &str) {
        self.push.emit((
            NotificationKind::Success,
            "Done".to_string(),
            message.to_string(),
        ));
    }
}

#[hook]
pub fn use_notifier() -> Notifier {
    use_context::<Notifier>().expect("a NotificationProvider above this component")
}

#[derive(Properties, PartialEq)]
pub struct NotificationProviderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(NotificationProvider)]
pub fn notification_provider(props: &NotificationProviderProps) -> Html {
    let notifications = use_state(|| Rc::new(Vec::<Notification>::new()));
    let next_id = use_mut_ref(|| 0usize);

    let push = {
        let notifications = notifications.clone();
        let next_id = next_id.clone();

        Callback::from(
            move |(kind, title, message): (NotificationKind, String, String)| {
                let id = {
                    let mut next = next_id.borrow_mut();
                    *next += 1;
                    *next
                };

                let mut updated = (**notifications).clone();
                updated.push(Notification {
                    id,
                    title,
                    message,
                    kind,
                });

                notifications.set(Rc::new(updated));
            },
        )
    };

    let dismiss = {
        let notifications = notifications.clone();

        Callback::from(move |id: usize| {
            let updated: Vec<Notification> = notifications
                .iter()
                .filter(|notification| notification.id != id)
                .cloned()
                .collect();

            notifications.set(Rc::new(updated));
        })
    };

    let notifier = Notifier { push };

    html! {
        <ContextProvider<Notifier> context={notifier}>
            { for props.children.iter() }

            <div class="notifications" role="log" aria-live="polite">
                { for notifications.iter().map(|notification| {
                    let dismiss = dismiss.clone();
                    let id = notification.id;
                    let kind = match notification.kind {
                        NotificationKind::Error => "notification--error",
                        NotificationKind::Success => "notification--success",
                    };

                    html! {
                        <div key={id} class={classes!("notification", kind)}>
                            <div class="notification__title">{ &notification.title }</div>
                            <div class="notification__message">{ &notification.message }</div>
                            <button
                                type="button"
                                class="notification__close"
                                aria-label="Dismiss"
                                onclick={Callback::from(move |_: MouseEvent| dismiss.emit(id))}>
                                { "×" }
                            </button>
                        </div>
                    }
                }) }
            </div>
        </ContextProvider<Notifier>>
    }
}
