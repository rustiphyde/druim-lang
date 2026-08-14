#[cfg(test)]
mod tests {
    use crate::compiler::lexer::{LexError, Lexer};
    use crate::compiler::token::TokenKind;
    use crate::compiler::token::TokenKind::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        let mut lx = Lexer::new(src);
        lx.tokenize()
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn colon_family_tokens() {
        let ks = kinds("a:b a::b a:=b a:?b a:>b");

        assert_eq!(
            ks,
            vec![
                Ident, Colon, Ident,
                Ident, Get, Ident,
                Ident, Copy, Ident,
                Ident, Has, Ident,
                Ident, Bind, Ident,
                Eof,
            ]
        );
    }

    #[test]
    fn keyword_vs_identifier() {
        let ks = kinds("num numx text void fn ret");
        assert_eq!(ks[0], KwNum);
        assert_eq!(ks[1], Ident);
        assert_eq!(ks[2], KwText);
        assert_eq!(ks[3], KwVoid);
        assert_eq!(ks[4], KwFn);
        assert_eq!(ks[5], KwRet);
    }

    #[test]
    fn number_literals() {
        let ks = kinds("42 3.14");
        assert_eq!(ks[0], NumLit);
        assert_eq!(ks[1], DecLit);
    }

    #[test]
    fn text_literal() {
        let ks = kinds("\"hello\"");
        assert_eq!(ks[0], TextLit);
    }

    #[test]
    fn block_tokens() {
        let src = ":[ \"carrots\", \"eggs\", \"milk\" ]: :| name: \"Rusty\", age: 47 |: :{ a = 16; }{ d := a; }: fn my_function :( b )( a = b; ):";
        let tokens = Lexer::new(src).tokenize().unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();

        assert!(kinds.contains(&TokenKind::BoxStart));
        assert!(kinds.contains(&TokenKind::BoxEnd));
        assert!(kinds.contains(&TokenKind::BagStart));
        assert!(kinds.contains(&TokenKind::BagEnd));
        assert!(kinds.contains(&TokenKind::BlockStart));
        assert!(kinds.contains(&TokenKind::BlockChain));
        assert!(kinds.contains(&TokenKind::BlockEnd));
        assert!(kinds.contains(&TokenKind::FuncStart));
        assert!(kinds.contains(&TokenKind::FuncChain));
        assert!(kinds.contains(&TokenKind::FuncEnd));
    }

    #[test]
    fn loop_tokens() {
        let src = ":< >?< >?< >:";
        let tokens = Lexer::new(src)
            .tokenize()
            .expect("loop delimiters should lex");

        let kinds: Vec<TokenKind> = tokens
            .iter()
            .map(|token| token.kind)
            .collect();

        assert_eq!(
            kinds,
            vec![
                TokenKind::LoopStart,
                TokenKind::LoopSplit,
                TokenKind::LoopSplit,
                TokenKind::LoopEnd,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn digit_leading_identifiers() {
        let ks = kinds("1a 9lives 123abc 123_456 1_foo");
        assert_eq!(ks[0], Ident);
        assert_eq!(ks[1], Ident);
        assert_eq!(ks[2], Ident);
        assert_eq!(ks[3], Ident);
        assert_eq!(ks[4], Ident);
    }

    #[test]
    fn pure_digit_sequences_are_numbers() {
        let ks = kinds("1 123 000");
        assert_eq!(ks[0], NumLit);
        assert_eq!(ks[1], NumLit);
        assert_eq!(ks[2], NumLit);
    }

    #[test]
    fn invalid_decimal_forms_error() {
        let mut lx = Lexer::new(".");
        assert!(lx.tokenize().is_err());

        let mut lx = Lexer::new(".5");
        assert!(lx.tokenize().is_err());

        let mut lx = Lexer::new("1.");
        assert!(lx.tokenize().is_err());

        let mut lx = Lexer::new("1..2");
        assert!(lx.tokenize().is_err());
    }

    #[test]
    fn guard_token() {
        let ks = kinds("x ?= y;");
        assert!(ks.contains(&Guard));
    }

    #[test]
    fn lexes_left_bracket() {
        let tokens = Lexer::new("[")
            .tokenize()
            .expect("left bracket should lex");

        assert_eq!(
            tokens[0].kind,
            TokenKind::LBracket,
        );
    }

    #[test]
    fn lexes_right_bracket() {
        let tokens = Lexer::new("]")
            .tokenize()
            .expect("right bracket should lex");

        assert_eq!(
            tokens[0].kind,
            TokenKind::RBracket,
        );
    }

    #[test]
    fn box_delimiters_remain_distinct_from_index_brackets() {
        let tokens = Lexer::new(":[1]: [0]")
            .tokenize()
            .expect("box and index brackets should lex");

        let kinds: Vec<TokenKind> = tokens
            .into_iter()
            .map(|token| token.kind)
            .collect();

        assert!(kinds.contains(&TokenKind::BoxStart));
        assert!(kinds.contains(&TokenKind::BoxEnd));
        assert!(kinds.contains(&TokenKind::LBracket));
        assert!(kinds.contains(&TokenKind::RBracket));
    }

    #[test]
    fn indexed_get_after_bracket_does_not_lex_as_box_end() {
        let tokens = Lexer::new("items::[0]::[1]")
            .tokenize()
            .expect("indexed traversal should lex");

        let kinds: Vec<TokenKind> = tokens
            .into_iter()
            .map(|token| token.kind)
            .collect();

        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident,
                TokenKind::Get,
                TokenKind::LBracket,
                TokenKind::NumLit,
                TokenKind::RBracket,
                TokenKind::Get,
                TokenKind::LBracket,
                TokenKind::NumLit,
                TokenKind::RBracket,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn indexed_has_after_bracket_does_not_lex_as_box_end() {
        let tokens = Lexer::new("items::[0]:?[1]")
            .tokenize()
            .expect("indexed traversal should lex");

        let kinds: Vec<TokenKind> = tokens
            .into_iter()
            .map(|token| token.kind)
            .collect();

        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident,
                TokenKind::Get,
                TokenKind::LBracket,
                TokenKind::NumLit,
                TokenKind::RBracket,
                TokenKind::Has,
                TokenKind::LBracket,
                TokenKind::NumLit,
                TokenKind::RBracket,
                TokenKind::Eof,
            ],
        );
    }

    #[test]
    fn scope_and_binding_keywords() {
        let tokens = Lexer::new("loc glo stone")
            .tokenize()
            .expect("lexing should succeed");

        assert_eq!(tokens[0].kind, TokenKind::KwLoc);
        assert_eq!(tokens[1].kind, TokenKind::KwGlo);
        assert_eq!(tokens[2].kind, TokenKind::KwStone);
        assert_eq!(tokens[3].kind, TokenKind::Eof);
    }

    #[test]
    fn mutate_token() {
        let tokens = Lexer::new("value << value + 1;")
            .tokenize()
            .expect("lexing should succeed");

        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[1].kind, TokenKind::Mutate);
        assert_eq!(tokens[2].kind, TokenKind::Ident);
        assert_eq!(tokens[3].kind, TokenKind::Add);
        assert_eq!(tokens[4].kind, TokenKind::NumLit);
        assert_eq!(tokens[5].kind, TokenKind::Semicolon);
        assert_eq!(tokens[6].kind, TokenKind::Eof);
    }

    #[test]
    fn print_token() {
        let tokens = Lexer::new("|> value;")
            .tokenize()
            .expect("lexing should succeed");

        assert_eq!(tokens[0].kind, TokenKind::Print);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[2].kind, TokenKind::Semicolon);
        assert_eq!(tokens[3].kind, TokenKind::Eof);
    }

