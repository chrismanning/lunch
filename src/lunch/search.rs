pub trait Search {
    fn search_terms(&self) -> SearchTerms;
}

pub struct SearchTerms {
    pub terms: Vec<String>,
    pub keywords: Vec<String>,
}
