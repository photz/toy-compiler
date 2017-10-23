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
  Assignment
}

#[derive(Debug)]
enum Ast {
  Term { factors: Vec<i32> },
  Factor { value: i32 }
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

  let mut iter =  contents.chars();
  let mut tokens = Vec::new();

  loop {

    let token: Option<Token> = match iter.next() {
      None => Some(Token::Eof),
      Some(c) => match c {
        '(' => Some(Token::LeftParenthesis),
        ')' => Some(Token::RightParenthesis),
        '+' => Some(Token::Add),
        '-' => Some(Token::Sub),
        '*' => Some(Token::Mul),
        '/' => Some(Token::Div),
        ' ' => None,
        '=' => Some(Token::Assignment),
        '0'...'9' => {
          let number_str: String = iter
            .by_ref()
            .take_while(|c: &char| char::is_digit(*c, 10))
            .collect();

          let mut complete_number_str = String::new();
          complete_number_str.push(c);
          complete_number_str.push_str(&number_str);

          let number: i32 = complete_number_str
            .parse()
            .unwrap();

          Some(Token::Number { value: number })
        },

        'a'...'z' => {
          let id: String = iter
            .by_ref()
            .take_while(|c: &char| char::is_digit(*c, 10) ||
                        char::is_alphabetic(*c))
            .collect();

          let mut complete_id = String::new();
          complete_id.push(c);
          complete_id.push_str(&id);

          Some(Token::Identifier { name: complete_id })
        },

        _ => Some(Token::InvalidToken)
      }
    };

    match token {
      Some(Token::Eof) => {
        tokens.push(Token::Eof);
        break
      },
      Some(t) => tokens.push(t),
      _ => {}
    }
  }

  Ok(tokens)
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

      parse_stmt(&mut tokens);
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


