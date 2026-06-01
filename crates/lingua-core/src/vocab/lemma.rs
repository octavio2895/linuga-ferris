pub struct LemmaResult {
    pub lemma: String,
    pub pos: String,
    pub gender: Option<String>,
    pub translation: Option<String>,
}

pub trait Lemmatizer {
    fn lemmatize(&self, word: &str, context: &str) -> LemmaResult;
}

pub struct NaiveLemmatizer;

impl Lemmatizer for NaiveLemmatizer {
    fn lemmatize(&self, word: &str, _context: &str) -> LemmaResult {
        LemmaResult {
            lemma: word.to_lowercase(),
            pos: "unknown".to_string(),
            gender: None,
            translation: None,
        }
    }
}