    #[test]
    fn lexes_program_boundary() {
        let tokens = Lexer::new(":-:-:")
            .tokenize()
            .expect("lexing should succeed");

        assert_eq!(tokens[0].kind, TokenKind::ProgramBoundary);
        assert_eq!(tokens[0].lexeme, ":-:-:");
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }

    #[test]
    fn skips_single_line_comment() {
        let tokens = Lexer::new(":- this is ignored -: value = 42;")
            .tokenize()
            .expect("lexing should succeed");

        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].lexeme, "value");
    }

    #[test]
    fn skips_multiline_comment() {
        let src = r#"
    :--
        x = 99;
        fn fake :()(ret 42;):
    --:
    value = 42;
    "#;

        let tokens = Lexer::new(src)
            .tokenize()
            .expect("lexing should succeed");

        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].lexeme, "value");
    }

    #[test]
    fn rejects_unterminated_single_line_comment() {
        let err = Lexer::new(":- this never closes")
            .tokenize()
            .expect_err("unterminated comment should fail");

        assert!(matches!(
            err,
            LexError::UnterminatedSingleComment { .. }
        ));
    }

    #[test]
    fn rejects_unterminated_multiline_comment() {
        let err = Lexer::new(":-- this never closes")
            .tokenize()
            .expect_err("unterminated comment should fail");

        assert!(matches!(
            err,
            LexError::UnterminatedMultiComment { .. }
        ));
    }

    #[test]
    fn lexes_interpolated_text() {
        let tokens = Lexer::new(
            r#""Hello, :.name.:!""#
        )
        .tokenize()
        .expect("lexing should succeed");

        assert_eq!(tokens[0].kind, TextStart);
        assert_eq!(tokens[1].kind, TextLit);
        assert_eq!(tokens[1].lexeme, "Hello, ");
        assert_eq!(tokens[2].kind, InterpStart);
        assert_eq!(tokens[3].kind, Ident);
        assert_eq!(tokens[3].lexeme, "name");
        assert_eq!(tokens[4].kind, InterpEnd);
        assert_eq!(tokens[5].kind, TextLit);
        assert_eq!(tokens[5].lexeme, "!");
        assert_eq!(tokens[6].kind, TextEnd);
        assert_eq!(tokens[7].kind, Eof);
    }

    #[test]
    fn lexes_interpolated_expression() {
        let tokens = Lexer::new(
            r#""Total: :.price + tax.:.""#
        )
        .tokenize()
        .expect("lexing should succeed");

        assert_eq!(tokens[0].kind, TextStart);
        assert_eq!(tokens[1].kind, TextLit);
        assert_eq!(tokens[1].lexeme, "Total: ");
        assert_eq!(tokens[2].kind, InterpStart);
        assert_eq!(tokens[3].kind, Ident);
        assert_eq!(tokens[3].lexeme, "price");
        assert_eq!(tokens[4].kind, Add);
        assert_eq!(tokens[5].kind, Ident);
        assert_eq!(tokens[5].lexeme, "tax");
        assert_eq!(tokens[6].kind, InterpEnd);
        assert_eq!(tokens[7].kind, TextLit);
        assert_eq!(tokens[7].lexeme, ".");
        assert_eq!(tokens[8].kind, TextEnd);
        assert_eq!(tokens[9].kind, Eof);
    }

    #[test]
    fn ordinary_text_remains_single_text_literal() {
        let tokens = Lexer::new(
            r#""Version 1.2 is current.""#
        )
        .tokenize()
        .expect("lexing should succeed");

        assert_eq!(tokens[0].kind, TextLit);
        assert_eq!(
            tokens[0].lexeme,
            "Version 1.2 is current."
        );
        assert_eq!(tokens[1].kind, Eof);
    }

}
