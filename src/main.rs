#[derive(Debug,PartialEq,Clone)]
pub enum Op {
  Add,
  Sub,
  Mul,
  Div
}

#[derive(Debug, PartialEq, Clone)]
enum Token {
  Add,
  Sub,
  Mul,
  Div,
  Number { value: i32 },
  LeftParenthesis,
  RightParenthesis,
  Identifier { name: String },
  Assignment,
  Semi
}

#[derive(Debug,PartialEq)]
pub enum Expr {
  Integer(i32),
  BinaryExpr(Box<Expr>, Op, Box<Expr>),
  Identifier(String)
}

#[derive(Debug, PartialEq)]
pub enum Statement {
  Assignment(String, Expr)
}

use std::collections::HashMap;

struct LocManager {
  bitmap: usize
}

impl LocManager {
  fn new() -> Self {
    LocManager {
      bitmap: 0
    }
  }
  fn request(&mut self) -> Option<Location> {
    for i in 0..5 {
      if self.is_free(Location::Register(i)) {
        self.bitmap |= 1 << i;
        return Some(Location::Register(i));
      }
    }

    None
  }
  fn is_free(&self, l: Location) -> bool {
    match l {
      Location::Register(n) => {
        0 == (self.bitmap & (1 << n))
      },
      _ => panic!("I only accept registers")
    }
  }
  fn free(&mut self, l: Location) {
    match l {
      Location::Register(n) => {
        if !self.is_free(Location::Register(n)) {
          self.bitmap &= !(1 << n);
        }
        else {
          panic!("register {} has been returned twice", n);
        }
      },
      _ => panic!("I only accept registers")
    }
  }
}

struct SymbolTable {
  table: HashMap<String, TableEntry>
}

#[derive(Debug)]
enum TableEntry {
  Data(Data)
}

impl SymbolTable {
  fn new() -> Self {
    SymbolTable {
      table: HashMap::new()
    }
  }

  fn save(&mut self, id: &str, data: TableEntry) {
    self.table.insert(id.to_string(), data);
  }

  fn lookup(&self, id: &str) -> Option<&TableEntry> {
    self.table.get(id)
  }
}

#[derive(Debug, PartialEq, Clone)]
enum Location {
  Register(i32),
  StackIndex(i32)
}

#[derive(Debug, Clone)]
enum Data {
  Loc(Location),
  Constant(i32)
}

enum Instruction {
  Add { source: Data, dest: Location },
  Mul { factor1: Data, factor2: Data },
  Move { source: Data, dest: Location },
  Push { source: Data }
}

trait Tokenizer {
  fn tokenize(&self) -> Vec<Token>;
}



fn process_expr(e: &Expr, loc_manager: &mut LocManager, table: &SymbolTable)
                -> (Data, Vec<Instruction>) {
  match e {
    &Expr::Integer(value) => {
      (Data::Constant(value), vec![])
    },
    &Expr::BinaryExpr(ref l, ref op, ref r) => {

      let (source_l, mut instr_l) = process_expr(l, loc_manager, table);
      let (source_r, mut instr_r) = process_expr(r, loc_manager, table);

      instr_l.append(&mut instr_r);

      let free_register = loc_manager.request().unwrap();

      instr_l.push(Instruction::Move { source: source_r.clone(),
                                       dest: free_register.clone() });

      let next_instr = match op {
        &Op::Add =>
          Instruction::Add { source: source_l.clone(),
                             dest: free_register.clone() },

        &Op::Mul =>
          Instruction::Mul { factor1: source_l.clone(),
                             factor2: source_r.clone() },

        _ => panic!("can only generate code for additions and multipl.")
      };

      match source_l {
        Data::Loc(Location::Register(n)) => {
          loc_manager.free(Location::Register(n));
        },
        _ => {}
      }

      match source_r {
        Data::Loc(Location::Register(n)) => {
          loc_manager.free(Location::Register(n));
        },
        _ => {}
      }
      
      instr_l.push(next_instr);

      match op {
        &Op::Mul => {
          let backup_eax = Instruction::Move {
            source: Data::Loc(Location::Register(0)),
            dest: free_register.clone()
          };

          instr_l.push(backup_eax);
        },
        _ => {}
      }

      (Data::Loc(free_register), instr_l)
    },
    &Expr::Identifier(ref name) => {
      match table.lookup(name) {
        Some(&TableEntry::Data(ref loc)) => match loc {
          
          &Data::Constant(c) => (Data::Constant(c), vec![]),

          &Data::Loc(Location::StackIndex(i)) => {
            (Data::Loc(Location::StackIndex(i)), vec![])
          },

          x => panic!("not yet implemented {:?}", x)
        },
        None => panic!("unknown identifier {}", name)
      }
    }
  }
}

