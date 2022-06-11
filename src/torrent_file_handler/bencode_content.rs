use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Content {
    Str(String),
    List(Vec<Content>),
    Int(i64),
    Dict(HashMap<String, Content>),
    Bytes(Vec<u8>),
}

impl Content {
    pub fn get_str(&self) -> Option<&String> {
        match self {
            Content::Str(c) => Some(c),
            _ => None,
        }
    }
    pub fn get_list(&self) -> Option<&Vec<Content>> {
        match self {
            Content::List(c) => Some(c),
            _ => None,
        }
    }
    pub fn get_int(&self) -> Option<&i64> {
        match self {
            Content::Int(c) => Some(c),
            _ => None,
        }
    }
    pub fn get_dict(&self) -> Option<&HashMap<String, Content>> {
        match self {
            Content::Dict(c) => Some(c),
            _ => None,
        }
    }
    pub fn get_bytes(&self) -> Option<&Vec<u8>> {
        match self {
            Content::Bytes(c) => Some(c),
            _ => None,
        }
    }
}
