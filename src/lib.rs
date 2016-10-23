//Copyright 2016 William Cody Laeder
//
//Licensed under the Apache License, Version 2.0 (the "License");
//you may not use this file except in compliance with the License.
//You may obtain a copy of the License at
//
//http://www.apache.org/licenses/LICENSE-2.0

//Unless required by applicable law or agreed to in writing, software
//distributed under the License is distributed on an "AS IS" BASIS,
//WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//See the License for the specific language governing permissions and
//limitations under the License.

//!Rust Lex
//!
//!The goal of this crate is to make interacting with the AST construct parts of Rust a little
//!easier. It is more or less a high level wrapping around the various syntex_syntax crates.
//!Those crates have partially complete documentation. This crate is the result of a few days of
//!trial and error.
//!
//!Some of the public types will force you to jump into the syntex crates, but it should only be
//!inspecting tokens/error messages.


extern crate syntex_syntax;
extern crate syntex_errors;
use syntex_syntax::parse::ParseSess;
use syntex_syntax::parse::parse_tts_from_source_str;
use std::io::prelude::*;
use std::fs::File;
//exporting types
pub use std::io::Error as IOErr;
pub use syntex_syntax::tokenstream::{TokenTree,Delimited,SequenceRepetition};
pub use syntex_syntax::parse::token::{Token,DelimToken};
pub use syntex_errors::DiagnosticBuilder;




///Error Wrapper
///
///This is a high level error wrapper. It lets you know what went wrong, and where. The syntex
///error case does not implement
pub enum Fault<'a> {
    FileOpen(IOErr),
    FileMeta(IOErr),
    FileRead(IOErr),
    SyntexErr(DiagnosticBuilder<'a>)
}

///This type is used internally with private methods
type TokenIter = ::std::vec::IntoIter<Token>;
///For deconstructor a branch of TokenTree into a usable type
pub fn sr( s: &SequenceRepetition ) -> TokenIter {
    let mut v = Vec::with_capacity(100);
    match s.separator {
        Option::None => {
            for t in s.tts.iter().flat_map(tt) {
                v.push(t);
            }
        },
        Option::Some(ref x) => {
            for t in s.tts.iter().flat_map(tt) {
                v.push(t);
                v.push(x.clone());
            }
            v.pop();
        }
    };
    v.into_iter()
}
///For deconstructor a branch of TokenTree into a usable type
pub fn dr(d: &Delimited) -> TokenIter {
    let mut v = Vec::with_capacity(100);
    v.push(Token::OpenDelim(d.delim.clone()));
    for t in d.tts.iter().flat_map(tt) {
        v.push(t);
    }
    v.push(Token::CloseDelim(d.delim.clone()));
    v.into_iter()
}
///For dispatching to dr/sr for deconstructing TokenTrees
pub fn tt(t: &TokenTree) -> TokenIter {
    match t {
        &TokenTree::Token(_,ref t) => vec![t.clone()].into_iter(),
        &TokenTree::Delimited(_,ref x) => dr(x),
        &TokenTree::Sequence(_,ref x) => sr(x)
    }
}

///Flatten TokenTree
///
///I use this functionality it is included in the Parser object. But this free function does
///the same task. This is a slightly memory intensive process as internal vectors get generated
///fairly adhoc.
pub fn flatten_token_tree(v: &Vec<TokenTree>) -> Vec<Token> {
    v.iter().flat_map(tt).collect()
}


///Parser type
///
///This is a high level object that wraps the file loader/parser.his doesn't do much special.
///The ParseSess object must be borrowed by the parsing processing. So this class ensures this
///is happening without much drama or requiring you to peer into the syntex_syntax internals.
///
///There are methods to load external files, and borrow a string. That'll be immediately cloned
///just fyi.
pub struct Parser {
    p: ParseSess,
}
impl Parser {
    ///Construct a new parser.
    pub fn new() -> Parser {
        Parser {
            p: ParseSess::new()
        }
    }
    ///Get a vector of all tokens, or get an error. Loads a file
    pub fn get_tokens<'a>(&'a self, path: &str) -> Result<Vec<Token>,Fault<'a>> {
        let mut f = match File::open(path) {
            Ok(x) => x,
            Err(e) => return Err(Fault::FileOpen(e))
        };
        let len = match f.metadata() {
            Ok(x) => x.len() as usize,
            Err(e) => return Err(Fault::FileMeta(e))
        };
        let mut buffer = String::with_capacity(len+10);
        match f.read_to_string(&mut buffer) {
            Ok(_) => { },
            Err(e) => return Err(Fault::FileRead(e))
        };
        //parse buffer
        match parse_tts_from_source_str("<anon>".to_string(), buffer, vec![],&self.p) {
            Ok(ref x) => Ok(flatten_token_tree(x)),
            Err(e) => Err(Fault::SyntexErr(e))
        }
    }
    ///Get the token tree, or get an error. Loads a file
    pub fn get_tokentree<'a>(&'a self, path: &str) -> Result<Vec<TokenTree>,Fault<'a>> {
        let mut f = match File::open(path) {
            Ok(x) => x,
            Err(e) => return Err(Fault::FileOpen(e))
        };
        let len = match f.metadata() {
            Ok(x) => x.len() as usize,
            Err(e) => return Err(Fault::FileMeta(e))
        };
        let mut buffer = String::with_capacity(len+10);
        match f.read_to_string(&mut buffer) {
            Ok(_) => { },
            Err(e) => return Err(Fault::FileRead(e))
        };
        //parse buffer
        match parse_tts_from_source_str("<anon>".to_string(), buffer, vec![],&self.p) {
            Ok(x) => Ok(x),
            Err(e) => Err(Fault::SyntexErr(e))
        }
    }
    ///Get a flat list of tokens, while using pre-loaded code. Does not file IO
    pub fn get_tokens_local<'a>(&'a self, code: &str) -> Result<Vec<Token>,Fault<'a>> {
        //parse buffer
        match parse_tts_from_source_str("<anon>".to_string(), code.to_string(), vec![],&self.p) {
            Ok(ref x) => Ok(flatten_token_tree(x)),
            Err(e) => Err(Fault::SyntexErr(e))
        }
    }
    ///Get token tree, while using pre-loaded code. Does not file IO
    pub fn get_tokenstree_local<'a>(&'a self, code: &str) -> Result<Vec<TokenTree>,Fault<'a>> {
        //parse buffer
        match parse_tts_from_source_str("<anon>".to_string(), code.to_string(), vec![],&self.p) {
            Ok(x) => Ok(x),
            Err(e) => Err(Fault::SyntexErr(e))
        }
    }
}
