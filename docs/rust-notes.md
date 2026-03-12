# Rust 핵심 개념 정리 (Han 컴파일러 개발용)

> Han 컴파일러를 만들기 위해 필요한 Rust 핵심 개념들을 정리합니다.

---

## 1. Ownership & Borrowing

컴파일러 코드에서 가장 중요한 개념. AST 노드를 이동/참조할 때 핵심.

```rust
// 소유권 이동 (move)
let s1 = String::from("hello");
let s2 = s1; // s1은 더 이상 유효하지 않음

// 불변 참조 (&T)
let s = String::from("hello");
let len = calculate_length(&s); // s를 빌려줌, s는 여전히 유효

// 가변 참조 (&mut T)
let mut s = String::from("hello");
change(&mut s);

fn change(s: &mut String) {
    s.push_str(", world");
}
```

**컴파일러에서 활용**: 토큰 벡터를 파서에 넘길 때 `&[Token]` 슬라이스 사용.

---

## 2. enum + match (AST 핵심)

Han 컴파일러의 AST 노드를 표현하는 핵심 패턴.

```rust
// AST 노드 예시
enum Expr {
    Integer(i64),
    Float(f64),
    Identifier(String),
    BinaryOp {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

// match로 처리
fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::Integer(n) => *n,
        Expr::BinaryOp { op, left, right } => {
            let l = eval(left);
            let r = eval(right);
            match op.as_str() {
                "+" => l + r,
                "-" => l - r,
                _ => panic!("Unknown op"),
            }
        }
        _ => todo!(),
    }
}
```

---

## 3. impl + trait

타입별 동작을 구현할 때 사용.

```rust
trait Evaluate {
    fn eval(&self) -> i64;
}

struct IntLiteral(i64);

impl Evaluate for IntLiteral {
    fn eval(&self) -> i64 {
        self.0
    }
}

// Display trait 구현 (디버깅용)
use std::fmt;
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
```

---

## 4. Box<T> — 재귀 타입 필수

AST 노드가 자기 자신을 포함할 때 (재귀 구조) 반드시 필요.

```rust
// ❌ 컴파일 에러: 크기를 알 수 없음
enum Expr {
    BinaryOp { left: Expr, right: Expr } // 무한 크기!
}

// ✅ Box로 힙에 할당
enum Expr {
    BinaryOp {
        left: Box<Expr>,  // 포인터 크기 = 8 bytes
        right: Box<Expr>,
    }
}

// Box 생성
let node = Expr::BinaryOp {
    left: Box::new(Expr::Integer(3)),
    right: Box::new(Expr::Integer(5)),
};
```

📌 **Concept: Box<T>** — 힙 할당 스마트 포인터. 재귀 타입의 크기 문제 해결.

---

## 5. Result<T, E> — 에러 처리

렉서/파서에서 에러를 반환할 때 사용.

```rust
#[derive(Debug)]
enum ParseError {
    UnexpectedToken { expected: String, found: String, line: usize },
    UnexpectedEof,
}

fn parse_function(&mut self) -> Result<Stmt, ParseError> {
    self.expect(Token::함수)?; // ?로 에러 전파
    let name = self.expect_identifier()?;
    // ...
    Ok(Stmt::FunctionDef { name, ... })
}

// ? 연산자 = 에러면 즉시 반환
// Ok(value) = 성공
// Err(e) = 실패
```

---

## 6. iter / map / filter — 토큰 처리

토큰 스트림을 처리할 때 활용.

```rust
let tokens: Vec<Token> = source
    .chars()
    .collect::<Vec<char>>();

// 키워드 매핑
let keywords: HashMap<&str, Token> = [
    ("함수", Token::함수),
    ("반환", Token::반환),
    ("변수", Token::변수),
].iter().cloned().collect();

// 필터링
let identifiers: Vec<&Token> = tokens
    .iter()
    .filter(|t| matches!(t, Token::Identifier(_)))
    .collect();
```

---

## 7. String vs &str

```rust
// String: 소유권 있는 힙 문자열 (가변)
let mut s = String::from("안녕");
s.push_str("하세요");

// &str: 문자열 슬라이스 (불변 참조)
let greeting: &str = "안녕하세요";

// 함수 파라미터는 &str 선호 (더 유연)
fn greet(name: &str) { println!("Hello, {}", name); }
greet("세계");           // &str 직접
greet(&String::from("세계")); // String도 가능
```

---

## 8. Vec<T> — 동적 배열

토큰 목록, AST 노드 목록에 사용.

```rust
let mut tokens: Vec<Token> = Vec::new();
tokens.push(Token::함수);
tokens.push(Token::Identifier("더하기".to_string()));

// 인덱싱
let first = &tokens[0];
let maybe = tokens.get(0); // Option<&Token>

// 슬라이스
let slice: &[Token] = &tokens[1..3];
```

---

## 9. HashMap — 키워드 매핑

렉서에서 키워드 → Token 변환에 필수.

```rust
use std::collections::HashMap;

let mut keywords: HashMap<String, Token> = HashMap::new();
keywords.insert("함수".to_string(), Token::함수);
keywords.insert("반환".to_string(), Token::반환);

// 조회
if let Some(token) = keywords.get("함수") {
    println!("키워드 발견: {:?}", token);
}
```

---

## 10. 한글 유니코드 처리

**핵심**: Rust의 `str`은 UTF-8이지만 바이트 단위 인덱싱은 한글을 깨뜨림.
반드시 `chars()` 사용!

```rust
let source = "함수 더하기() { }";

// ❌ 바이트 단위 (한글 깨짐)
let byte = source.as_bytes()[0]; // 0xED (깨진 값)

// ✅ char 단위 (유니코드 올바름)
let chars: Vec<char> = source.chars().collect();
let first_char = chars[0]; // '함'

// 한글 식별자 판별
fn is_korean(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c) // 완성형 한글
    || ('\u{1100}'..='\u{11FF}').contains(&c) // 자모
    || ('\u{3130}'..='\u{318F}').contains(&c) // 호환 자모
}

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || is_korean(c)
}
```

---

## 참고 자료

- [The Rust Book](https://doc.rust-lang.org/book/) — Ch. 1-15 필수
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Crafting Interpreters](https://craftinginterpreters.com/) — 컴파일러 구조 참고
