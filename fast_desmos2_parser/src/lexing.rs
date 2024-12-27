use crate::lexing::error::LexResult;
use builtins::Builtins;
use color_eyre::owo_colors::OwoColorize;
pub use error::{ContextError, LexError};
use fast_desmos2_utils::{IdVec, ResExt};
pub use span::Span;
use std::cell::Cell;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;

pub mod builtins;
mod error;
mod span;

pub fn lex<'a>(
    idents: &IdentStorer,
    source: &'a str,
) -> (Vec<Token>, Option<ContextError<'a, LexError>>) {
    let lexer = Lexer::new(idents);
    let mut result = Vec::new();
    let err = loop {
        match lexer.lex_one(source) {
            Ok(token) => result.push(token),
            Err(e) => break e,
        }
    };
    (result, (!err.is_eos()).then_some(err))
}

type Tokens = Vec<Token>;

pub fn display_tokens(source: &str, tokens: &[Token]) {
    println!("{}", "Tokens:".bold());
    print!("   ");
    for (index, token) in tokens.iter().enumerate() {
        match index % 2 > 0 {
            true => print!("{}", token.span(source).yellow()),
            false => print!("{}", token.span(source).blue()),
        }
    }
    println!();

    println!("{}", "Types:".bold());
    for (index, token) in tokens.iter().enumerate() {
        match index % 2 > 0 {
            true => println!(" - {:?}", token.kind.yellow()),
            false => println!(" - {:?}", token.kind.blue()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IdentStorer {
    ids: IdVec<String>,
}

impl IdentStorer {
    pub fn convert_id(&self, id: &str) -> IdentId {
        IdentId(self.ids.id_or_insert_with(id, || id.to_string()))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct IdentId(pub(crate) usize);

impl IdentId {
    pub fn get(&self) -> usize {
        self.0
    }
}

pub struct Lexer<'a> {
    idents: &'a IdentStorer,
    index: Cell<usize>,
    this_lex_start: Cell<usize>,
}

impl<'i> Lexer<'i> {
    pub fn new(idents: &'i IdentStorer) -> Self {
        Self {
            idents,
            index: Cell::new(0),
            this_lex_start: Cell::new(0),
        }
    }

    pub fn lex_one<'a>(&self, from: &'a str) -> Result<Token, ContextError<'a, LexError>> {
        let result: Result<_, LexError> = (|| loop {
            self.this_lex_start.set(self.index.get());

            use PairedPunct as Paired;
            use Punctuation as Punct;
            let result = match self.advance(from)? {
                b'.' => {
                    if self.advance_if_eq2(from, *b"..") {
                        self.token_punct(Punctuation::Ellipses)
                    } else if self.advance_if(from, |x| x.is_ascii_digit()) {
                        self.advance_while(from, |x| x.is_ascii_digit());
                        self.token_with(from, |s| {
                            TokenKind::Number(f64::from_str(s).unwrap_unreach())
                        })
                    } else if self.advance_if_eq(from, b'x') {
                        self.token(TokenKind::Element(Element::X))
                    } else if self.advance_if_eq(from, b'y') {
                        self.token(TokenKind::Element(Element::Y))
                    } else {
                        self.token_punct(Punctuation::Period)
                    }
                }
                b',' => self.token_punct(Punct::Comma),
                b'=' => self.token_punct(Punct::Equals),
                b'-' => self.token_punct(Punct::Minus),
                b'+' => self.token_punct(Punct::Plus),
                b'^' => self.token_punct(Punct::Exp),
                b'_' => self.token_punct(Punct::Subscript),
                b'<' => self.token_punct(Punctuation::LessThan),
                b'>' => self.token_punct(Punctuation::MoreThan),
                b':' => self.token_punct(Punctuation::Colon),
                b'|' => self.token_punct(Punct::Abs),

                b'[' => self.token_l(Paired::Square),
                b']' => self.token_r(Paired::Square),
                b'{' => self.token_l(Paired::LatexCurly),
                b'}' => self.token_r(Paired::LatexCurly),
                b'(' => self.token_l(Paired::Paren),
                b')' => self.token_r(Paired::Paren),

                b'\\' => match self.advance(from)? {
                    b'{' => self.token_l(Paired::Curly),
                    b'}' => self.token_r(Paired::Curly),

                    c if c.is_ascii_alphabetic() => {
                        self.index.set(self.index.get() - 1);
                        self.parse_post_slash(from)?
                    }
                    b' ' => continue,

                    _ => return Err(LexError::UnknownSymbol),
                },
                a if a.is_ascii_alphabetic() => {
                    if self.advance_if_eq(from, b'_') {
                        let mut layers = 0;
                        loop {
                            match self
                                .advance(from)
                                .map_err(|_| LexError::EndOfStringWhile("identifier subscript"))?
                            {
                                b'{' => layers += 1,
                                b'}' => layers -= 1,
                                _ => {}
                            }
                            if layers <= 0 {
                                break;
                            }
                        }
                    }

                    self.token_with(from, |span| {
                        let id = self.idents.convert_id(span);
                        TokenKind::Identifier(id)
                    })
                }
                a if a.is_ascii_digit() => {
                    self.advance_while(from, |x| x.is_ascii_digit());
                    if self.advance_if2(from, |x| x == b'.', |x| x.is_ascii_digit()) {
                        self.advance_while(from, |x| x.is_ascii_digit());
                    }

                    self.token_with(from, |s| {
                        TokenKind::Number(f64::from_str(s).unwrap_unreach())
                    })
                }
                b' ' => continue,
                byte => return Err(LexError::UnknownByte(byte)),
            };
            break Ok(result);
        })();
        result.map_err(|error| ContextError {
            string: from,
            index: self.index.get() - 1,
            error,
        })
    }

    /// expects self.index to be after the backslash.
    /// for example, if the input is `\floor`, the pointer should be on the `f`.
    fn parse_post_slash(&self, from: &str) -> Result<Token, LexError> {
        #[derive(Copy, Clone)]
        enum ValidKind {
            Token(TokenKind),
            Identifier,
            Left,
            Right,
            OperatorName,
        }

        use std::f64::consts;
        const fn number(x: f64) -> ValidKind {
            ValidKind::Token(TokenKind::Number(x))
        }
        const fn punct(x: Punctuation) -> ValidKind {
            ValidKind::Token(TokenKind::Punct(x))
        }
        // const IDENTS: &[&[u8]] = &[
        //     //
        //     // Greek characters
        //     //
        //     b"alpha", b"beta", b"gamma", b"delta", b"epsilon", b"zeta", b"eta", b"theta", b"iota",
        //     b"kappa", b"lambda", b"mu", b"nu", b"xi", b"omicron", b"rho", b"sigma", b"upsilon",
        //     b"phi", b"chi", b"psi", b"omega",
        //     //
        //     // Capital greek characters
        //     //
        //     b"Alpha", b"Beta", b"Gamma", b"Delta", b"Epsilon", b"Zeta", b"Eta", b"Theta", b"Iota",
        //     b"Kappa", b"Lambda", b"Mu", b"Nu", b"Xi", b"Omicron", b"Rho", b"Sigma", b"Upsilon",
        //     b"Phi", b"Chi", b"Psi", b"Omega",
        // ];
        const VALID: &[(&[u8], ValidKind)] = &[
            (b"pi", number(consts::PI)),
            (b"to", punct(Punctuation::Arrow)),
            (b"tau", number(consts::TAU)),
            (b"left", ValidKind::Left),
            (b"right", ValidKind::Right),
            (b"le", punct(Punctuation::LessOrEqual)),
            (b"ge", punct(Punctuation::MoreOrEqual)),
            (b"frac", punct(Punctuation::Frac)),
            (b"sqrt", punct(Punctuation::Sqrt)),
            (b"cdot", punct(Punctuation::Times)),
            (b"times", punct(Punctuation::Times)),
            (b"sum", punct(Punctuation::Sum)),
            (b"prod", punct(Punctuation::Prod)),
            (b"operatorname", ValidKind::OperatorName),
        ];

        let start_part = self.index.get();
        self.advance_while(from, |x| x.is_ascii_alphabetic());

        let span = &from[start_part..self.index.get()];
        // println!("span: {span}");
        let mut kind = ValidKind::Identifier;
        for &(ident, k_kind) in VALID {
            if ident == span.as_bytes() {
                kind = k_kind;
            }
        }

        Ok(match kind {
            ValidKind::Token(token) => self.token(token),
            ValidKind::Identifier => self.token_with(from, |span| {
                let span_b = span.as_bytes();
                assert_eq!(span_b[0], b'\\');
                let body = &span_b[1..];
                Builtins::from_str(body).map_or_else(
                    || TokenKind::Identifier(self.idents.convert_id(span)),
                    TokenKind::Builtins,
                )
            }),
            ValidKind::Left => match self.advance(from)? {
                b'|' => self.token_punct(Punctuation::Abs),
                b'(' => self.token_l(PairedPunct::Paren),
                b'[' => self.token_l(PairedPunct::Square),
                b'\\' => {
                    self.advance_if_eq(from, b'{')
                        .then_some(())
                        .ok_or(LexError::NoBraceLeft)?;
                    self.token_l(PairedPunct::Curly)
                }
                _ => return Err(LexError::NoLeft),
            },
            ValidKind::Right => match self.advance(from)? {
                b'|' => self.token_punct(Punctuation::Abs),
                b')' => self.token_r(PairedPunct::Paren),
                b']' => self.token_r(PairedPunct::Square),
                b'\\' => {
                    self.advance_if_eq(from, b'}')
                        .then_some(())
                        .ok_or(LexError::NoBraceRight)?;
                    self.token_r(PairedPunct::Curly)
                }
                _ => return Err(LexError::NoRight),
            },
            ValidKind::OperatorName => {
                self.advance_if_eq(from, b'{')
                    .then_some(())
                    .ok_or(LexError::BadOperatorName)?;
                let start = self.index.get();
                self.advance_while(from, |x| x.is_ascii_alphabetic());
                let fragment = &from[start..self.index.get()];
                self.advance_if_eq(from, b'}')
                    .then_some(())
                    .ok_or(LexError::BadOperatorName)?;

                match fragment {
                    "for" => self.token_punct(Punctuation::For),
                    "with" => self.token_punct(Punctuation::With),
                    _ => self.token_with(from, |span| {
                        Builtins::from_str(fragment.as_bytes()).map_or_else(
                            || TokenKind::Identifier(self.idents.convert_id(span)),
                            TokenKind::Builtins,
                        )
                    }),
                }
            }
        })
    }
}

impl<'i> Lexer<'i> {
    fn token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            span: Span::new(self.this_lex_start.get(), self.index.get()),
        }
    }

    fn try_token_with<E>(
        &self,
        source: &str,
        with: impl FnOnce(&str) -> Result<TokenKind, E>,
    ) -> Result<Token, E> {
        let span = self.this_token_span();

        Ok(Token {
            kind: with(span.select(source))?,
            span,
        })
    }

    fn this_token_span(&self) -> Span {
        Span::new(self.this_lex_start.get(), self.index.get())
    }

    fn token_with(&self, source: &str, with: impl FnOnce(&str) -> TokenKind) -> Token {
        let span = self.this_token_span();

        Token {
            kind: with(span.select(source)),
            span,
        }
    }

    fn token_punct(&self, punctuation: Punctuation) -> Token {
        self.token(TokenKind::Punct(punctuation))
    }

    fn token_paired(&self, paired_punct: PairedPunct) -> Token {
        self.token(TokenKind::Paired(paired_punct))
    }

    fn token_l(&self, punct: impl FnOnce(LeftRight) -> PairedPunct) -> Token {
        self.token_paired(punct(LeftRight::Left))
    }

    fn token_r(&self, punct: impl FnOnce(LeftRight) -> PairedPunct) -> Token {
        self.token_paired(punct(LeftRight::Right))
    }

    fn peek(&self, from: &str) -> LexResult<u8> {
        from.as_bytes()
            .get(self.index.get())
            .copied()
            .ok_or(LexError::EndOfString)
    }

    fn peek_next(&self, from: &str) -> LexResult<u8> {
        from.as_bytes()
            .get(self.index.get() + 1)
            .copied()
            .ok_or(LexError::EndOfString)
    }

    fn advance(&self, from: &str) -> LexResult<u8> {
        let data = self.peek(from);
        self.move_by(1);
        data
    }

    fn advance_if(&self, from: &str, cond: impl Fn(u8) -> bool) -> bool {
        self.peek(from).is_ok_and(|c| {
            let adv = cond(c);
            adv.then(|| self.move_by(1));
            adv
        })
    }

    fn advance_if_eq(&self, from: &str, char: u8) -> bool {
        self.advance_if(from, |x| x == char)
    }

    fn advance_if_eq2(&self, from: &str, chars: [u8; 2]) -> bool {
        self.advance_if2(from, |x| x == chars[0], |x| x == chars[1])
    }

    fn advance_if2(
        &self,
        from: &str,
        cond1: impl Fn(u8) -> bool,
        cond2: impl Fn(u8) -> bool,
    ) -> bool {
        self.peek(from)
            .and_then(|x| Ok((x, self.peek_next(from)?)))
            .is_ok_and(|(f1, f2)| {
                let adv = cond1(f1) && cond2(f2);
                adv.then(|| self.move_by(2));
                adv
            })
    }

    fn advance_while(&self, from: &str, cond: impl Fn(u8) -> bool) {
        while self.advance_if(from, &cond) {}
    }

    fn move_by(&self, by: usize) {
        self.index.set(self.index.get() + by);
    }
}

#[derive(Copy, Clone)]
pub struct Token {
    pub span: Span,
    pub kind: TokenKind,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind.eq(&other.kind)
    }
}

