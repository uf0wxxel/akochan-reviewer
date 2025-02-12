use crate::pai::Pai;
use std::fmt;

use serde::de::Error;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{serde_as, DisplayFromStr};

/// Describes an event in mjlog format.
///
/// Note that this is a simplified version of mjlog, but it is sufficient for
/// akochan to read.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    None,

    StartGame {
        kyoku_first: u8,
        aka_flag: bool,
        names: [String; 4],
    },
    StartKyoku {
        #[serde_as(as = "DisplayFromStr")]
        bakaze: Pai,
        #[serde_as(as = "DisplayFromStr")]
        dora_marker: Pai,
        kyoku: u8, // counts from 1
        honba: u8,
        kyotaku: u8,
        oya: u8,
        scores: [i32; 4],
        #[serde_as(as = "[[DisplayFromStr; 13]; 4]")]
        tehais: [[Pai; 13]; 4],
    },

    Tsumo {
        actor: u8,
        #[serde_as(as = "DisplayFromStr")]
        pai: Pai,
    },
    Dahai {
        actor: u8,
        #[serde_as(as = "DisplayFromStr")]
        pai: Pai,
        tsumogiri: bool,
    },

    Chi {
        actor: u8,
        target: u8,
        #[serde_as(as = "DisplayFromStr")]
        pai: Pai,
        consumed: Consumed2,
    },
    Pon {
        actor: u8,
        target: u8,
        #[serde_as(as = "DisplayFromStr")]
        pai: Pai,
        consumed: Consumed2,
    },
    Daiminkan {
        actor: u8,
        target: u8,
        #[serde_as(as = "DisplayFromStr")]
        pai: Pai,
        consumed: Consumed3,
    },
    Kakan {
        actor: u8,
        #[serde_as(as = "DisplayFromStr")]
        pai: Pai,
        consumed: Consumed3,
    },
    Ankan {
        actor: u8,
        consumed: Consumed4,
    },
    Dora {
        #[serde_as(as = "DisplayFromStr")]
        dora_marker: Pai,
    },

    Reach {
        actor: u8,
    },
    ReachAccepted {
        actor: u8,
    },

    Hora {
        actor: u8,
        target: u8,

        // It is an Option because akochan won't send this field, but we need to
        // record the field.
        #[serde(skip_serializing_if = "Option::is_none")]
        deltas: Option<[i32; 4]>,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default)]
        #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
        ura_markers: Option<Vec<Pai>>,
    },
    Ryukyoku {
        #[serde(skip_serializing_if = "Option::is_none")]
        deltas: Option<[i32; 4]>,
    },

    EndKyoku,
    EndGame,
}

impl Eq for Event {}

// ["5sr", "3p", "6m", ...] => [Pai::AkaSou5, Pai::Pin3, Pai::Man6, ...]
macro_rules! make_pai_array_from_string_array {
    ($array:ident, $($index:expr),*) => {
        [$($array[$index].parse::<Pai>().map_err(Error::custom)?),*]
    };
}

macro_rules! build_consumed_struct {
    ($name:ident; $n:expr; $($index:expr),*) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct $name([Pai; $n]);

        impl fmt::Debug for $name {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.0, f)
            }
        }

        impl From<[Pai; $n]> for $name {
            #[inline]
            fn from(pais: [Pai; $n]) -> Self {
                let mut list = pais;
                list.sort_unstable_by_key(|pai| pai.as_ord());
                Self(list)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                for p in &self.0 {
                    seq.serialize_element(&p.to_string())?;
                }
                seq.end()
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let s = <[String; $n]>::deserialize(deserializer)?;
                let pais = make_pai_array_from_string_array!(s, $($index),*);
                Ok($name::from(pais))
            }
        }

        impl $name {
            #[inline]
            pub const fn as_array(self) -> [Pai; $n] {
                self.0
            }
        }
    };
}

build_consumed_struct!(Consumed2; 2; 0, 1);
build_consumed_struct!(Consumed3; 3; 0, 1, 2);
build_consumed_struct!(Consumed4; 4; 0, 1, 2, 3);

impl Event {
    #[inline]
    pub fn actor(&self) -> Option<u8> {
        match *self {
            Event::Tsumo { actor, .. }
            | Event::Dahai { actor, .. }
            | Event::Chi { actor, .. }
            | Event::Pon { actor, .. }
            | Event::Daiminkan { actor, .. }
            | Event::Kakan { actor, .. }
            | Event::Ankan { actor, .. }
            | Event::Reach { actor, .. }
            | Event::ReachAccepted { actor, .. }
            | Event::Hora { actor, .. } => Some(actor),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn naki_info(&self) -> Option<(u8, Pai)> {
        match *self {
            Event::Chi { target, pai, .. }
            | Event::Pon { target, pai, .. }
            | Event::Daiminkan { target, pai, .. } => Some((target, pai)),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn naki_to_ord(&self) -> isize {
        match *self {
            Event::Chi { .. } => 0,
            Event::Pon { .. } => 1,
            _ => -1,
        }
    }
}
