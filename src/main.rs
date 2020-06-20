#[macro_use]
extern crate lazy_static;

use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::html::{start_highlighted_html_snippet, styled_line_to_highlighted_html, IncludeBackground};
use syntect::parsing::SyntaxSet;

const RECENT_POST_LIMIT: usize = 6;
const TOML_MARKER: &str = "+++";

lazy_static! {
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
    static ref THEME: Theme = ThemeSet::get_theme("assets/ayu-light.tmTheme")
        .expect("Unable to load theme");
}

#[derive(Deserialize, Debug)]
struct FrontMatter {
    date: Option<String>,
    title: String,
    summary: String,
    tags: Vec<String>,
    draft: Option<bool>,
}

fn main() {
    // Capture theme before changing working directory
    println!("Using theme: {}", THEME.name.as_ref().unwrap());

    let args: Vec<String> = env::args().collect();
    let working_dir = args.get(1).expect("Must pass a working directory as the first arg");
    let path = std::path::Path::new(working_dir);
    let base_html = fs::read_to_string("assets/base.html").expect("Something went wrong reading base.html");
    env::set_current_dir(path).expect("Unable to set working directory");
    let mut toml_options = Options::empty();
    toml_options.insert(Options::ENABLE_STRIKETHROUGH);
    toml_options.insert(Options::ENABLE_TABLES);

    fs::remove_dir_all("posts").unwrap_or(());
    fs::create_dir("posts").unwrap();
    fs::remove_dir_all("tags").unwrap_or(());
    fs::create_dir("tags").unwrap();

    let mut all = String::new();
    let mut recent = String::new();
    let mut tags: HashMap<String, String> = HashMap::new();
    let unprocessed_posts = fs::read_dir("_posts").expect("Working dir must have a _posts dir");

    unprocessed_posts.enumerate().for_each(|(i, post)| {
        let front_matter = process_post(&base_html, toml_options, &post.unwrap().path());
        let summary_html = get_summary_html(&front_matter);

        if i < RECENT_POST_LIMIT {
            recent += &summary_html;
        }

        front_matter.tags.iter().for_each(|tag| {
            if !tags.contains_key(tag) {
                tags.insert(tag.to_string(), summary_html.to_string());
            }
        });

        all += &summary_html;
    });

    fs::write("index.html", recent).expect("Unable to write index");
    fs::write("all.html", all).expect("Unable to write all");

    for tag_entry in tags {
        let filename = format!("tags/{}.html", tag_entry.0);
        fs::write(filename, tag_entry.1).expect("Unable to write tag");
    }
}

fn get_summary_html(front_matter: &FrontMatter) -> String {
    // TODO render tags
    format!(
        "<li><h3><a href=\"\"></a>{}</h3><div>{}</div><p>{}</p></li>",
        front_matter.title, front_matter.date.as_ref().unwrap(), front_matter.summary
    )
}

pub fn get_highlight_lines<'a>(lang: &str) -> HighlightLines<'a> {
    let syntax = SYNTAX_SET
        .find_syntax_by_token(lang)
        .or_else(|| None)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    HighlightLines::new(syntax, &THEME)
}

fn process_post(base_html: &str, toml_options: Options, path: &PathBuf) -> FrontMatter {
    let filepath = path.to_str().unwrap();
    let new_filename = format!("{}.html", path.file_stem().unwrap().to_str().unwrap());
    let new_filepath = format!("posts/{}", new_filename);
    let file = fs::read_to_string(filepath).expect("Unable to read md");

    // 0: empty, 1: front matter, 2: content
    let splits: Vec<&str> = file.split(TOML_MARKER).collect();
    let mut front_matter: FrontMatter = toml::from_str(&splits[1]).unwrap();
    let mut highlighter: Option<HighlightLines> = None;

    let parser = Parser::new_ext(&splits[2], toml_options).map(|event| {
        match event {
            Event::Text(text) => {
                // Check if we're in the middle of a code block
                if let Some(ref mut highlighter) = highlighter {
                    let highlighted = highlighter.highlight(&text, &SYNTAX_SET);
                    let html =
                        styled_line_to_highlighted_html(&highlighted, IncludeBackground::Yes);
                    return Event::Html(html.into());
                }

                Event::Text(text)
            }
            Event::Start(Tag::CodeBlock(ref kind)) => {
                match kind {
                    CodeBlockKind::Fenced(info) => {
                        highlighter = Some(get_highlight_lines(info));
                    }
                    _ => (),
                }

                let snippet = start_highlighted_html_snippet(&THEME);
                Event::Html(snippet.0.into())
            }
            Event::End(Tag::CodeBlock(_)) => {
                highlighter = None;
                Event::Html("</pre>".into())
            }
            _ => event,
        }
    });

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    let mut tags_html = String::new();

    if front_matter.tags.len() == 0 {
        front_matter.tags.push(String::from("misc"));
    }

    front_matter.tags.iter().for_each(|tag| {
        tags_html += &format!("<a href=\"/tags/{}\">{}</a>", tag, tag);
    });

    let html = base_html
        .replace("{{title}}", &front_matter.title)
        .replace("{{date}}", &front_matter.date.as_ref().unwrap())
        .replace("{{tags}}", &tags_html)
        .replace("{{content}}", &html_output);

    fs::write(new_filepath, html).expect("Unable to write file");

    front_matter
}
