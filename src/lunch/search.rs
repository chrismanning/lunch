use std::borrow::Cow;
use std::collections::BTreeMap;
use std::rc::Rc;

use fst::{Automaton, IntoStreamer, Map, MapBuilder, Streamer};
use fst::map::{IndexedValue, OpBuilder};
use fst_levenshtein::Levenshtein;

use lunch::errors::*;
use lunch::Lunchable;

struct LunchableIndex {
    lunchable_index: Vec<IndexedLunchable>,
    maps: Vec<Map>,
}

impl LunchableIndex {
    fn new<'a, I: Iterator<Item=&'a Rc<Lunchable>>>(lunchables: I) -> Result<Self> {
        let mut lunchable_index = vec![];
        let mut counter: u64 = 0;
        let mut maps: Vec<Map> = vec![];
        for lunchable in lunchables {
            debug!("Indexing '{}'", lunchable);
            match IndexedLunchable::new(lunchable.clone(), &mut counter) {
                Ok(lunchable_idx) => {
                    for map in lunchable_idx.to_maps()? {
                        maps.push(map);
                    }
                    lunchable_index.push(lunchable_idx);
                },
                Err(err) => {
                    error!("Error indexing '{}': {}", lunchable, err);
                    return Err(err.into());
                }
            }
        }
        Ok(LunchableIndex {
            lunchable_index,
            maps,
        })
    }

    fn search_idx(&self) -> Result<SearchIndex> {
        let mut op_builder = OpBuilder::new();
        for map in &self.maps[..] {
            op_builder.push(map);
        }
        let mut union = op_builder.union();

        let mut counter: u64 = 0;
        let mut index_mapping: Vec<Vec<IndexedValue>> = vec![];
        let mut map_builder = MapBuilder::memory();
        while let Some((bytes, indexed_values)) = union.next() {
            let i = counter;
            counter += 1;
            trace!("indexed: {:?}; {}", String::from_utf8(bytes.into()), i);
            map_builder.insert(bytes, i);
            index_mapping.push(indexed_values.into());
        }
        let bytes = map_builder.into_inner()?;
        trace!("fst size: {} bytes", bytes.len());
        trace!("idx_mapping: {:?}", index_mapping);
        let combined_index = Map::from_bytes(bytes)?;

        Ok(SearchIndex {
            combined_index,
            index_mapping,
        })
    }
}

struct SearchIndex {
    combined_index: Map,
    index_mapping: Vec<Vec<IndexedValue>>,
}

impl SearchIndex {
    fn map(&self) -> &Map {
        &self.combined_index
    }
}

struct IndexedLunchable {
    lunchable: Rc<Lunchable>,
    term_idx: BTreeMap<Term, u32>,
}

#[derive(Eq)]
enum Term {
    Keyword(String, u64),
    Term(String, u64),
}

impl Term {
    fn get_term<'a>(&'a self) -> &'a str {
        match self {
            &Term::Keyword(ref s, _) => &s,
            &Term::Term(ref s, _) => &s,
        }
    }
}

impl Ord for Term {
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
        self.get_term().to_lowercase().cmp(&other.get_term().to_lowercase())
    }
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
        self.get_term().to_lowercase().partial_cmp(&other.get_term().to_lowercase())
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        self.get_term().to_lowercase().eq(&other.get_term().to_lowercase())
    }
}

impl IndexedLunchable {
    fn new(lunchable: Rc<Lunchable>, start_idx: &mut u64) -> Result<IndexedLunchable> {
        let search_idx = lunchable.search_terms();
        let mut term_idx = BTreeMap::new();
        for term in search_idx.terms.iter() {
            let i = *start_idx;
            *start_idx += 1;
            trace!("Adding term '{}' to index for '{}'", term, lunchable);
            *term_idx.entry(Term::Term(term.to_string(), i)).or_insert(0) += 1;
        };
        for keyword in search_idx.keywords.iter() {
            let i = *start_idx;
            *start_idx += 1;
            trace!("Adding keyword '{}' to index for '{}", keyword, lunchable);
            *term_idx.entry(Term::Keyword(keyword.to_string(), i)).or_insert(0) += 1;
        };

        Ok(IndexedLunchable {
            term_idx,
            lunchable: lunchable.clone(),
        })
    }

    fn to_maps(&self) -> Result<Vec<Map>> {
        let mut term_map = MapBuilder::memory();
        let mut keyword_map = MapBuilder::memory();
        for term in self.term_idx.keys() {
            match term {
                &Term::Keyword(ref keyword, id) => {
                    trace!("Adding keyword '{}' to Map", keyword.to_lowercase());
                    keyword_map.insert(keyword.to_lowercase().as_bytes(), id)?;
                }
                &Term::Term(ref term, id) => {
                    trace!("Adding term '{}' to Map", term.to_lowercase());
                    term_map.insert(term.to_lowercase().as_bytes(), id)?;
                }
            }
        }

        vec![term_map, keyword_map].into_iter()
            .map(|map_builder| map_builder.into_inner().and_then(Map::from_bytes).map_err(|err| err.into()))
            .collect()
    }
}

pub trait SearchIdxItem {
    fn search_terms<'a>(&'a self) -> SearchIdxData<'a>;
}

#[derive(Debug)]
pub struct SearchIdxData<'a> {
    pub terms: Vec<Cow<'a, str>>,
    pub keywords: Vec<Cow<'a, str>>,
}

pub type Weight = i32;

pub trait Search {
    fn search<I: IntoIterator<Item=SRef>, SRef: AsRef<str>>(&self, search_terms: I) -> Weight;
}

pub struct Searcher {
    pub lunchables: Vec<Rc<Lunchable>>,
}

impl Searcher {
    pub fn score<I, S>(&self, terms: I) -> Result<()>
//                                        BTreeMap<Weight, Rc<Lunchable>>
    where
        I: IntoIterator<Item=S>,
        S: AsRef<str>,
    {
        let index = LunchableIndex::new(self.lunchables.iter())?;
        let search_index = index.search_idx()?;
        for term in terms.into_iter() {
            debug!("Searching for term '{}' in index", term.as_ref());
            let query = Levenshtein::new(term.as_ref(), 1)
                .and_then(|a| Levenshtein::new(term.as_ref(), 1)
                    .map(|b| b.starts_with())
                    .map(|b| a.union(b)))?;
            let mut result_stream = search_index.combined_index.search(&query).into_stream();

            let mut kvs = vec![];
            while let Some((k, v)) = result_stream.next() {
                debug!("Result: ({:?}, {:?})", k, v);
                if let Some(indexed_values) = search_index.index_mapping.get(v as usize) {
                    debug!("Matched item {}: {:?}", v, indexed_values);
                    for match_idx in indexed_values {
                        for lunchable_idx in &index.lunchable_index {
                            for term in &lunchable_idx.term_idx {
                                match term {
                                    (&Term::Term(ref term, i), freq) => {
                                        if i == match_idx.value {
                                            info!("Matched term '{}' on '{}'", term, lunchable_idx.lunchable);
                                        }
                                    },
                                    (&Term::Keyword(ref keyword, i), freq) => {
                                        if i == match_idx.value {
                                            info!("Matched keyword '{}' on '{}'", keyword, lunchable_idx.lunchable);
                                        }
                                    },
                                }
                            }
                        }
                    }
                }

                kvs.push((k.to_vec(), v));
            }
            debug!("{:?}", kvs);

        }
//        let union = op_builder.union();

        // TODO create Idx
        // TODO create Map
        // TODO create map Op (union)
        Ok(())
    }

    fn create_map_op(&self) -> () {

    }
}
