use super::*;

#[test]
fn test_small_main_programm() {
    let input = r#"
        int main() {
            return 42;
        } 
        "#;

    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::IntNumber("42".to_string()),
            Token::Semicolon,
            Token::RightBrace
        ]
    );
}

#[test]
fn test_return_token() {
    let input = "return";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(tokens, vec![Token::Return]);
}

#[test]
fn test_int_token() {
    let input = "int";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(tokens, vec![Token::IntKeyword]);
}

#[test]
fn test_identifier() {
    let input = "my_variable123";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(
        tokens,
        vec![Token::Identifier("my_variable123".to_string())]
    );
}

#[test]
fn test_keyword_in_identifier() {
    let input = "my_return_int_var";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(
        tokens,
        vec![Token::Identifier("my_return_int_var".to_string())]
    );
}

#[test]
fn test_left_paren_token() {
    let input = "(";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(tokens, vec![Token::LeftParen]);
}

#[test]
fn test_right_paren_token() {
    let input = ")";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(tokens, vec![Token::RightParen]);
}

#[test]
fn test_left_brace_token() {
    let input = "{";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(tokens, vec![Token::LeftBrace]);
}

#[test]
fn test_right_brace_token() {
    let input = "}";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(tokens, vec![Token::RightBrace]);
}

#[test]
fn test_semicolon_token() {
    let input = ";";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(tokens, vec![Token::Semicolon]);
}

#[test]
fn test_int_number() {
    let input = "42";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(tokens, vec![Token::IntNumber("42".to_string())]);
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
            column: 1,
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
            column: 14,
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
            column: 5,
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
            column: 10,
        })
    );
}

#[test]
fn test_plus_token() {
    let input = "40 + 2";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::IntNumber("40".to_string()),
            Token::Plus,
            Token::IntNumber("2".to_string())
        ]
    );
}

#[test]
fn test_minus_token() {
    let input = "40 - 2";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::IntNumber("40".to_string()),
            Token::Minus,
            Token::IntNumber("2".to_string())
        ]
    );
}

#[test]
fn test_star_token() {
    let input = "40 * 2";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::IntNumber("40".to_string()),
            Token::Star,
            Token::IntNumber("2".to_string())
        ]
    );
}

#[test]
fn test_slash_token() {
    let input = "40 / 2";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::IntNumber("40".to_string()),
            Token::Slash,
            Token::IntNumber("2".to_string())
        ]
    );
}

#[test]
fn test_variable() {
    let input = "int i = 0;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::IntKeyword,
            Token::Identifier("i".to_string()),
            Token::Equal,
            Token::IntNumber("0".to_string()),
            Token::Semicolon
        ]
    );
}

#[test]
fn test_if_else() {
    let input = "if(a==1){}else{}";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::If,
            Token::LeftParen,
            Token::Identifier("a".to_string()),
            Token::DoubleEqual,
            Token::IntNumber("1".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::RightBrace,
            Token::Else,
            Token::LeftBrace,
            Token::RightBrace
        ]
    );
}

#[test]
fn test_greater_less_equals() {
    let input = "<= >= == < >";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();

    assert_eq!(
        result,
        vec![
            Token::LessOrEqual,
            Token::GreaterOrEqual,
            Token::DoubleEqual,
            Token::LessThan,
            Token::GreaterThan
        ]
    )
}

#[test]
fn test_parameters() {
    let input = "int main(int a, int b)";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();

    assert_eq!(
        result,
        vec![
            Token::IntKeyword,
            Token::Identifier("main".to_string()),
            Token::LeftParen,
            Token::IntKeyword,
            Token::Identifier("a".to_string()),
            Token::Comma,
            Token::IntKeyword,
            Token::Identifier("b".to_string()),
            Token::RightParen,
        ]
    );
}

#[test]
fn test_bool() {
    let input = "bool a = false;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();

    assert_eq!(
        result,
        vec![
            Token::BoolKeyword,
            Token::Identifier("a".to_string()),
            Token::Equal,
            Token::Bool(false),
            Token::Semicolon
        ]
    );
}

#[test]
fn test_string() {
    let input = "string a = \"test\"";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();

    assert_eq!(
        result,
        vec![
            Token::StringKeyword,
            Token::Identifier("a".to_string()),
            Token::Equal,
            Token::String("test".to_string())
        ]
    );
}

