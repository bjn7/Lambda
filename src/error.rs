#[macro_export]
macro_rules! throw_lexer_syntax_error {
    ($expected:expr, $got:expr, $row:expr, $col:expr) => {
        panic!("Expected: '{}', got: '{}' at {}:{}", $expected, $got, $row, $col)
    };
}

#[macro_export]
macro_rules! throw_syntax_error {
    ($expected:expr, $got:expr) => {
        panic!("Unexpected token: {:?}, expected: {:?}", $expected, $got)
    };
}