#[derive(Debug, PartialEq, Clone)]
enum Token { //<'a> 
  Add,
  Sub,
  Mul,
  Div,
  Number { value: i32 },
  LeftParenthesis,
  RightParenthesis,
  InvalidToken,
  //Identifier { name: &'a str },
  Identifier { name: String },
  Eof,
  Assignment,
  Semi
}

#[derive(Debug)]
enum Ast {
  Term { factors: Vec<i32> },
  Factor { value: i32 }
}

trait Tokenizer {
  fn tokenize(&self) -> Vec<Token>;
}

use std::iter::Peekable;
use std::str::Chars;

fn consume_while<F>(it: &mut Peekable<Chars>, x: F)
  -> Vec<char>
  where F : Fn(char) -> bool {

  let mut v: Vec<char> = vec![];

  while let Some(&ch) = it.peek() {
    if x(ch) {
      it.next().unwrap();
      v.push(ch);
    }
    else {
      break;
    }
  }

  v
}

impl Tokenizer for String {
  fn tokenize(&self) -> Vec<Token> {
    let mut it = self.chars().peekable();
    let mut tokens: Vec<Token> = vec![];

    loop {
      match it.peek() {
        Some(&ch) => match ch {
          '0' ... '9' => {
            
            let s: String = consume_while(&mut it, |a| a.is_numeric())
              .into_iter()
              .collect();

            let num: i32 = s.parse::<i32>().unwrap();

            tokens.push(Token::Number { value: num });
          },
          'a' ... 'z' => {
            let s: String = consume_while(&mut it, |a| a.is_numeric() ||
                                          a.is_alphabetic())
              .into_iter()
              .collect();

            tokens.push(Token::Identifier { name : s });
          },
          '-' => {
            it.next().unwrap();
            tokens.push(Token::Sub);
          },
          '*' => {
            it.next().unwrap();
            tokens.push(Token::Mul);
          },
          '/' => {
            it.next().unwrap();
            tokens.push(Token::Div);
          },
          '+' => {
            it.next().unwrap();
            tokens.push(Token::Add);
          },
          '(' => {
            it.next().unwrap();
            tokens.push(Token::LeftParenthesis);
          },
          ')' => {
            it.next().unwrap();
            tokens.push(Token::RightParenthesis);
          },
          '=' => {
            it.next().unwrap();
            tokens.push(Token::Assignment);
          },
          ' ' => {
            it.next().unwrap();
          },
          ';' => {
            it.next().unwrap();
            tokens.push(Token::Semi);
          },
          _ => panic!("invalid char")
        },
        None => break
      }
    }

    tokens
  }
}

fn parse_stmt(tokens: &mut Vec<Token>) -> Option<Ast> {
  println!("parse_stmt: {:?}", tokens);

  match tokens.is_empty() {

    true => None,

    false => match tokens.pop() {
      
      Some(Token::Identifier { name }) => {

        match tokens.iter().last() {

          Some(&Token::Assignment) => {

            tokens.pop();

            parse_expr(tokens);

            Some(Ast::Term { factors: vec![] })
          },
          _ => None
        }
      },

      _ => None

    }
  }
}

fn parse_expr(tokens: &mut Vec<Token>) -> Ast {

  println!("parse_expr: {:?}", tokens);

  parse_term(tokens);

  loop {
    match tokens.is_empty() {

      true => break,

      false => match tokens[tokens.len()-1] {

        Token::Add => {
          tokens.pop();
          parse_term(tokens);
        },

        Token::Sub => {
          tokens.pop();
          parse_term(tokens);
        },

        _ => break

      }

    }
  }
  
  Ast::Factor { value: 123 }
}

fn parse_term(tokens: &mut Vec<Token>) -> Ast {
  println!("parse_term: {:?}", tokens);

  parse_factor(tokens).expect("");

  loop {
    match tokens.is_empty() {
      false => match tokens[tokens.len()-1] {
        Token::Mul => {
          tokens.pop();
          parse_factor(tokens).expect("");
        },
        _ => break
      },
      true => break
    }
  }

  Ast::Term { factors: vec![] }
}


fn parse_factor(tokens: &mut Vec<Token>) -> Option<Ast> {
  println!("parse_factor: {:?}", tokens);

  match tokens.pop() {
    Some(Token::Number { value }) => Some(Ast::Factor { value: value }),

    Some(Token::LeftParenthesis) => {
      parse_expr(tokens);

      match tokens.pop() {
        Some(Token::RightParenthesis) =>
          Some(Ast::Factor { value: 123 }),

        _ => None
      }
      
    },

    _ => None
  }
}



fn tokenize(filename: &str) -> Result<Vec<Token>, std::io::Error> {
  let mut contents = String::new();

  {
    use std::io::Read;
    let mut f = std::fs::File::open(filename)?;
    f.read_to_string(&mut contents)?;
  }

  Ok(contents.tokenize())
}

fn main() {
  let args: Vec<String> = std::env::args().collect();

  match tokenize(&args[1]) {
    Err(_) => println!("token error"),

    Ok(mut tokens) => {
      tokens.pop();

      for (i, token) in tokens.iter().enumerate() {
        println!("{:?}: {:?}", i, token);
      }

      //parse_stmt(&mut tokens);
    }
  }

  // let mut tokens = vec![
  //   Token::Number { value: 4 },
  //   Token::Mul,
  //   Token::Number { value : 3 },
  //   Token::Add,
  //   Token::Number { value : 2},
  //   Token::Mul,
  //   Token::Number { value : 1 },
  //   Token::Sub,
  //   Token::RightParenthesis,
  //   Token::Number { value : 0 },
  //   Token::LeftParenthesis
  // ];


  // let new_tokens = parse_stmt(&mut tokens).expect("failed");

  // println!("new_tokens: {:?}", tokens);

}


