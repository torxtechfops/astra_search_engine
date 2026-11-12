use core::fmt;

pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

impl Default for SearchResult {
    fn default() -> Self {
        SearchResult {
            title: String::default(),
            link: String::default(),
            snippet: String::default(),
        }
    }
}

impl fmt::Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Title: {0}\nLink: {1}\nSnippet: {2}", self.title, self.link, self.snippet)
    }
}

pub trait Json {
    fn into_json(&self) -> String;
}

impl Json for SearchResult {
    fn into_json(&self) -> String {
        format!("{{\n\"title\": {}\n\"link\": {}\n\"snippet\": {}\n}}", self.title, self.link, self.snippet)
    }
}

#[allow(dead_code)]
pub trait DisplayResults {
    fn display_all(&self);
}

impl DisplayResults for Vec<SearchResult> {
    fn display_all(&self) {
        let len: usize = self.len();
        let mut index: usize = 1;

        for result in self {
            println!("{}/{}", index, len);
            println!("{}", result);

            index += 1;
        };
    }
}