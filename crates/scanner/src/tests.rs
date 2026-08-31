use super::*;

macro_rules! test_single_tokens {
        ( $( $name:ident : $source:expr => $kind:expr ),* $(,)? ) => {
            $(
                #[test]
                fn $name() {
                    let mut scanner = Scanner::new($source);
                    let result = scanner.scan_source().expect("Scanner should not fail");

                    let expected = vec![
                        Token::new(1, 1, $kind),
                        Token::new(1, 1 + $source.len(), TokenKind::EoF),
                    ];

                    assert_eq!(result, expected);
                }
            )*
        };
    }

// Automatische Generierung der Tests für alle Token-Typen:
test_single_tokens! {
    test_kw_return: "return" => TokenKind::Return,
    test_kw_int: "int" => TokenKind::IntKeyword,
    test_kw_bool: "bool" => TokenKind::BoolKeyword,
    test_kw_float: "float" => TokenKind::FloatKeyword,
    test_kw_string: "string" => TokenKind::StringKeyword,
    test_kw_print: "print" => TokenKind::PrintKeyword,
    test_kw_struct: "struct" => TokenKind::StructKeyword,
    test_kw_if: "if" => TokenKind::If,
    test_kw_else: "else" => TokenKind::Else,
    test_kw_while: "while" => TokenKind::While,

    test_bool_true: "true" => TokenKind::Bool(true),
    test_bool_false: "false" => TokenKind::Bool(false),
    test_identifier: "foo_bar" => TokenKind::Identifier("foo_bar"),
    test_int_num: "12345" => TokenKind::IntNumber("12345"),
    test_float_num: "3.1415" => TokenKind::FloatNumber("3.1415"),
    test_string_lit: "\"hallo\"" => TokenKind::String("hallo"),

    test_op_plus: "+" => TokenKind::Plus,
    test_op_minus: "-" => TokenKind::Minus,
    test_op_star: "*" => TokenKind::Star,
    test_op_slash: "/" => TokenKind::Slash,
    test_op_equal: "=" => TokenKind::Equal,
    test_op_less: "<" => TokenKind::LessThan,
    test_op_greater: ">" => TokenKind::GreaterThan,
    test_op_less_eq: "<=" => TokenKind::LessOrEqual,
    test_op_greater_eq: ">=" => TokenKind::GreaterOrEqual,
    test_op_double_eq: "==" => TokenKind::DoubleEqual,

    test_paren_left: "(" => TokenKind::LeftParen,
    test_paren_right: ")" => TokenKind::RightParen,
    test_brace_left: "{" => TokenKind::LeftBrace,
    test_brace_right: "}" => TokenKind::RightBrace,
    test_bracket_left: "[" => TokenKind::LeftBracket,
    test_bracket_right: "]" => TokenKind::RightBracket,
    test_comma: "," => TokenKind::Comma,
    test_semicolon: ";" => TokenKind::Semicolon,
    test_dot: "." => TokenKind::Dot,
}

#[test]
fn test_small_main_programm() {
    let input = r#"
        int main() {
            return 42;
        } 
        "#;

    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::IntKeyword,
            &TokenKind::Identifier("main"),
            &TokenKind::LeftParen,
            &TokenKind::RightParen,
            &TokenKind::LeftBrace,
            &TokenKind::Return,
            &TokenKind::IntNumber("42"),
            &TokenKind::Semicolon,
            &TokenKind::RightBrace,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_return_token() {
    let input = "return";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(kinds, vec![&TokenKind::Return, &TokenKind::EoF]);
}

#[test]
fn test_keyword_in_identifier() {
    let input = "my_return_int_var";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![&TokenKind::Identifier("my_return_int_var"), &TokenKind::EoF]
    );
}

#[test]
fn test_int_number() {
    let input = "42";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(kinds, vec![&TokenKind::IntNumber("42"), &TokenKind::EoF]);
}

#[test]
fn test_unexpected_character_at_start() {
    let input = "@int main() {}";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source();

    assert_eq!(
        result,
        Err(ScannerError {
            message: "Unexpected character: '@'".to_string(),
            line: 1,
            column: 2,
        })
    );
}

#[test]
fn test_unexpected_character_multiline() {
    let input = "int main() {\n    return 42#;\n}";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source();

    assert_eq!(
        result,
        Err(ScannerError {
            message: "Unexpected character: '#'".to_string(),
            line: 2,
            column: 15,
        })
    );
}

