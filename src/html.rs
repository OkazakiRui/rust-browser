use crate::dom;
use dom::Node;
use std::collections::HashMap;

struct Parser {
    pos: usize,
    input: String,
}

impl Parser {
    // 現在の文字を読み取る
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    // 文字が与えられた文字列で始まるか
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    // すべての文字が解析できた場合、true を返す
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    // 現在の文字を返す
    // self.pos の値を1文字分ずらす
    fn consume_char(&mut self) -> char {
        // マルチバイト文字を処理できるようにする
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    // false を返すまで文字を解析します
    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    // スペース文字を無視
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    // タグまたは属性名を解析
    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => true,
            _ => false,
        })
    }

    // nodeが1件の場合
    fn parse_node(&mut self) -> dom::Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    // text nodeが1件の場合
    fn parse_text(&mut self) -> dom::Node {
        dom::text(self.consume_while(|c| c != '<'))
    }

    // タグを解析する <, >, /, tag_name, attribute
    fn parse_element(&mut self) -> dom::Node {
        // < を読み取り、次の文字を読み取る
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        assert!(self.consume_char() == '>');

        // nodeの内容を解析
        let children = self.parse_nodes();

        // 閉じタグを読み取る
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        dom::elem(tag_name, attrs, children)
    }

    // name="value" のペアを解析し、属性値として取得する
    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        (name, value)
    }

    // "" の中身
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        value
    }

    // 空白で区切られた属性を解析する
    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }
        attributes
    }

    // 子ノードを解析するために、閉じタグに到達するまでループを再帰的に実行する
    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }
}

// HTMLを解析し、htmlタグを返却する
pub fn parse(source: String) -> dom::Node {
    let mut nodes = Parser {
        pos: 0,
        input: source,
    }
    .parse_nodes();

    // htmlタグが存在しない場合は作成する
    if nodes.len() == 1 {
        nodes.swap_remove(0)
    } else {
        dom::elem(String::from("html"), HashMap::new(), nodes)
    }
}