impl Token {
    pub fn span<'a>(&self, source: &'a str) -> &'a str {
        self.span.select(source)
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct("Token")
            .field("span", &self.span)
            .field("kind", &self.kind)
            .finish()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenKind {
    Number(f64),
    Identifier(IdentId),
    Paired(PairedPunct),
    Punct(Punctuation),
    Element(Element),
    Builtins(Builtins),
    Command(Command),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Command {
    Polygon,
    Rgb,
    Hsv,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Element {
    X,
    Y,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LeftRight {
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PairedPunct {
    Paren(LeftRight),
    Square(LeftRight),
    Curly(LeftRight),
    LatexCurly(LeftRight),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Punctuation {
    Period,
    Comma,
    Arrow,
    Equals,
    With,
    For,
    Frac,
    Sqrt,
    Plus,
    Minus,
    Times,
    Exp,
    Subscript,
    LessThan,
    LessOrEqual,
    MoreThan,
    MoreOrEqual,
    Colon,
    Ellipses,
    Sum,
    Prod,
    Abs,
}

impl PairedPunct {
    pub const fn reference_str(&self) -> &'static str {
        match self {
            PairedPunct::Paren(LeftRight::Left) => "(",
            PairedPunct::Square(LeftRight::Left) => "[",
            PairedPunct::Curly(LeftRight::Left) => "\\{",
            PairedPunct::LatexCurly(LeftRight::Left) => "{",
            PairedPunct::Paren(LeftRight::Right) => ")",
            PairedPunct::Square(LeftRight::Right) => "]",
            PairedPunct::Curly(LeftRight::Right) => "\\}",
            PairedPunct::LatexCurly(LeftRight::Right) => "}",
        }
    }
}

impl Punctuation {
    pub const fn reference_str(&self) -> &'static str {
        match self {
            Punctuation::Period => ".",
            Punctuation::Comma => ",",
            Punctuation::Arrow => "\\to",
            Punctuation::Equals => "=",
            Punctuation::With => "\\operatorname{with}",
            Punctuation::For => "\\operatorname{for}",
            Punctuation::Frac => "\\frac",
            Punctuation::Sqrt => "\\sqrt",
            Punctuation::Plus => "+",
            Punctuation::Minus => "-",
            Punctuation::Times => "\\cdot",
            Punctuation::Exp => "^",
            Punctuation::Subscript => "_",
            Punctuation::LessThan => "<",
            Punctuation::LessOrEqual => "\\le",
            Punctuation::MoreThan => ">",
            Punctuation::MoreOrEqual => "\\ge",
            Punctuation::Colon => ":",
            Punctuation::Ellipses => "...",
            Punctuation::Sum => "\\sum",
            Punctuation::Prod => "\\prod",
            Punctuation::Abs => "|",
        }
    }
}

impl TokenKind {
    pub fn left(punct: impl FnOnce(LeftRight) -> PairedPunct) -> Self {
        Self::Paired(punct(LeftRight::Left))
    }

    pub fn right(punct: impl FnOnce(LeftRight) -> PairedPunct) -> Self {
        Self::Paired(punct(LeftRight::Right))
    }
}
