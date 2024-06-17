use crate::*;

const PREC_NONE: usize = 0;
const PREC_NEGATE: usize = 2;
const PREC_NOT: usize = 2;
const PREC_ADDRESS_OF: usize = 2;
const PREC_DEREF: usize = 2;
const PREC_TYPE_CAST: usize = 3;
const PREC_MULTIPLY: usize = 4;
const PREC_DIVIDE: usize = 4;
const PREC_REMAINDER: usize = 4;
const PREC_ADD: usize = 5;
const PREC_SUBTRACT: usize = 5;
const PREC_LESS_THAN: usize = 6;
const PREC_LESS_THAN_EQUAL: usize = 6;
const PREC_GREATER_THAN: usize = 6;
const PREC_GREATER_THAN_EQUAL: usize = 6;
const PREC_EQUAL: usize = 7;
const PREC_NOT_EQUAL: usize = 7;
const PREC_AND: usize = 8;
const PREC_OR: usize = 8;

fn get_infix_precedence(t: TokenType) -> usize {
    match t {
        TokenType::KeywordAs => PREC_TYPE_CAST,
        TokenType::Asterisk => PREC_MULTIPLY,
        TokenType::Slash => PREC_DIVIDE,
        TokenType::Percent => PREC_REMAINDER,
        TokenType::Plus => PREC_ADD,
        TokenType::Minus => PREC_SUBTRACT,
        TokenType::LessThan => PREC_LESS_THAN,
        TokenType::LessThanEqual => PREC_LESS_THAN_EQUAL,
        TokenType::GreaterThan => PREC_GREATER_THAN,
        TokenType::GreaterThanEqual => PREC_GREATER_THAN_EQUAL,
        TokenType::DoubleEqual => PREC_EQUAL,
        TokenType::NotEqual => PREC_NOT_EQUAL,
        TokenType::DoubleAmpersand => PREC_AND,
        TokenType::DoublePipe => PREC_OR,
        _ => PREC_NONE
    }
}


pub struct Parser<'c> {
    comp: &'c mut Compiler,
    lexer: Lexer,
    last: Option<Token>,
    current: Token
}

