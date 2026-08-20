use rex_api::CollectionV3;

use super::{Id, format_id, new_id, parse_id};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Collection {
    pub collection_id: Id,
    /// The principal this view of the collection belongs to.
    ///
    /// Collections themselves are shared, so this is populated with whoever
    /// asked for it — which is exactly what the UI assumes `userId` means.
    pub user_id: Id,
    pub name: String,
}

impl From<Collection> for CollectionV3 {
    fn from(collection: Collection) -> Self {
        Self {
            id: Some(format_id(collection.collection_id)),
            user_id: Some(format_id(collection.user_id)),
            name: collection.name,
        }
    }
}

impl From<CollectionV3> for Collection {
    fn from(val: CollectionV3) -> Self {
        Collection {
            user_id: val
                .user_id
                .as_deref()
                .and_then(parse_id)
                .unwrap_or_default(),
            collection_id: val.id.as_deref().and_then(parse_id).unwrap_or_else(new_id),
            name: val.name,
        }
    }
}