fn generate(statements: &Vec<Statement>) {

  let mut table = SymbolTable::new();

  let mut instructions = Vec::new();

  let mut stack_index = 0;

  instructions.push(Instruction::Push {
    source: Data::Constant(123)
  });

  for statement in statements {

    match statement {
      &Statement::Assignment(ref id, ref expr) => {

        match table.lookup(id) {

          None => {
            let mut loc_manager = LocManager::new();

            let (source, mut instr) = process_expr(expr,
                                                   &mut loc_manager,
                                                   &table);

            let loc = Location::StackIndex(stack_index);

            stack_index = stack_index + 1;

            let entry = TableEntry::Data(Data::Loc(loc.clone()));

            table.save(id, entry);

            instructions.append(&mut instr);
            instructions.push(Instruction::Push { source });

          },
          Some(&TableEntry::Data(Data::Loc(Location::StackIndex(i)))) => {
            println!("We already encountered {:?} before", id);

            let mut loc_manager = LocManager::new();

            let (source, mut instr) = process_expr(expr,
                                                   &mut loc_manager,
                                                   &table);

            instructions.append(&mut instr);
            
            let new_instr = Instruction::Move {
              source,
              dest: Location::StackIndex(i)
            };

            instructions.push(new_instr);
          },
          _ => panic!("unexpected value from LocManager::lookup()")
        }
      }
    }
  }

  program_to_gas(&instructions, "out.s".to_string());
}

fn program_to_gas(instructions: &Vec<Instruction>, out: String) {
  use std::fs::File;
  use std::io::prelude::*;

  let mut file = File::create(out).unwrap();

  file.write(b".global _start\n").unwrap();
  file.write(b".text\n").unwrap();
  file.write(b"_start:\n").unwrap();

  for instruction in instructions {
    
    let as_str = instr_to_str(instruction);

    file.write(as_str.as_bytes()).unwrap();
    file.write(b"\n").unwrap();
  }

  file.write(b"ret\n").unwrap();

}

fn reg_to_gas(reg: &Location) -> String {
  match reg {
    &Location::Register(n) => {
      match n {
        0 => "%rbx".to_string(),
        1 => "%rcx".to_string(),
        2 => "%rdx".to_string(),
        3 => "%r12".to_string(),
        4 => "%r13".to_string(),
        5 => "%r14".to_string(),
        _ => panic!("no such register")
      }
    },
    &Location::StackIndex(i) => {
      let mut s = String::new();
      s.push_str(&(i * 8).to_string());
      s.push_str("(%rsp)");//FIXME
      s
    }
  }
}

fn data_to_gas(data: &Data) -> String {
  match data {
    &Data::Loc(ref l) => {
      reg_to_gas(&l)
    },
    &Data::Constant(n) => {
      let mut s = "$".to_string();
      s.push_str(&n.to_string());
      s
    }
  }
}