impl<'c> Parser<'c> {
    pub fn new(comp: &'c mut Compiler, mut lexer: Lexer) -> Parser<'c> {
        let current: Token = lexer.next(comp);
        return Parser { comp, lexer, last: None, current };
    }

    fn next(&mut self) {
        self.last = Some(self.current);
        self.current = self.lexer.next(self.comp);
    }

    fn report_unexpected(&mut self) -> AstNode {
        if self.current.t != TokenType::Invalid {
            let e: String = format!(
                "unexpected {}{}",
                self.current.display(self.comp),
                self.last.map(|t| 
                    format!(" after {}", t.display(self.comp))
                ).unwrap_or_else(|| String::from(""))
            );
            let s: Source = if self.current.t == TokenType::EndOfFile {
                self.last.unwrap_or(self.current).source
            } else { self.current.source };
            self.comp.errors.push(Error::dynamic(e, s));
        }
        loop {
            let recovered: bool = match self.current.t {
                TokenType::Semicolon |
                TokenType::BraceClose |
                TokenType::EndOfFile => true,
                _ => false
            };
            self.next();
            if recovered { break; }
        }
        return AstNode::new(
            NodeType::Invalid, self.current.source,
            NodeValue::None, Vec::new()
        );
    }

    fn expect(&mut self, tt: &[TokenType]) -> Result<(), AstNode> {
        if tt.contains(&self.current.t) {
            return Ok(());
        }
        return Err(self.report_unexpected());
    }

    fn expect_not(&mut self, tt: &[TokenType]) -> Result<(), AstNode> {
        if !tt.contains(&self.current.t) {
            return Ok(());
        }
        return Err(self.report_unexpected());
    }

    pub fn parse_file(&mut self) -> Vec<AstNode> {
        return self.parse_statements(true);
    }

    fn parse_statements(&mut self, global: bool) -> Vec<AstNode> {
        let mut nodes: Vec<AstNode> = Vec::new();
        loop {
            while self.current.t == TokenType::Semicolon {
                self.next();
            }
            let end: bool = match self.current.t {
                TokenType::BraceClose |
                TokenType::EndOfFile => true,
                _ => false
            };
            if end { break; }
            match self.parse_statement(global) {
                Ok(n) | Err(n) => nodes.push(n)
            }
        }
        return nodes;
    }

    fn parse_path(&mut self) -> Result<PathIdx, AstNode> {
        let mut p: Vec<StringIdx> = Vec::new();
        self.expect(&[TokenType::Identifier])?;
        p.push(self.current.content);
        self.next();
        while self.current.t == TokenType::DoubleColon {
            self.next();
            self.expect(&[TokenType::Identifier])?;
            p.push(self.current.content);
            self.next();
        }
        return Ok(self.comp.paths.insert(&p));
    }

    fn parse_t_args_def(&mut self) -> Result<AstNode, AstNode> {
        if self.current.t != TokenType::BracketOpen {
            return Ok(AstNode::new(
                NodeType::ArgumentList, self.current.source,
                NodeValue::None, Vec::new()
            ));
        }
        let start: Source = self.current.source;
        self.next();
        let mut args: Vec<AstNode> = Vec::new();
        while self.current.t != TokenType::BracketClose {
            self.expect(&[TokenType::Identifier])?;
            args.push(AstNode::new(
                NodeType::ArgumentDecl, self.current.source,
                NodeValue::String(self.current.content),
                Vec::new()
            ));
            self.next();
            self.expect(&[TokenType::Comma, TokenType::BracketClose])?;
            if self.current.t == TokenType::Comma { self.next(); }
        }
        let end: Source = self.current.source;
        self.next();
        return Ok(AstNode::new(
            NodeType::ArgumentList, Source::across(start, end),
            NodeValue::None, args
        ));
    }

    fn parse_args_def(&mut self) -> Result<AstNode, AstNode> {
        self.expect(&[TokenType::ParenOpen])?;
        let start: Source = self.current.source;
        self.next();
        let mut args: Vec<AstNode> = Vec::new();
        while self.current.t != TokenType::ParenClose {
            let mut arg_children: Vec<AstNode> = Vec::new();
            let arg_start: Source = self.current.source;
            if self.current.t == TokenType::KeywordConst {
                arg_children.push(AstNode::empty(
                    NodeType::IsConstant, self.current.source,
                ));
                self.next();
            }
            self.expect(&[TokenType::Identifier])?;
            let arg_name: StringIdx = self.current.content;
            self.next();
            let arg_type: AstNode = self.parse_type()?;
            let arg_end: Source = arg_type.source;
            arg_children.push(arg_type);
            args.push(AstNode::new(
                NodeType::ArgumentDecl,
                Source::across(arg_start, arg_end),
                NodeValue::String(arg_name),
                arg_children
            ));
            self.expect(&[TokenType::Comma, TokenType::ParenClose])?;
            if self.current.t == TokenType::Comma { self.next(); }
        }
        let end: Source = self.current.source;
        self.next();
        return Ok(AstNode::new(
            NodeType::ArgumentList,
            Source::across(start, end),
            NodeValue::None,
            args
        ));
    }

    fn parse_block(&mut self) -> Result<AstNode, AstNode> {
        self.expect(&[TokenType::BraceOpen])?;
        let start: Source = self.current.source;
        self.next();
        let body: Vec<AstNode> = self.parse_statements(false);
        self.expect(&[TokenType::BraceClose])?;
        let end: Source = self.current.source;
        self.next();
        return Ok(AstNode::new(
            NodeType::Block, 
            Source::across(start, end),
            NodeValue::None,
            body
        ));
    }

    fn parse_t_args(&mut self) -> Result<AstNode, AstNode> {
        if self.current.t != TokenType::BracketOpen {
            return Ok(AstNode::new(
                NodeType::ArgumentList, self.current.source,
                NodeValue::None, Vec::new()
            ));
        }
        let start: Source = self.current.source;
        self.next();
        let mut args: Vec<AstNode> = Vec::new();
        while self.current.t != TokenType::BracketClose {
            args.push(self.parse_type()?);
            self.expect(&[TokenType::Comma, TokenType::BracketClose])?;
            if self.current.t == TokenType::Comma { self.next(); }
        }
        let end: Source = self.current.source;
        self.next();
        return Ok(AstNode::new(
            NodeType::ArgumentList,
            Source::across(start, end),
            NodeValue::None,
            args
        ));
    }

    fn parse_used_paths(&mut self) -> Result<Vec<Vec<StringIdx>>, AstNode> {
        let mut paths: Vec<Vec<StringIdx>> = Vec::new();
        loop {
            match self.current.t {
                TokenType::Asterisk | TokenType::Identifier => {
                    if paths.len() == 0 {
                        paths.push(vec!(self.current.content));
                    } else {
                        for path in &mut paths {
                            path.push(self.current.content);
                        }
                    }
                    self.next();
                }
                TokenType::ParenOpen => {
                    let mut new_paths: Vec<Vec<StringIdx>> = Vec::new();
                    self.next();
                    while self.current.t != TokenType::ParenClose {
                        let added_paths: Vec<Vec<StringIdx>> 
                            = self.parse_used_paths()?;
                        for base in &paths {
                            for added in &added_paths {
                                let mut new_path: Vec<StringIdx> = base.clone();
                                new_path.append(&mut added.clone());
                                new_paths.push(new_path);
                            }
                        }
                        self.expect(&[
                            TokenType::Comma, TokenType::ParenClose
                        ])?;
                        if self.current.t == TokenType::Comma { self.next(); }
                    }
                    self.next();
                    paths = new_paths;
                }
                _ => return Err(self.report_unexpected())
            }
            if self.current.t != TokenType::DoubleColon {
                break;
            }
            self.next();
        }
        return Ok(paths);
    }

    fn parse_statement(&mut self, global: bool) -> Result<AstNode, AstNode> {
        let start: Source = self.current.source;
        let is_public: bool = self.current.t == TokenType::KeywordPub;
        if is_public && global {
            self.next();
            self.expect(&[
                TokenType::KeywordStruct, TokenType::KeywordEnum,
                TokenType::KeywordInterface, TokenType::KeywordFun,
                TokenType::KeywordVar, TokenType::KeywordConst,
                TokenType::KeywordExt, TokenType::KeywordExp
            ])?;
        }
        let is_external: bool = self.current.t == TokenType::KeywordExt;
        let is_exported: bool = self.current.t == TokenType::KeywordExp;
        if (is_external || is_exported) && global {
            self.next();
            self.expect(&[
                TokenType::KeywordFun,
                TokenType::KeywordVar, TokenType::KeywordConst
            ])?;
        }
        if global {
            self.expect(&[
                TokenType::KeywordMod, TokenType::KeywordUse,
                TokenType::KeywordStruct, TokenType::KeywordEnum,
                TokenType::KeywordInterface, TokenType::KeywordFun,
                TokenType::KeywordVar, TokenType::KeywordConst
            ])?;
        } else {
            self.expect_not(&[
                TokenType::KeywordMod, TokenType::KeywordUse,
                TokenType::KeywordStruct, TokenType::KeywordEnum,
                TokenType::KeywordInterface, TokenType::KeywordFun
            ])?;
        }
        match self.current.t {
            TokenType::KeywordMod => {
                self.next();
                let name: PathIdx = self.parse_path()?;
                return Ok(AstNode::new(
                    NodeType::ModuleDecl,
                    Source::across(
                        start, 
                        self.last.expect("cannot be the first token").source
                    ),
                    NodeValue::Path(name),
                    Vec::new()
                ));
            }
            TokenType::KeywordUse => {
                self.next();
                let p_source: Source = self.current.source;
                let children: Vec<AstNode> = self.parse_used_paths()?
                    .iter()
                    .map(|p| AstNode::new(
                        NodeType::UsedPath, p_source,
                        NodeValue::Path(self.comp.paths.insert(p)), 
                        Vec::new()
                    ))
                    .collect();
                return Ok(AstNode::new(
                    NodeType::UsageDecl,
                    Source::across(
                        start, 
                        self.last.expect("cannot be the first token").source
                    ),
                    NodeValue::None,
                    children
                ));
            }
            TokenType::KeywordStruct => {
                let mut children: Vec<AstNode> = Vec::new();
                if is_public {
                    children.push(AstNode::empty(
                        NodeType::IsPublic, self.current.source
                    ));
                }
                self.next();
                self.expect(&[TokenType::Identifier])?;
                let name: StringIdx = self.current.content;
                self.next();
                children.push(self.parse_t_args_def()?);
                let args: AstNode = self.parse_args_def()?;
                let end: Source = args.source;
                children.push(args);
                if self.current.t == TokenType::Colon {
                    self.next();
                    todo!(
                        "interface parsing + make 'end' mut and update it"
                    );
                }
                return Ok(AstNode::new(
                    NodeType::StructDecl,
                    Source::across(start, end),
                    NodeValue::String(name),
                    children
                ));
            }
            TokenType::KeywordEnum => {
                todo!()
            }
            TokenType::KeywordInterface => {
                todo!()
            }
            TokenType::KeywordFun => {
                let mut children: Vec<AstNode> = Vec::new();
                if is_public {
                    children.push(AstNode::empty(
                        NodeType::IsPublic, self.current.source
                    ));
                }
                if is_external {
                    children.push(AstNode::empty(
                        NodeType::IsExternal, self.current.source
                    ));
                }
                if is_exported {
                    children.push(AstNode::empty(
                        NodeType::IsExported, self.current.source
                    ));
                }
                self.next();
                self.expect(&[TokenType::Identifier])?;
                let name: StringIdx = self.current.content;
                self.next();
                let t_args: AstNode = self.parse_t_args_def()?;
                if is_exported && t_args.children.len() > 0 {
                    self.comp.errors.push(Error::dynamic(
                        format!(
                            concat!(
                                "'{}' is exported and may therefore not ",
                                "specify any template arguments"
                            ),
                            self.comp.strings.get(name)
                        ), 
                        t_args.source
                    ))
                }
                children.push(t_args);
                children.push(self.parse_args_def()?);
                if self.current.t == TokenType::Colon {
                    self.next();
                    children.push(self.parse_type()?);
                } else {
                    children.push(
                        AstNode::empty(NodeType::UnitType, self.current.source)
                    )
                }
                if !is_external {
                    children.push(self.parse_block()?);
                }
                return Ok(AstNode::new(
                    NodeType::FunctionDecl,
                    Source::across(
                        start, self.last.expect("cannot be first").source
                    ),
                    NodeValue::String(name),
                    children
                ));
            }
            TokenType::KeywordVar => {
                let mut children: Vec<AstNode> = Vec::new();
                if is_public {
                    children.push(AstNode::empty(
                        NodeType::IsPublic, self.current.source
                    ));
                }
                if is_external {
                    children.push(AstNode::empty(
                        NodeType::IsExternal, self.current.source
                    ));
                }
                if is_exported {
                    children.push(AstNode::empty(
                        NodeType::IsExported, self.current.source
                    ));
                }
                self.next();
                self.expect(&[TokenType::Identifier])?;
                let name: StringIdx = self.current.content;
                self.next();
                let value_type: AstNode = self.parse_type()?;
                let mut end: Source = value_type.source;
                children.push(value_type);
                if (self.current.t == TokenType::Equal || global)
                        && !is_external {
                    self.expect(&[TokenType::Equal])?;
                    self.next();
                    let value: AstNode = self.parse_full_expression()?;
                    end = value.source;
                    children.push(value);
                }
                return Ok(AstNode::new(
                    NodeType::VariableDecl,
                    Source::across(start, end),
                    NodeValue::String(name),
                    children
                ));
            }
            TokenType::KeywordConst => {
                let mut children: Vec<AstNode> = Vec::new();
                if is_public {
                    children.push(AstNode::empty(
                        NodeType::IsPublic, self.current.source
                    ));
                }
                if is_external {
                    children.push(AstNode::empty(
                        NodeType::IsExternal, self.current.source
                    ));
                }
                if is_exported {
                    children.push(AstNode::empty(
                        NodeType::IsExported, self.current.source
                    ));
                }
                self.next();
                self.expect(&[TokenType::Identifier])?;
                let name: StringIdx = self.current.content;
                self.next();
                let value_type: AstNode = self.parse_type()?;
                let mut end: Source = value_type.source;
                children.push(AstNode::new(
                    NodeType::IsConstant, start,
                    NodeValue::None, Vec::new()
                ));
                children.push(value_type);
                if (self.current.t == TokenType::Equal || global)
                        && !is_external {
                    self.expect(&[TokenType::Equal])?;
                    self.next();
                    let value: AstNode = self.parse_full_expression()?;
                    end = value.source;
                    children.push(value);
                }
                return Ok(AstNode::new(
                    NodeType::VariableDecl,
                    Source::across(start, end),
                    NodeValue::String(name),
                    children
                ));
            }
            TokenType::KeywordReturn => {
                self.next();
                let returned: AstNode = self.parse_full_expression()?;
                return Ok(AstNode::new(
                    NodeType::Return,
                    Source::across(start, returned.source),
                    NodeValue::None,
                    vec!(returned)
                ));
            }
            TokenType::KeywordContinue => {
                self.next();
                return Ok(AstNode::new(
                    NodeType::Continue,
                    start,
                    NodeValue::None, Vec::new()
                ));
            }
            TokenType::KeywordBreak => {
                self.next();
                return Ok(AstNode::new(
                    NodeType::Break,
                    start,
                    NodeValue::None, Vec::new()
                ));
            }
            TokenType::KeywordIf => {
                self.next();
                let mut children: Vec<AstNode> = Vec::new();
                children.push(self.parse_full_expression()?);
                children.push(self.parse_block()?);
                if self.current.t == TokenType::KeywordElse {
                    self.next();
                    self.expect(&[TokenType::KeywordIf, TokenType::BraceOpen])?;
                    if self.current.t == TokenType::BraceOpen {
                        children.push(self.parse_block()?);
                    } else {
                        let else_st: AstNode = self.parse_statement(false)?;
                        children.push(AstNode::new(
                            NodeType::Block, else_st.source,
                            NodeValue::None, vec!(else_st)
                        ));
                    }
                } else {
                    children.push(AstNode::empty(
                        NodeType::Block, 
                        self.last.expect("cannot be first").source
                    ));
                }
                return Ok(AstNode::new(
                    NodeType::If, 
                    Source::across(
                        start, self.last.expect("cannot be first").source
                    ), 
                    NodeValue::None, 
                    children
                ));
            }
            TokenType::KeywordLoop => {
                self.next();
                let body: AstNode = self.parse_block()?;
                return Ok(AstNode::new(
                    NodeType::Loop, Source::across(start, body.source),
                    NodeValue::None, vec!(body)
                ));
            }
            TokenType::KeywordWhile => {
                self.next();
                let condition: AstNode = self.parse_full_expression()?;
                let body: AstNode = self.parse_block()?;
                return Ok(AstNode::new(
                    NodeType::While, Source::across(start, body.source),
                    NodeValue::None, vec!(condition, body)
                ));
            }
            _ => {
                let expr: AstNode = self.parse_full_expression()?;
                match self.current.t {
                    TokenType::Equal |
                    TokenType::PlusEqual | TokenType::MinusEqual |
                    TokenType::AsteriskEqual | TokenType::SlashEqual |
                    TokenType::PercentEqual => {
                        let nt: NodeType = match self.current.t {
                            TokenType::Equal => NodeType::Assign,
                            TokenType::PlusEqual => NodeType::AssignAdd,
                            TokenType::MinusEqual => NodeType::AssignSubtract,
                            TokenType::AsteriskEqual => NodeType::AssignMultiply,
                            TokenType::SlashEqual => NodeType::AssignDivide,
                            TokenType::PercentEqual => NodeType::AssignRemainder,
                            _ => unreachable!()
                        };
                        self.next();
                        let value: AstNode = self.parse_full_expression()?;
                        return Ok(AstNode::new(
                            nt, Source::across(start, value.source),
                            NodeValue::None, vec!(expr, value)
                        )); 
                    }
                    _ => return Ok(expr)
                }
            }
        }
    }

    fn parse_full_expression(&mut self) -> Result<AstNode, AstNode> {
        return self.parse_expression(usize::MAX);
    }

    fn parse_expression(
        &mut self, precedence: usize
    ) -> Result<AstNode, AstNode> {
        let mut previous: Option<AstNode> = None;
        loop {
            let start: Source = self.current.source;
            // infix operators
            if let Some(left) = previous {
                if get_infix_precedence(self.current.t) >= precedence {
                    return Ok(left);
                }
                match self.current.t {
                    TokenType::ParenOpen => {
                        self.next();
                        let mut args: Vec<AstNode> = Vec::new();
                        while self.current.t != TokenType::ParenClose {
                            args.push(self.parse_full_expression()?);
                            self.expect(&[
                                TokenType::ParenClose, TokenType::Comma
                            ])?;
                            if self.current.t == TokenType::Comma {
                                self.next();
                            }
                        }
                        let end: Source = self.current.source;
                        self.next();
                        previous = Some(AstNode::new(
                            NodeType::Call, Source::across(left.source, end),
                            NodeValue::None,
                            vec!(
                                left,
                                AstNode::new(
                                    NodeType::ArgumentList,
                                    Source::across(start, end), 
                                    NodeValue::None, args
                                )
                            )
                        ));
                    }
                    TokenType::Dot => {
                        self.next();
                        self.expect(&[TokenType::Identifier])?;
                        let name: StringIdx = self.current.content;
                        let end: Source = self.current.source;
                        self.next();
                        previous = Some(AstNode::new(
                            NodeType::MemberAccess,
                            Source::across(left.source, end),
                            NodeValue::String(name),
                            vec!(left)
                        ));
                    }
                    TokenType::KeywordAs => {
                        self.next();
                        let target_t: AstNode = self.parse_type()?;
                        previous = Some(AstNode::new(
                            NodeType::TypeCast,
                            Source::across(left.source, target_t.source),
                            NodeValue::None,
                            vec!(left, target_t)
                        ));
                    }
                    TokenType::Plus | TokenType::Minus |
                    TokenType::Asterisk | TokenType::Slash |
                    TokenType::Percent |
                    TokenType::LessThan | TokenType::LessThanEqual |
                    TokenType::GreaterThan | TokenType::GreaterThanEqual |
                    TokenType::DoubleEqual | TokenType::NotEqual |
                    TokenType::DoublePipe | TokenType::DoubleAmpersand => {
                        let nt: NodeType = match self.current.t {
                            TokenType::Plus => NodeType::Add,
                            TokenType::Minus => NodeType::Subtract,
                            TokenType::Asterisk => NodeType::Multiply,
                            TokenType::Slash => NodeType::Divide,
                            TokenType::Percent => NodeType::Remainder,
                            TokenType::LessThan => NodeType::LessThan,
                            TokenType::LessThanEqual => NodeType::LessThanEqual,
                            TokenType::GreaterThan => NodeType::GreaterThan,
                            TokenType::GreaterThanEqual => NodeType::GreaterThanEqual,
                            TokenType::DoubleEqual => NodeType::Equal,
                            TokenType::NotEqual => NodeType::NotEqual,
                            TokenType::DoublePipe => NodeType::LogicalOr,
                            TokenType::DoubleAmpersand => NodeType::LogicalAnd,
                            _ => unreachable!()
                        };
                        let pr: usize = match self.current.t {
                            TokenType::Plus => PREC_ADD,
                            TokenType::Minus => PREC_SUBTRACT,
                            TokenType::Asterisk => PREC_MULTIPLY,
                            TokenType::Slash => PREC_DIVIDE,
                            TokenType::Percent => PREC_REMAINDER,
                            TokenType::LessThan => PREC_LESS_THAN,
                            TokenType::LessThanEqual => PREC_LESS_THAN_EQUAL,
                            TokenType::GreaterThan => PREC_GREATER_THAN,
                            TokenType::GreaterThanEqual => PREC_GREATER_THAN_EQUAL,
                            TokenType::DoubleEqual => PREC_EQUAL,
                            TokenType::NotEqual => PREC_NOT_EQUAL,
                            TokenType::DoublePipe => PREC_OR,
                            TokenType::DoubleAmpersand => PREC_AND,
                            _ => unreachable!()
                        };
                        self.next();
                        let right: AstNode = self.parse_expression(pr)?;
                        previous = Some(AstNode::new(
                            nt, Source::across(left.source, right.source),
                            NodeValue::None, vec!(left, right)
                        ));
                    }
                    _ => return Ok(left)
                }
                continue;
            }
            // prefix operators and literals
            match self.current.t {
                TokenType::Identifier => {
                    let name: StringIdx = self.current.content;
                    self.next();
                    let t_args: AstNode = self.parse_t_args()?;
                    previous = Some(AstNode::new(
                        NodeType::NamespaceAccess, 
                        Source::across(
                            start, self.last.expect("cannot be first").source
                        ),
                        NodeValue::String(name),
                        vec!(t_args)
                    )); 
                }
                TokenType::KeywordUnit => {
                    self.next();
                    previous = Some(AstNode::new(
                        NodeType::UnitLiteral, start,
                        NodeValue::None, Vec::new()
                    ));
                }
                TokenType::KeywordSizeof => {
                    self.next();
                    let sized_type: AstNode = self.parse_type()?;
                    previous = Some(AstNode::new(
                        NodeType::SizeOf, 
                        Source::across(start, sized_type.source),
                        NodeValue::None,
                        vec!(sized_type)
                    ));
                }
                TokenType::KeywordTrue | TokenType::KeywordFalse |
                TokenType::Integer | TokenType::Float |
                TokenType::String | TokenType::CString => {
                    let nt: NodeType = match self.current.t {
                        TokenType::KeywordTrue | 
                        TokenType::KeywordFalse => NodeType::BooleanLiteral,
                        TokenType::Integer => NodeType::IntegerLiteral,
                        TokenType::Float => NodeType::FloatLiteral,
                        TokenType::String => NodeType::StringLiteral,
                        TokenType::CString => NodeType::CStringLiteral,
                        _ => unreachable!()
                    };
                    let value: StringIdx = self.current.content;
                    self.next();
                    previous = Some(AstNode::new(
                        nt, start, NodeValue::String(value), Vec::new()
                    ));
                }
                TokenType::Ampersand |
                TokenType::Asterisk |
                TokenType::ExclamationMark |
                TokenType::Minus => {
                    let nt: NodeType = match self.current.t {
                        TokenType::Ampersand => NodeType::AddressOf,
                        TokenType::Asterisk => NodeType::Deref,
                        TokenType::ExclamationMark => NodeType::LogicalNot,
                        TokenType::Minus => NodeType::Negate,
                        _ => unreachable!()
                    };
                    let pr: usize = match self.current.t {
                        TokenType::Ampersand => PREC_ADDRESS_OF,
                        TokenType::Asterisk => PREC_DEREF,
                        TokenType::ExclamationMark => PREC_NOT,
                        TokenType::Minus => PREC_NEGATE,
                        _ => unreachable!()
                    };
                    self.next();
                    let value: AstNode = self.parse_expression(pr)?;
                    previous = Some(AstNode::new(
                        nt, Source::across(start, value.source),
                        NodeValue::None, vec!(value)
                    ));
                }
                TokenType::ParenOpen => {
                    self.next();
                    let content: AstNode = self.parse_full_expression()?;
                    self.expect(&[TokenType::ParenClose])?;
                    self.next();
                    previous = Some(content);
                }
                _ => return Err(self.report_unexpected())
            }
        }
    }

    fn parse_type(&mut self) -> Result<AstNode, AstNode> {
        let start: Source = self.current.source;
        match self.current.t {
            TokenType::Asterisk => {
                self.next();
                let mut children: Vec<AstNode> = Vec::new();
                if self.current.t == TokenType::KeywordConst {
                    children.push(AstNode::empty(
                        NodeType::IsConstant, self.current.source
                    ));
                    self.next();
                }
                let ptr_type: AstNode = self.parse_type()?;
                let end: Source = ptr_type.source;
                children.push(ptr_type);
                return Ok(AstNode::new(
                    NodeType::PointerType,
                    Source::across(start, end),
                    NodeValue::None,
                    children
                ));
            }
            TokenType::KeywordU8 | TokenType::KeywordU16 |
            TokenType::KeywordU32 | TokenType::KeywordU64 |
            TokenType::KeywordS8 | TokenType::KeywordS16 |
            TokenType::KeywordS32 | TokenType::KeywordS64 |
            TokenType::KeywordF32 | TokenType::KeywordF64 |
            TokenType::KeywordUsize | TokenType::KeywordUnit |
            TokenType::KeywordBool => {
                let nt: NodeType = match self.current.t {
                    TokenType::KeywordU8 => NodeType::U8Type,
                    TokenType::KeywordU16 => NodeType::U16Type,
                    TokenType::KeywordU32 => NodeType::U32Type,
                    TokenType::KeywordU64 => NodeType::U64Type,
                    TokenType::KeywordS8 => NodeType::S8Type,
                    TokenType::KeywordS16 => NodeType::S16Type,
                    TokenType::KeywordS32 => NodeType::S32Type,
                    TokenType::KeywordS64 => NodeType::S64Type,
                    TokenType::KeywordF32 => NodeType::F32Type,
                    TokenType::KeywordF64 => NodeType::F64Type,
                    TokenType::KeywordUsize => NodeType::UsizeType,
                    TokenType::KeywordUnit => NodeType::UnitType,
                    TokenType::KeywordBool => NodeType::BoolType,
                    _ => unreachable!()
                };
                self.next();
                return Ok(AstNode::empty(nt, start));
            }
            TokenType::KeywordFun => {
                todo!()
            }
            TokenType::Identifier => {
                let name: StringIdx = self.current.content;
                self.next();
                let t_args: AstNode = self.parse_t_args()?;
                return Ok(AstNode::new(
                    NodeType::NamespaceAccess, 
                    Source::across(
                        start, self.last.expect("cannot be first").source
                    ),
                    NodeValue::String(name),
                    vec!(t_args)
                )); 
            }
            _ => return Err(self.report_unexpected())
        }
    }

}