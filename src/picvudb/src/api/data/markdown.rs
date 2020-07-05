use std::str::FromStr;
use pulldown_cmark::{html, Event, Options, Parser, Tag};

use crate::err::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleMarkdown(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotesMarkdown(String);

impl TitleMarkdown
{
    pub fn parse(input: String) -> Result<TitleMarkdown, ParseError>
    {
        let parser = Parser::new_ext(&input, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH);

        let mut got_paragraph = false;

        for event in parser
        {
            match event
            {
                Event::Start(tag) =>
                {
                    match tag
                    {
                        Tag::Emphasis | Tag::Strong | Tag::Strikethrough =>
                        {
                            // OK - these are allowed in a a title
                        },
                        Tag::Paragraph =>
                        {
                            if !got_paragraph
                            {
                                // Plain text is wrapped in a single paragraph element -
                                // we'll allow this, but no more

                                got_paragraph = true;
                            }
                            else
                            {
                                return Err(ParseError::new(format!("Markdown error: Title: Block-level elements not allowed in title: {:?}, {:?}", input, tag)));
                            }
                        },
                        _ =>
                        {
                            //return Err(ParseError::new("Markdown error: Title: Block-level elements not allowed in title".to_owned()));
                            return Err(ParseError::new(format!("Markdown error: Title: Block-level elements not allowed in title: {:?}, {:?}", input, tag)));
                        },
                    }
                },
                Event::Html(_) =>
                {
                    return Err(ParseError::new("Markdown error: Title: inline HTML not allowed".to_owned()));
                },
                Event::FootnoteReference(_) =>
                {
                    return Err(ParseError::new("Markdown error: Title: Footnotes not supported".to_owned()));
                },
                Event::SoftBreak
                    | Event::HardBreak
                    | Event::Rule =>
                {
                    return Err(ParseError::new("Markdown error: Title: Line breaks not allowed".to_owned()));
                },
                Event::TaskListMarker(_) =>
                {
                    return Err(ParseError::new("Markdown error: Title: Task-lists not supported".to_owned()));
                },
                Event::End(_)
                    | Event::Code(_)
                    | Event::Text(_) =>
                {
                    // All OK - we handle these events                    
                }
            }
        }
        
        Ok(TitleMarkdown(input))
    }

    pub(crate) fn from_db_field(input: Option<String>) -> Result<Option<TitleMarkdown>, ParseError>
    {
        match input
        {
            Some(input) => Ok(Some(TitleMarkdown::parse(input)?)),
            None => Ok(None),
        }
    }

    pub fn get_markdown(&self) -> String
    {
        self.0.clone()
    }

    pub fn get_events(&self) -> impl Iterator<Item = pulldown_cmark::Event>
    {
        let isnt_paragraph = |e: &Event| -> bool
        {
            match e
            {
                Event::Start(tag) =>
                {
                    match tag
                    {
                        Tag::Paragraph => false,
                        _ => true,
                    }
                },
                Event::End(tag) =>
                {
                    match tag
                    {
                        Tag::Paragraph => false,
                        _ => true,
                    }
                },
                _ =>
                {
                    true
                },
            }
        };

        Parser::new_ext(&self.0, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH).filter(isnt_paragraph)
    }

    pub fn get_html(&self) -> String
    {
        let mut result = String::new();
        html::push_html(&mut result, self.get_events());
        result
    }

    pub fn get_display_text(&self) -> String
    {
        get_display_text(&self.0)
    }

    pub fn get_search_text(&self) -> String
    {
        get_search_text(&self.0)
    }
}

impl FromStr for TitleMarkdown
{
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err>
    {
        TitleMarkdown::parse(input.to_owned())
    }
}

impl NotesMarkdown
{
    pub fn parse(input: String) -> Result<NotesMarkdown, ParseError>
    {
        let parser = Parser::new_ext(&input, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH);

        for event in parser
        {
            match event
            {
                Event::Start(tag) =>
                {
                    match tag
                    {
                        Tag::FootnoteDefinition(_) =>
                        {
                            return Err(ParseError::new("Markdown error: Notes: footnotes not supported".to_owned()));
                        },
                        _ =>
                        {
                            // All other tags are supported
                        },
                    }
                },
                Event::Html(_) =>
                {
                    return Err(ParseError::new("Markdown error: Notes: inline HTML not allowed".to_owned()));
                },
                Event::FootnoteReference(_) =>
                {
                    return Err(ParseError::new("Markdown error: Notes: Footnotes not supported".to_owned()));
                },
                Event::TaskListMarker(_) =>
                {
                    return Err(ParseError::new("Markdown error: Notes: Task-lists not supported".to_owned()));
                },
                Event::End(_)
                    | Event::SoftBreak
                    | Event::HardBreak
                    | Event::Rule
                    | Event::Code(_)
                    | Event::Text(_) =>
                {
                    // All OK - we handle these events                    
                }
            }
        }
        
        Ok(NotesMarkdown(input))
    }

    pub(crate) fn from_db_field(input: Option<String>) -> Result<Option<NotesMarkdown>, ParseError>
    {
        match input
        {
            Some(input) => Ok(Some(NotesMarkdown::parse(input)?)),
            None => Ok(None),
        }
    }

    pub fn get_markdown(&self) -> String
    {
        self.0.clone()
    }

    pub fn get_html(&self) -> String
    {
        let mut result = String::new();
        html::push_html(&mut result, Parser::new_ext(&self.0, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH));
        result
    }

    pub fn get_search_text(&self) -> String
    {
        get_search_text(&self.0)
    }
}

impl FromStr for NotesMarkdown
{
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err>
    {
        NotesMarkdown::parse(input.to_owned())
    }
}

fn get_display_text(s: &str) -> String
{
    // To get a "display" string, we just want to
    // return the content from "Text" events.

    let mut result = String::new();
    let parser = Parser::new_ext(s, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH);

    for event in parser
    {
        match event
        {
            Event::Text(t) => result.push_str(&t),
            Event::Code(t) => result.push_str(&t),
            _ => (),
        }
    }

    result
}

fn get_search_text(s: &str) -> String
{
    // To get a search string, we want to return
    // all text, but ensure we insert spaces
    // to break up words - e.g.

    let mut result = String::new();
    let parser = Parser::new_ext(s, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH);

    let mut insert = |s: &str|
    {
        let mut got_space = true;

        if !result.is_empty()
            && (result.as_bytes()[result.len() - 1] != b' ')
        {
            got_space = false;
        }

        for ch in s.chars()
        {
            if ch.is_whitespace()
            {
                if !got_space
                {
                    result.push(' ');
                    got_space = true;
                }
            }
            else
            {
                result.push(ch);
                got_space = false;
            }
        }
    };

    for event in parser
    {
        match event
        {
            Event::Start(tag) =>
            {
                match tag
                {
                    Tag::Paragraph
                        | Tag::Heading(_)
                        | Tag::BlockQuote
                        | Tag::CodeBlock(_)
                        | Tag::List(_)
                        | Tag::Item
                        | Tag::TableCell =>
                    {
                        insert(" ");
                    },
                    _ =>
                    {
                        // Other elements are either table elements that
                        // will eventually get to a TableCell, or
                        // inline elements (strong, emphasis, strikethrough, link)
                        // so we only need the explicit space from text elements.
                    },
                }
            },
            Event::End(_) =>
            {
                // Ignore
            },
            Event::Code(t) =>
            {
                insert(&t);
            },
            Event::Text(t) =>
            {
                insert(&t);
            },
            Event::SoftBreak
                | Event::HardBreak
                | Event::Rule =>
            {
                insert(" ");                
            }
            _ =>
            {
                // Ignore
            },
        }
    }

    result
}