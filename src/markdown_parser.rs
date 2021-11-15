// use chumsky::prelude::*;
// use chumsky::text::*;

// Discord will ping everyone unless the mention is inside a code block
// below are my tests on writing a markdown parser using chumsky but it's not worth doing
// all that works just to cover this rare edge case...

//{
// // fast exit so we don't have to parse the entire thing
// if !content.contains("@everyone") && !content.contains("@here") {
//     return false;
// }
//
// let tokens = parse_discord_markdown(content);
// for token in tokens {
//     if let DiscordMarkdown::Text(text) = token {
//         return text.contains("@everyone") || text.contains("@here");
//     }
// }
//
// return false;
//}

// #[derive(Clone, Debug)]
// pub enum DiscordMarkdown {
//     Italics(String),
//     Bold(String),
//     BoldItalics(String),
//     Underline(String),
//     UnderlineItalics(String),
//     UnderlineBold(String),
//     UnderlineBoldItalics(String),
//     Strikethrough(String),
//     InlineCode(String),
//     CodeBlock(String),
//     SingleLineQuote(String),
//     BlockQuote(String),
//     Text(String),
//     Char(char)
// }
//
// // my IDE fucking dies trying to identify every type in this monster
// fn parser() -> impl Parser<char, Vec<DiscordMarkdown>, Error = Simple<char>> {
//     // *italics*
//     let italics_star = just('*')
//         .ignore_then(filter(|c| *c != '*').repeated())
//         .then_ignore(just('*'))
//         .collect::<String>()
//         .map(DiscordMarkdown::Italics);
//
//     // _italics_
//     let italics_underscore = just('_')
//         .ignore_then(filter(|c| *c != '_').repeated())
//         .then_ignore(just('_'))
//         .collect::<String>()
//         .map(DiscordMarkdown::Italics);
//
//     // **bold**
//     let bold = just('*')
//         .ignore_then(just('*'))
//         .ignore_then(filter(|c| *c != '*').repeated())
//         .then_ignore(just('*'))
//         .then_ignore(just('*'))
//         .collect::<String>()
//         .map(DiscordMarkdown::Bold);
//
//     // ***bold italics***
//     let bold_italics = just('*')
//         .ignore_then(just('*'))
//         .ignore_then(just('*'))
//         .ignore_then(filter(|c| *c != '*').repeated())
//         .then_ignore(just('*'))
//         .then_ignore(just('*'))
//         .then_ignore(just('*'))
//         .collect::<String>()
//         .map(DiscordMarkdown::BoldItalics);
//
//     // __underline__
//     let underline = just('_')
//         .ignore_then(just('_'))
//         .ignore_then(filter(|c| *c != '_').repeated())
//         .then_ignore(just('_'))
//         .then_ignore(just('_'))
//         .collect::<String>()
//         .map(DiscordMarkdown::Underline);
//
//     // __*underline italics*__
//     let underline_italics = just('_')
//         .ignore_then(just('_'))
//         .ignore_then(just('*'))
//         .ignore_then(filter(|c| *c != '*').repeated())
//         .then_ignore(just('*'))
//         .then_ignore(just('_'))
//         .then_ignore(just('_'))
//         .collect::<String>()
//         .map(DiscordMarkdown::UnderlineItalics);
//
//     // __**underline bold**__
//     let underline_bold = just('_')
//         .ignore_then(just('_'))
//         .ignore_then(just('*'))
//         .ignore_then(just('*'))
//         .ignore_then(filter(|c| *c != '*').repeated())
//         .then_ignore(just('*'))
//         .then_ignore(just('*'))
//         .then_ignore(just('_'))
//         .then_ignore(just('_'))
//         .collect::<String>()
//         .map(DiscordMarkdown::UnderlineBold);
//
//     // __***underline bold italics***__
//     let underline_bold_italics = just('_')
//         .ignore_then(just('_'))
//         .ignore_then(just('*'))
//         .ignore_then(just('*'))
//         .ignore_then(just('*'))
//         .ignore_then(filter(|c| *c != '*').repeated())
//         .then_ignore(just('*'))
//         .then_ignore(just('*'))
//         .then_ignore(just('*'))
//         .then_ignore(just('_'))
//         .then_ignore(just('_'))
//         .collect::<String>()
//         .map(DiscordMarkdown::UnderlineBoldItalics);
//
//     // ~~Strikethrough~~
//     let strikethrough = just('~')
//         .ignore_then(just('~'))
//         .ignore_then(filter(|c| *c != '~').repeated())
//         .then_ignore(just('~'))
//         .then_ignore(just('~'))
//         .collect::<String>()
//         .map(DiscordMarkdown::Strikethrough);
//
//     // `inline code`
//     let inline_code = just('`')
//         .ignore_then(filter(|c| *c != '`').repeated())
//         .then_ignore(just('`'))
//         .collect::<String>()
//         .map(DiscordMarkdown::InlineCode);
//
//     // ```txt
//     // Hello World!
//     // ```
//     let code_block = just('`')
//         .ignore_then(just('`'))
//         .ignore_then(just('`'))
//         .ignore_then(filter(|c| *c != '`').repeated())
//         .then_ignore(just('`'))
//         .then_ignore(just('`'))
//         .then_ignore(just('`'))
//         .collect::<String>()
//         .map(DiscordMarkdown::CodeBlock);
//
//     // "> this is a quote", the space is important!
//     let single_line_quote = just('>')
//         .ignore_then(whitespace())
//         .ignore_then(filter(|c| *c != '\n').repeated())
//         .collect::<String>()
//         .map(DiscordMarkdown::SingleLineQuote);
//
//     // ">>> Quote", all text from >>> until the end is a quote
//     let block_quote = just('>')
//         .ignore_then(just('>'))
//         .ignore_then(just('>'))
//         .ignore_then(whitespace())
//         .ignore_then(any().repeated())
//         .then_ignore(end())
//         .collect::<String>()
//         .map(DiscordMarkdown::BlockQuote);
//
//     // let text = filter(|c| *c != '`' && *c != '>' && *c != '*' && *c != '_' && *c != '~')
//     //     .then()
//     //     .repeated()
//     //     .then_ignore(end())
//     //     .collect::<String>()
//     //     .map(DiscordMarkdown::Text);
//
//     let text = any().map(DiscordMarkdown::Char);
//     // let text = filter(|c| *c != '`')
//     //     .or()
//
//     let token = block_quote
//         .or(single_line_quote)
//         .or(code_block)
//         .or(inline_code)
//         .or(strikethrough)
//         .or(underline_bold_italics)
//         .or(underline_bold)
//         .or(underline_italics)
//         .or(underline)
//         .or(bold_italics)
//         .or(bold)
//         .or(italics_star)
//         .or(italics_underscore)
//         .or(text);
//
//     token
//         .repeated()
//         .then_ignore(end())
// }
//
// fn combine_chars(tokens: Vec<DiscordMarkdown>) -> Vec<DiscordMarkdown> {
//     let mut res: Vec<DiscordMarkdown> = Vec::new();
//
//     let mut cur_string: Vec<char> = Vec::new();
//
//     for i in 0..tokens.len() {
//         let cur = tokens.get(i).unwrap();
//         match cur {
//             DiscordMarkdown::Char(c) => {
//                 cur_string.push(*c);
//                 let next = tokens.get(i + 1);
//                 match next {
//                     Some(t) => {
//                         match t {
//                             DiscordMarkdown::Char(_) => continue,
//                             _ => {
//                                 res.push(DiscordMarkdown::Text(cur_string.iter().collect::<String>()));
//                                 cur_string = Vec::new();
//                             }
//                         }
//                     },
//                     None => {
//                         res.push(DiscordMarkdown::Text(cur_string.iter().collect::<String>()));
//                         cur_string = Vec::new();
//                     }
//                 }
//             },
//             _ => continue
//         }
//     }
//
//     return res;
// }
//
// pub fn parse_discord_markdown(contents: &String) -> Vec<DiscordMarkdown> {
//     let tokens = parser().parse(contents.as_str()).unwrap();
//     let tokens = combine_chars(tokens);
//     return tokens;
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::message_utils::{combine_chars, DiscordMarkdown, is_mentioning_everyone, parser};
//     use chumsky::prelude::*;
//
//     fn test_parser(contents: &str) -> DiscordMarkdown {
//         let tokens = parser().parse(contents).unwrap();
//         assert_eq!(1, tokens.len());
//         let token = tokens.get(0).unwrap().to_owned();
//         return token;
//     }
//
//     #[test]
//     fn test_parser_italics_star() {
//         let token = test_parser("*Hello World*");
//         assert!(matches!(token, DiscordMarkdown::Italics(s) if s == "Hello World"));
//     }
//
//     #[test]
//     fn test_parser_italics_underscore() {
//         let token = test_parser("_Hello World_");
//         assert!(matches!(token, DiscordMarkdown::Italics(s) if s == "Hello World"));
//     }
//
//     #[test]
//     fn test_parser_bold() {
//         let token = test_parser("**Hello World**");
//         assert!(matches!(token, DiscordMarkdown::Bold(s) if s == "Hello World"));
//     }
//
//     #[test]
//     fn test_parser_bold_italics() {
//         let token = test_parser("***Hello World***");
//         assert!(matches!(token, DiscordMarkdown::BoldItalics(s) if s == "Hello World"));
//     }
//
//     #[test]
//     fn test_parser_underline() {
//         let token = test_parser("__Hello World__");
//         assert!(matches!(token, DiscordMarkdown::Underline(s) if s == "Hello World"));
//     }
//
//     #[test]
//     fn test_parser_underline_italics() {
//         let token = test_parser("__*Hello World*__");
//         assert!(matches!(token, DiscordMarkdown::UnderlineItalics(s) if s == "Hello World"));
//     }
//
//     #[test]
//     fn test_parser_underline_bold() {
//         let token = test_parser("__**Hello World**__");
//         assert!(matches!(token, DiscordMarkdown::UnderlineBold(s) if s == "Hello World"));
//     }
//
//     #[test]
//     fn test_parser_underline_bold_italics() {
//         let token = test_parser("__***Hello World***__");
//         assert!(matches!(token, DiscordMarkdown::UnderlineBoldItalics(s) if s == "Hello World"));
//     }
//
//     #[test]
//     fn test_parser_strikethrough() {
//         let token = test_parser("~~Hello World~~");
//         assert!(matches!(token, DiscordMarkdown::Strikethrough(s) if s == "Hello World"));
//     }
//
//     #[test]
//     fn test_parser_inline_code() {
//         let token = test_parser("`Hello World`");
//         assert!(matches!(token, DiscordMarkdown::InlineCode(s) if s == "Hello World"));
//     }
//
//     #[test]
//     fn test_parser_code_block() {
//         let token = test_parser("```txt\nHello World\n```");
//         assert!(matches!(token, DiscordMarkdown::CodeBlock(s) if s == "txt\nHello World\n"))
//     }
//
//     #[test]
//     fn test_parser_single_line_quote() {
//         let token = test_parser("> Hello World");
//         assert!(matches!(token, DiscordMarkdown::SingleLineQuote(s) if s == "Hello World"))
//     }
//
//     #[test]
//     fn test_parser_block_quote() {
//         let token = test_parser(">>> Hello World\nMy name is erri120!");
//         assert!(matches!(token, DiscordMarkdown::BlockQuote(s) if s == "Hello World\nMy name is erri120!"))
//     }
//
//     #[test]
//     fn test_parser_text() {
//         let tokens = parser().parse("This is just normal text. My is this the hardest thing...").unwrap();
//         let tokens = combine_chars(tokens);
//         assert_eq!(1, tokens.len());
//         let token = tokens.get(0).unwrap().to_owned();
//         assert!(matches!(token, DiscordMarkdown::Text(s) if s == "This is just normal text. My is this the hardest thing..."))
//     }
// }
