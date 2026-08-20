use std::collections::HashSet;

use rex_api::{IdeaV1, IdeaV2, IdeaV3};

use super::{Id, format_id, new_id, parse_id};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Idea {
    pub id: Id,
    pub collection_id: Id,
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
    pub completed: bool,
}

/// The optional narrowing applied to `get_ideas` / `get_random_idea`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IdeaFilter {
    pub tag: Option<String>,
    pub is_completed: Option<bool>,
}

impl IdeaFilter {
    pub fn new(tag: Option<String>, is_completed: Option<bool>) -> Self {
        Self { tag, is_completed }
    }

    pub fn matches(&self, idea: &Idea) -> bool {
        if let Some(is_completed) = self.is_completed
            && idea.completed != is_completed
        {
            return false;
        }

        if let Some(tag) = self.tag.as_deref()
            && !idea.tags.contains(tag)
        {
            return false;
        }

        true
    }
}

impl From<Idea> for IdeaV1 {
    fn from(idea: Idea) -> Self {
        Self {
            id: Some(format_id(idea.id)),
            name: idea.name,
            description: idea.description,
        }
    }
}

impl From<IdeaV1> for Idea {
    fn from(val: IdeaV1) -> Self {
        Idea {
            id: val.id.as_deref().and_then(parse_id).unwrap_or_default(),
            collection_id: 0,
            name: val.name,
            description: val.description,
            tags: HashSet::new(),
            completed: false,
        }
    }
}

impl From<Idea> for IdeaV2 {
    fn from(idea: Idea) -> Self {
        Self {
            id: Some(format_id(idea.id)),
            name: idea.name,
            description: idea.description,
            tags: if idea.tags.is_empty() {
                None
            } else {
                Some(idea.tags)
            },
            completed: Some(idea.completed),
        }
    }
}

impl From<IdeaV2> for Idea {
    fn from(val: IdeaV2) -> Self {
        Idea {
            id: val.id.as_deref().and_then(parse_id).unwrap_or_default(),
            collection_id: 0,
            name: val.name,
            description: val.description,
            tags: val.tags.unwrap_or_default(),
            completed: val.completed.unwrap_or(false),
        }
    }
}

impl From<Idea> for IdeaV3 {
    fn from(idea: Idea) -> Self {
        Self {
            id: Some(format_id(idea.id)),
            collection: Some(format_id(idea.collection_id)),
            name: idea.name,
            description: idea.description,
            tags: if idea.tags.is_empty() {
                None
            } else {
                Some(idea.tags)
            },
            completed: Some(idea.completed),
        }
    }
}

impl From<IdeaV3> for Idea {
    fn from(val: IdeaV3) -> Self {
        Idea {
            id: val.id.as_deref().and_then(parse_id).unwrap_or_else(new_id),
            collection_id: val
                .collection
                .as_deref()
                .and_then(parse_id)
                .unwrap_or_default(),
            name: val.name,
            description: val.description,
            tags: val.tags.unwrap_or_default(),
            completed: val.completed.unwrap_or(false),
        }
    }
}
