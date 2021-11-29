use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::io::Read;
use std::str::FromStr;

use indexmap::IndexMap;
use itertools::Itertools;
use jieba_rs::Jieba;
use log::info;
use pinyin::{Pinyin, ToPinyin};

pub struct SunBible {
    exact_dict: IndexMap<String, String>,
    pinyin_dict: IndexMap<PinyinSeq, String>,
    jieba: Jieba,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct PinyinSeq(Box<[&'static str]>);

impl Display for PinyinSeq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join(" "))
    }
}

impl FromStr for PinyinSeq {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_pinyin()
            .map(|item| item.map(Pinyin::plain).ok_or(()))
            .collect::<Result<Box<_>, ()>>()
            .map(Self)
    }
}

impl SunBible {
    pub fn new(dict: IndexMap<String, String>) -> Self {
        info!("Loading jieba dict...");
        let mut jieba = Jieba::new();
        jieba.add_word("爱了", None, None);
        let exact_dict = dict.clone();
        info!("Generating pinyin dict...");
        let pinyin_dict = dict
            .into_iter()
            .map(|(phrase, emoji)| (PinyinSeq::from_str(phrase.as_ref()).ok(), emoji))
            .filter_map(|(k, v)| k.map(|k| (k, v)))
            .collect();
        Self {
            exact_dict,
            pinyin_dict,
            jieba,
        }
    }

    pub fn new_from_reader(reader: impl Read) -> serde_json::Result<Self> {
        Ok(Self::new(serde_json::from_reader(reader)?))
    }

    fn single_emoji(&self, phrase: &str) -> String {
        // try exact phrase match
        self.exact_dict.get(phrase).cloned().unwrap_or_else(|| {
            // try pinyin match
            PinyinSeq::from_str(phrase)
                .ok()
                .and_then(|seq| self.pinyin_dict.get(&seq))
                .cloned()
                .unwrap_or_else(|| {
                    if phrase.chars().count() == 1 {
                        // return single char
                        phrase.to_string()
                    } else {
                        // convert word by word
                        let mut buf = [0; 4];
                        phrase
                            .chars()
                            .map(|c| self.single_emoji(c.encode_utf8(&mut buf)))
                            .join("")
                    }
                })
        })
    }
    pub fn convert(&self, sentence: &str) -> String {
        self.jieba
            .cut(sentence, true)
            .into_iter()
            .map(|phrase| self.single_emoji(phrase))
            .join("")
    }
}