fn instr_to_str(instruction: &Instruction) -> String {
  match instruction {
    &Instruction::Add { ref source, ref dest } => {
      let mut s = "addq ".to_string();
      s.push_str(&data_to_gas(source));
      s.push_str(", ");
      s.push_str(&reg_to_gas(dest));
      s
    },
    &Instruction::Mul { ref factor1, ref factor2 } => {
      let mut s = "imulq ".to_string();
      s.push_str(&data_to_gas(factor1));
      s.push_str(", ");
      s.push_str(&data_to_gas(factor2));
      s
    },
    &Instruction::Move { ref source, ref dest } => {
      let mut s = "movq ".to_string();
      s.push_str(&data_to_gas(source));
      s.push_str(", ");
      s.push_str(&reg_to_gas(dest));
      s
    },
    &Instruction::Push { ref source } => {
      let mut s = "pushq ".to_string();
      s.push_str(&data_to_gas(source));
      s
    }
  }
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
          'a' ... 'z' | '_' => {
            let s: String = consume_while(&mut it, |a| a.is_numeric() ||
                                          a.is_alphabetic() ||
                                          a == '_')
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
          ' ' | '\n' => {
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

fn tokenize(filename: &str) -> Result<Vec<Token>, std::io::Error> {
  let mut contents = String::new();

  {
    use std::io::Read;
    let mut f = std::fs::File::open(filename)?;
    f.read_to_string(&mut contents)?;
  }

  Ok(contents.tokenize())
}

fn parse_factor<'a, It>(it: &mut Peekable<It>)
                        -> Expr
  where It: Iterator<Item=&'a Token> {

  match it.next().unwrap() {
    &Token::Identifier { ref name } => Expr::Identifier(name.to_string()),
    &Token::Number { value } => Expr::Integer(value),
    &Token::LeftParenthesis => {
      
      let expr = parse_expr(it);

      let next = it.next().unwrap();

      match next {
        &Token::RightParenthesis => {},
        _ => panic!("unexpecting a )")
      }

      expr

    },
    _ => panic!("unexpected token")
  }

}

fn parse_term<'a, It>(it: &mut Peekable<It>)
                      -> Expr
  where It: Iterator<Item=&'a Token> {

  let mut first_expr = parse_factor(it);

  loop {
    let next = it.peek().unwrap().clone();

    match next {
      &Token::Div => {
        it.next();

        let more = parse_factor(it);

        first_expr = Expr::BinaryExpr(Box::new(first_expr),
                                      Op::Div,
                                      Box::new(more));
      },

      &Token::Mul => {
        it.next();

        let more = parse_factor(it);

        first_expr = Expr::BinaryExpr(Box::new(first_expr),
                                      Op::Mul,
                                      Box::new(more));
      },
      _ => break
    }
  }
  
  first_expr
}

fn parse_expr<'a, It>(it: &mut Peekable<It>)
                      -> Expr
  where It: Iterator<Item=&'a Token> {

  let mut first_expr = parse_term(it);

  loop {

    let next = it.peek().unwrap().clone();

    match next {
      &Token::Add => {
        it.next();

        let next_expr = parse_term(it);

        first_expr = Expr::BinaryExpr(Box::new(first_expr),
                                      Op::Add,
                                      Box::new(next_expr));
      },
      _ => break
    }

  }

  first_expr
}

fn parse<'a, It>(it: &mut Peekable<It>)
                 -> Vec<Statement>
  where It: Iterator<Item=&'a Token> {

  let mut statements = vec![];

  loop {

    let next = it.next().unwrap();

    let identifier = match next {

      &Token::Identifier { ref name } => {
        name
      },

      _ => panic!("unexpected token")

    };
    
    let next = it.next().unwrap();

    match next {
      &Token::Assignment => {},
      _ => panic!("expected a =")
    }

    let expr = parse_expr(it);

    let next = it.next().unwrap();

    match next {
      &Token::Semi => {
        let statement = Statement::Assignment(identifier.to_string(), expr);

        statements.push(statement);
      },
      _ => panic!("expected a semicolon")
    }

    match it.peek() {
      None => break,
      _ => {}
    }

  }
  
  statements
}

fn main() {
  let args: Vec<String> = std::env::args().collect();

  match tokenize(&args[1]) {
    Err(_) => println!("token error"),

    Ok(tokens) => {
      let statements = parse(&mut tokens.iter().peekable());

      generate(&statements);
    }
  }
}