#[test]
fn test_unexpected_symbol_in_middle() {
    let input = "int $variable;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source();

    assert_eq!(
        result,
        Err(ScannerError {
            message: "Unexpected character: '$'".to_string(),
            line: 1,
            column: 6,
        })
    );
}

#[test]
fn test_unexpected_question_mark() {
    let input = "int main(?) {}";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source();

    assert_eq!(
        result,
        Err(ScannerError {
            message: "Unexpected character: '?'".to_string(),
            line: 1,
            column: 11,
        })
    );
}

#[test]
fn test_plus_token() {
    let input = "40 + 2";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::IntNumber("40"),
            &TokenKind::Plus,
            &TokenKind::IntNumber("2"),
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_minus_token() {
    let input = "40 - 2";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::IntNumber("40"),
            &TokenKind::Minus,
            &TokenKind::IntNumber("2"),
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_star_token() {
    let input = "40 * 2";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::IntNumber("40"),
            &TokenKind::Star,
            &TokenKind::IntNumber("2"),
            &TokenKind::EoF
        ]
    );
}

#[test]
fn test_slash_token() {
    let input = "40 / 2";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::IntNumber("40"),
            &TokenKind::Slash,
            &TokenKind::IntNumber("2"),
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_variable() {
    let input = "int i = 0;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::IntKeyword,
            &TokenKind::Identifier("i"),
            &TokenKind::Equal,
            &TokenKind::IntNumber("0"),
            &TokenKind::Semicolon,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_if_else() {
    let input = "if(a==1){}else{}";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::If,
            &TokenKind::LeftParen,
            &TokenKind::Identifier("a"),
            &TokenKind::DoubleEqual,
            &TokenKind::IntNumber("1"),
            &TokenKind::RightParen,
            &TokenKind::LeftBrace,
            &TokenKind::RightBrace,
            &TokenKind::Else,
            &TokenKind::LeftBrace,
            &TokenKind::RightBrace,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_greater_less_equals() {
    let input = "<= >= == < >";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();

    assert_eq!(
        kinds,
        vec![
            &TokenKind::LessOrEqual,
            &TokenKind::GreaterOrEqual,
            &TokenKind::DoubleEqual,
            &TokenKind::LessThan,
            &TokenKind::GreaterThan,
            &TokenKind::EoF,
        ]
    )
}

#[test]
fn test_parameters() {
    let input = "int main(int a, int b)";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();

    assert_eq!(
        kinds,
        vec![
            &TokenKind::IntKeyword,
            &TokenKind::Identifier("main"),
            &TokenKind::LeftParen,
            &TokenKind::IntKeyword,
            &TokenKind::Identifier("a"),
            &TokenKind::Comma,
            &TokenKind::IntKeyword,
            &TokenKind::Identifier("b"),
            &TokenKind::RightParen,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_bool() {
    let input = "bool a = false;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();

    assert_eq!(
        kinds,
        vec![
            &TokenKind::BoolKeyword,
            &TokenKind::Identifier("a"),
            &TokenKind::Equal,
            &TokenKind::Bool(false),
            &TokenKind::Semicolon,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_string() {
    let input = "string a = \"test\"";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();

    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::StringKeyword,
            &TokenKind::Identifier("a"),
            &TokenKind::Equal,
            &TokenKind::String("test"),
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_float() {
    let input = "float a = 3.0";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();

    assert_eq!(
        kinds,
        vec![
            &TokenKind::FloatKeyword,
            &TokenKind::Identifier("a"),
            &TokenKind::Equal,
            &TokenKind::FloatNumber("3.0"),
            &TokenKind::EoF,
        ]
    );
}
#[test]
fn test_struct_and_member_access() {
    let input = "struct Point { int x; int y; } Point p; p.x = 5;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::StructKeyword,
            &TokenKind::Identifier("Point"),
            &TokenKind::LeftBrace,
            &TokenKind::IntKeyword,
            &TokenKind::Identifier("x"),
            &TokenKind::Semicolon,
            &TokenKind::IntKeyword,
            &TokenKind::Identifier("y"),
            &TokenKind::Semicolon,
            &TokenKind::RightBrace,
            &TokenKind::Identifier("Point"),
            &TokenKind::Identifier("p"),
            &TokenKind::Semicolon,
            &TokenKind::Identifier("p"),
            &TokenKind::Dot,
            &TokenKind::Identifier("x"),
            &TokenKind::Equal,
            &TokenKind::IntNumber("5"),
            &TokenKind::Semicolon,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_boolean_true_and_comparisons() {
    let input = "bool flag = true; if (flag == true) {}";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::BoolKeyword,
            &TokenKind::Identifier("flag"),
            &TokenKind::Equal,
            &TokenKind::Bool(true),
            &TokenKind::Semicolon,
            &TokenKind::If,
            &TokenKind::LeftParen,
            &TokenKind::Identifier("flag"),
            &TokenKind::DoubleEqual,
            &TokenKind::Bool(true),
            &TokenKind::RightParen,
            &TokenKind::LeftBrace,
            &TokenKind::RightBrace,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_unterminated_string_error() {
    let input = "string s = \"hello;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source();
    assert_eq!(
        result,
        Err(ScannerError {
            message: "Unterminated string literal".to_string(),
            line: 1,
            column: 19,
        })
    );
}

#[test]
fn test_multiline_string() {
    let input = "\"line1\nline2\"";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![&TokenKind::String("line1\nline2"), &TokenKind::EoF]
    );
}

#[test]
fn test_brackets_token() {
    let input = "int arr[10];";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::IntKeyword,
            &TokenKind::Identifier("arr"),
            &TokenKind::LeftBracket,
            &TokenKind::IntNumber("10"),
            &TokenKind::RightBracket,
            &TokenKind::Semicolon,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_array_index_assignment() {
    let input = "arr[0] = 42;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Identifier("arr"),
            &TokenKind::LeftBracket,
            &TokenKind::IntNumber("0"),
            &TokenKind::RightBracket,
            &TokenKind::Equal,
            &TokenKind::IntNumber("42"),
            &TokenKind::Semicolon,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_line_and_column_increment_on_newlines() {
    let input = "int\na;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::IntKeyword,
            &TokenKind::Identifier("a"),
            &TokenKind::Semicolon,
            &TokenKind::EoF
        ]
    );
}

#[test]
fn test_float_number_variations() {
    let input = "0.0 123.456";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::FloatNumber("0.0"),
            &TokenKind::FloatNumber("123.456"),
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_while_loop_tokens() {
    let input = "while (i < 5) { i = i + 1; }";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = result.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::While,
            &TokenKind::LeftParen,
            &TokenKind::Identifier("i"),
            &TokenKind::LessThan,
            &TokenKind::IntNumber("5"),
            &TokenKind::RightParen,
            &TokenKind::LeftBrace,
            &TokenKind::Identifier("i"),
            &TokenKind::Equal,
            &TokenKind::Identifier("i"),
            &TokenKind::Plus,
            &TokenKind::IntNumber("1"),
            &TokenKind::Semicolon,
            &TokenKind::RightBrace,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_scanner_comments() {
    let source = "
            // This is a comment at the beginning
            int x = 10; // A comment at the end of the line
            // Another comment
            while (x > 0) {
                x = x - 1; // comment inside the loop 
            }
        ";

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_source().unwrap();
    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();

    let expected = vec![
        &TokenKind::IntKeyword,
        &TokenKind::Identifier("x"),
        &TokenKind::Equal,
        &TokenKind::IntNumber("10"),
        &TokenKind::Semicolon,
        &TokenKind::While,
        &TokenKind::LeftParen,
        &TokenKind::Identifier("x"),
        &TokenKind::GreaterThan,
        &TokenKind::IntNumber("0"),
        &TokenKind::RightParen,
        &TokenKind::LeftBrace,
        &TokenKind::Identifier("x"),
        &TokenKind::Equal,
        &TokenKind::Identifier("x"),
        &TokenKind::Minus,
        &TokenKind::IntNumber("1"),
        &TokenKind::Semicolon,
        &TokenKind::RightBrace,
        &TokenKind::EoF,
    ];

    assert_eq!(kinds, expected);
}

#[test]
fn test_scan_print_statement() {
    let source = "print(42);";
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_source().unwrap();

    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::PrintKeyword,
            &TokenKind::LeftParen,
            &TokenKind::IntNumber("42"),
            &TokenKind::RightParen,
            &TokenKind::Semicolon,
            &TokenKind::EoF,
        ]
    );
}

#[test]
fn test_scan_print_string() {
    let source = "print(\"Hallo Welt!\");";
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_source().unwrap();

    let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::PrintKeyword,
            &TokenKind::LeftParen,
            &TokenKind::String("Hallo Welt!"),
            &TokenKind::RightParen,
            &TokenKind::Semicolon,
            &TokenKind::EoF,
        ]
    );
}
