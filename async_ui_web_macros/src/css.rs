use std::collections::HashSet;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse_macro_input, LitStr};

pub(crate) fn css_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as LitStr);
    generate(input.value()).into()
}
fn generate(input: String) -> TokenStream {
    let classes = find_classes::find_classes(&input);
    let postfix = generate_postfix(&input);
    let postfix = std::str::from_utf8(&postfix).unwrap();
    let mut output = String::with_capacity(input.capacity() + classes.len() * 13);
    let mut last_end_idx = 0;
    let mut names = HashSet::new();
    for (start_idx, length) in classes.iter().cloned() {
        let end_idx = start_idx + length;
        output.push_str(&input[last_end_idx..end_idx]);

        assert_eq!(
            output.as_bytes()[output.len() - length - 1] as char,
            '.' as char
        );
        names.insert(&input[start_idx..end_idx]);
        output.push('-');
        output.push_str(postfix);
        last_end_idx = end_idx;
    }
    output.push_str(&input[last_end_idx..]);
    let classes_declaration = names
        .iter()
        .map(|c| Ident::new(&c.replace("-", "_"), Span::call_site()))
        .collect::<Vec<_>>();

    let classes_value = names.iter().map(|c| format!("{c}-{postfix}"));

    let inner_mod_name = format_ident!("style_{postfix}_inner");
    let style_var_name = format_ident!("style_{postfix}");
    let dce_hack_fn_name = format_ident!("style_{postfix}_dep");

    let js_content = format!(
        "
export const {style_var_name} = `{output}`;
document.head.appendChild(document.createElement(\"style\")).innerHTML = {style_var_name};
"
    );

    quote!(
    #[doc(hidden)]
    mod #inner_mod_name {
        // pub const STYLE_RAW_TEXT: &str = #input;
        // pub const STYLE_POSTFIXED_TEXT: &str = #output;
        #[::wasm_bindgen::prelude::wasm_bindgen(inline_js = #js_content)]
        extern "C" {
            static #style_var_name: ::wasm_bindgen::JsValue;
        }
        #[doc(hidden)]
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub fn #dce_hack_fn_name() {
            let _ = &*#style_var_name;
        }
    }
    #(pub const #classes_declaration: &str = #classes_value;)*
        )
}

fn generate_postfix(input: &str) -> [u8; 8] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();
    let hash = hash.to_ne_bytes();
    hash.map(|n| char::from_digit((n % 32) as u32, 32).unwrap() as u8)
}

mod find_classes {
    use cssparser::{
        AtRuleParser, BasicParseError, BasicParseErrorKind, CowRcStr, Parser, ParserInput,
        QualifiedRuleParser, RuleListParser, Token,
    };

    pub fn find_classes(input: &str) -> Vec<(usize, usize)> {
        let mut input = ParserInput::new(&input);
        let mut input = Parser::new(&mut input);
        let mut classes = Vec::new();
        let mut parser = RuleListParser::new_for_stylesheet(
            &mut input,
            OurParser {
                classes: &mut classes,
            },
        );
        while let Some(_) = parser.next() {}
        classes
    }
    struct OurParser<'d> {
        classes: &'d mut Vec<(usize, usize)>,
    }

    impl<'i, 'd> QualifiedRuleParser<'i> for OurParser<'d>
    where
        'i: 'd,
    {
        type Prelude = ();
        type QualifiedRule = ();
        type Error = ();

        fn parse_prelude<'t>(
            &mut self,
            input: &mut Parser<'i, 't>,
        ) -> Result<Self::Prelude, cssparser::ParseError<'i, Self::Error>> {
            let mut was_dot = false;
            loop {
                let position = input.position().byte_index();
                let token = input.next_including_whitespace();
                let mut is_dot = false;
                match token {
                    Ok(Token::Delim(ch)) if *ch == '.' => {
                        is_dot = true;
                    }
                    Ok(Token::Ident(ident)) if was_dot => {
                        let len = ident.len();
                        self.classes.push((position, len));
                    }
                    Ok(Token::Function(_funct)) => {
                        input.parse_nested_block(|input| {
                            QualifiedRuleParser::parse_prelude(self, input)
                        })?;
                    }
                    Ok(_token) => {}
                    Err(e) => match e.kind {
                        cssparser::BasicParseErrorKind::EndOfInput => break,
                        kind => return Err(input.new_error(kind)),
                    },
                }
                was_dot = is_dot;
            }
            Ok(())
        }

        fn parse_block<'t>(
            &mut self,
            _prelude: Self::Prelude,
            _start: &cssparser::ParserState,
            input: &mut Parser<'i, 't>,
        ) -> Result<Self::QualifiedRule, cssparser::ParseError<'i, Self::Error>> {
            drain_input(input)?;
            Ok(())
        }
    }

    impl<'i, 'd> AtRuleParser<'i> for OurParser<'d>
    where
        'i: 'd,
    {
        type Prelude = bool;
        type AtRule = ();
        type Error = ();

        fn parse_prelude<'t>(
            &mut self,
            name: CowRcStr<'i>,
            input: &mut Parser<'i, 't>,
        ) -> Result<Self::Prelude, cssparser::ParseError<'i, Self::Error>> {
            drain_input(input)?;
            match &*name {
                "media" | "supports" | "container" | "layer" => Ok(true),
                _ => Ok(false),
            }
        }

        fn rule_without_block(
            &mut self,
            _prelude: Self::Prelude,
            _start: &cssparser::ParserState,
        ) -> Result<Self::AtRule, ()> {
            Ok(())
        }

        fn parse_block<'t>(
            &mut self,
            prelude: Self::Prelude,
            _start: &cssparser::ParserState,
            input: &mut Parser<'i, 't>,
        ) -> Result<Self::AtRule, cssparser::ParseError<'i, Self::Error>> {
            if prelude {
                let mut parser = RuleListParser::new_for_nested_rule(
                    input,
                    OurParser {
                        classes: &mut self.classes,
                    },
                );
                while let Some(_) = parser.next() {}
            }
            drain_input(input)?;
            Ok(())
        }
    }

    fn drain_input<'i>(input: &mut Parser<'i, '_>) -> Result<(), BasicParseError<'i>> {
        loop {
            match input.next() {
                Ok(_) => {}
                Err(BasicParseError {
                    kind: BasicParseErrorKind::EndOfInput,
                    ..
                }) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}