#[test]
fn test_float() {
    let input = "float a = 3.0";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();

    assert_eq!(
        result,
        vec![
            Token::FloatKeyword,
            Token::Identifier("a".to_string()),
            Token::Equal,
            Token::FloatNumber("3.0".to_string()),
        ]
    );
}
#[test]
fn test_struct_and_member_access() {
    let input = "struct Point { int x; int y; } Point p; p.x = 5;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::StructKeyword,
            Token::Identifier("Point".to_string()),
            Token::LeftBrace,
            Token::IntKeyword,
            Token::Identifier("x".to_string()),
            Token::Semicolon,
            Token::IntKeyword,
            Token::Identifier("y".to_string()),
            Token::Semicolon,
            Token::RightBrace,
            Token::Identifier("Point".to_string()),
            Token::Identifier("p".to_string()),
            Token::Semicolon,
            Token::Identifier("p".to_string()),
            Token::Dot,
            Token::Identifier("x".to_string()),
            Token::Equal,
            Token::IntNumber("5".to_string()),
            Token::Semicolon,
        ]
    );
}

#[test]
fn test_boolean_true_and_comparisons() {
    let input = "bool flag = true; if (flag == true) {}";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::BoolKeyword,
            Token::Identifier("flag".to_string()),
            Token::Equal,
            Token::Bool(true),
            Token::Semicolon,
            Token::If,
            Token::LeftParen,
            Token::Identifier("flag".to_string()),
            Token::DoubleEqual,
            Token::Bool(true),
            Token::RightParen,
            Token::LeftBrace,
            Token::RightBrace,
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
    assert_eq!(result, vec![Token::String("line1\nline2".to_string())]);
}

#[test]
fn test_brackets_token() {
    let input = "int arr[10];";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::IntKeyword,
            Token::Identifier("arr".to_string()),
            Token::LeftBracket,
            Token::IntNumber("10".to_string()),
            Token::RightBracket,
            Token::Semicolon,
        ]
    );
}

#[test]
fn test_array_index_assignment() {
    let input = "arr[0] = 42;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::Identifier("arr".to_string()),
            Token::LeftBracket,
            Token::IntNumber("0".to_string()),
            Token::RightBracket,
            Token::Equal,
            Token::IntNumber("42".to_string()),
            Token::Semicolon,
        ]
    );
}

#[test]
fn test_line_and_column_increment_on_newlines() {
    let input = "int\na;";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::IntKeyword,
            Token::Identifier("a".to_string()),
            Token::Semicolon,
        ]
    );
}

#[test]
fn test_float_number_variations() {
    let input = "0.0 123.456";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::FloatNumber("0.0".to_string()),
            Token::FloatNumber("123.456".to_string()),
        ]
    );
}
#[test]
fn test_while_token() {
    let input = "while";
    let mut sc = Scanner::new(input);
    let tokens = sc.scan_source().unwrap();
    assert_eq!(tokens, vec![Token::While]);
}
#[test]
fn test_while_loop_tokens() {
    let input = "while (i < 5) { i = i + 1; }";
    let mut sc = Scanner::new(input);
    let result = sc.scan_source().unwrap();
    assert_eq!(
        result,
        vec![
            Token::While,
            Token::LeftParen,
            Token::Identifier("i".to_string()),
            Token::LessThan,
            Token::IntNumber("5".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Identifier("i".to_string()),
            Token::Equal,
            Token::Identifier("i".to_string()),
            Token::Plus,
            Token::IntNumber("1".to_string()),
            Token::Semicolon,
            Token::RightBrace,
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

    let expected = vec![
        Token::IntKeyword,
        Token::Identifier("x".to_string()),
        Token::Equal,
        Token::IntNumber("10".to_string()),
        Token::Semicolon,
        Token::While,
        Token::LeftParen,
        Token::Identifier("x".to_string()),
        Token::GreaterThan,
        Token::IntNumber("0".to_string()),
        Token::RightParen,
        Token::LeftBrace,
        Token::Identifier("x".to_string()),
        Token::Equal,
        Token::Identifier("x".to_string()),
        Token::Minus,
        Token::IntNumber("1".to_string()),
        Token::Semicolon,
        Token::RightBrace,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_scan_print_statement() {
    let source = "print(42);";
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_source().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::PrintKeyword,
            Token::LeftParen,
            Token::IntNumber("42".to_string()),
            Token::RightParen,
            Token::Semicolon,
        ]
    );
}

#[test]
fn test_scan_print_string() {
    let source = "print(\"Hallo Welt!\");";
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_source().unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::PrintKeyword,
            Token::LeftParen,
            Token::String("Hallo Welt!".to_string()),
            Token::RightParen,
            Token::Semicolon,
        ]
    );
}
