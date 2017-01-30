use traitdef::Value;
use rustc_serialize::json::{Object, Json};
use merger::Mergeable;
use std::mem;
use std::collections::btree_map::Entry::*;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum JsonKey {
    Index(usize),
    String(String),
}

impl Value for Json {
    type Item = Json;
    type Key = JsonKey;
    fn items<'a>(&'a self) -> Option<Box<Iterator<Item = (Self::Key, &'a Self::Item)> + 'a>> {
        match *self {
            Json::String(_) | Json::U64(_) | Json::I64(_) | Json::F64(_) | Json::Boolean(_) |
            Json::Null => None,
            Json::Array(ref inner) => {
                Some(Box::new(inner.iter().enumerate().map(|(i, v)| (JsonKey::Index(i), v))))
            }
            Json::Object(ref inner) => {
                Some(Box::new(inner.iter().map(|(s, v)| (JsonKey::String(s.clone()), v))))
            }
        }
    }
}

impl Mergeable for Json {
    type Key = JsonKey;
    type Item = Json;

    fn set(&mut self, keys: &[Self::Key], v: &Self::Item, _previous: Option<&Self::Item>) {
        if keys.len() == 0 {
            *self = v.clone();
        } else {
            let mut c = self;
            let last_key_index = keys.len() - 1;
            let object_or_value = |index| if index == last_key_index {
                v.clone()
            } else {
                Json::Object(Object::new())
            };
            fn _runup_array_or_value<'a>(array: &'a mut Vec<Json>,
                                         _key_index: usize)
                                         -> &'a mut Json {
                array.push(Json::Null);
                let alen = array.len();
                array.get_mut(alen - 1).expect("at least one item")
            };
            for (i, k) in keys.iter().enumerate() {
                c = match *k {
                    JsonKey::String(ref k) => {
                        match {
                            c
                        } {
                            &mut Json::Object(ref mut obj) => {
                                match obj.entry(k.clone()) {
                                    Vacant(e) => e.insert(object_or_value(i)),
                                    Occupied(e) => {
                                        if i == last_key_index {
                                            *e.into_mut() = v.clone();
                                            return;
                                        } else {
                                            e.into_mut()
                                        }
                                    }
                                }
                            }
                            c @ &mut Json::String(_) |
                            c @ &mut Json::F64(_) |
                            c @ &mut Json::Boolean(_) |
                            c @ &mut Json::Null |
                            c @ &mut Json::U64(_) |
                            c @ &mut Json::Array(_) |
                            c @ &mut Json::I64(_) => {
                                mem::replace(c,
                                             Json::Object({
                                                 let mut o = Object::new();
                                                 o.insert(k.clone(), object_or_value(i));
                                                 o
                                             }));
                                if i == last_key_index {
                                    return;
                                }
                                match c {
                                    &mut Json::Object(ref mut obj) => {
                                        obj.get_mut(k).expect("previous insertion")
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
            }
        }
    }


    //    JsonKey::Index(idx) => {
    //match {
    //c
    //} {
    //&mut Json::Array(ref mut a) => runup_array_or_value(a, i),
    //&mut Json::String(_) |
    //&mut Json::F64(_) |
    //&mut Json::Boolean(_) |
    //&mut Json::Null |
    //&mut Json::U64(_) |
    //&mut Json::Object(_) |
    //&mut Json::I64(_) => {
    //let mut a = Vec::new();
    //runup_array_or_value(&mut a, i);
    //mem::replace(c, Json::Array(a));
    //c
    //}
    //}
    //}

    fn remove(&mut self, keys: &[Self::Key]) {
        let mut c = self;
        for (i, k) in keys.iter().enumerate() {
            c = match *k {
                JsonKey::String(ref k) => {
                    match {
                        c
                    } {
                        &mut Json::Object(ref mut obj) => {
                            if i == keys.len() - 1 {
                                obj.remove(k);
                                return;
                            } else {
                                if let Some(json) = obj.get_mut(k) {
                                    json
                                } else {
                                    return;
                                }
                            }
                        }
                        _ => return,
                    }
                }
                _ => panic!("handle JsonKey::Index"),
            }
        }
    }
}
