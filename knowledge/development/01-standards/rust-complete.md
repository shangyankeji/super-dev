---
id: rust-complete
title: Rust语言完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [complete, development, rust, 参考资料, 学习路径, 最佳实践, 核心概念, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Rust语言完整指南

## 概述
Rust是一门系统编程语言,专注于安全性、并发性和性能。通过所有权系统保证内存安全,无需垃圾回收器。广泛用于系统编程、WebAssembly、嵌入式开发。

## 核心概念

### 1. 所有权系统

**所有权规则**:
```rust
fn main() {
    // 1. 每个值有一个所有者
    let s1 = String::from("hello");
    
    // 2. 同一时刻只能有一个所有者
    let s2 = s1;  // s1的所有权转移给s2
    // println!("{}", s1);  // 错误: s1已失效
    
    // 3. 所有者离开作用域时,值被释放
    {
        let s3 = String::from("world");
    }  // s3在这里被释放
}
```

**借用**:
```rust
fn main() {
    let s = String::from("hello");
    
    // 不可变借用(&T)
    let len = calculate_length(&s);
    println!("Length of '{}' is {}", s, len);
    
    // 可变借用(&mut T)
    let mut s2 = String::from("hello");
    change(&mut s2);
    println!("{}", s2);  // "hello, world"
    
    // 借用规则:
    // 1. 可以有多个不可变借用
    let r1 = &s;
    let r2 = &s;
    
    // 2. 或一个可变借用
    // let r3 = &mut s;  // 错误: 已有不可变借用
}

fn calculate_length(s: &String) -> usize {
    s.len()
}

fn change(s: &mut String) {
    s.push_str(", world");
}
```

### 2. 数据类型

**标量类型**:
```rust
fn main() {
    // 整数
    let a: i32 = 42;
    let b: u8 = 255;
    
    // 浮点数
    let x: f64 = 3.14;
    let y: f32 = 2.71;
    
    // 布尔
    let t: bool = true;
    
    // 字符
    let c: char = 'z';
    
    // 元组
    let tup: (i32, f64, u8) = (500, 6.4, 1);
    let (x, y, z) = tup;
    let first = tup.0;
    
    // 数组(固定长度)
    let arr = [1, 2, 3, 4, 5];
    let first = arr[0];
}
```

**结构体**:
```rust
// 基本结构体
struct User {
    username: String,
    email: String,
    age: u32,
    active: bool,
}

// 创建实例
let user1 = User {
    email: String::from("alice@example.com"),
    username: String::from("alice"),
    age: 30,
    active: true,
};

// 结构体更新语法
let user2 = User {
    email: String::from("bob@example.com"),
    ..user1  // 其余字段来自user1
};

// 元组结构体
struct Point(i32, i32, i32);
let origin = Point(0, 0, 0);

// 单元结构体
struct AlwaysEqual;

// 方法
impl User {
    // 关联函数(构造器)
    fn new(email: String, username: String) -> Self {
        Self {
            email,
            username,
            age: 0,
            active: true,
        }
    }
    
    // 方法
    fn is_adult(&self) -> bool {
        self.age >= 18
    }
    
    fn set_age(&mut self, age: u32) {
        self.age = age;
    }
}

// 使用
let mut user = User::new(
    String::from("alice@example.com"),
    String::from("alice")
);
user.set_age(30);
println!("Is adult: {}", user.is_adult());
```

**枚举**:
```rust
// 基本枚举
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// 带数据的枚举
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

// 使用
let msg = Message::Move { x: 10, y: 20 };

match msg {
    Message::Quit => println!("Quit"),
    Message::Move { x, y } => println!("Move to ({}, {})", x, y),
    Message::Write(text) => println!("Text: {}", text),
    Message::ChangeColor(r, g, b) => println!("Color: ({}, {}, {})", r, g, b),
}

// Option枚举(空值处理)
fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

let result = divide(10.0, 2.0);
match result {
    Some(x) => println!("Result: {}", x),
    None => println!("Cannot divide by zero"),
}

// if let语法
if let Some(x) = result {
    println!("Result: {}", x);
}
```

### 3. 模式匹配

**match表达式**:
```rust
fn main() {
    let number = 13;
    
    match number {
        1 => println!("One"),
        2 | 3 | 5 | 7 | 11 | 13 => println!("Prime"),
        13..=19 => println!("Teen"),
        _ => println!("Other"),
    }
    
    // 解构
    let point = (3, 5);
    match point {
        (0, 0) => println!("Origin"),
        (0, y) => println!("On Y axis at {}", y),
        (x, 0) => println!("On X axis at {}", x),
        (x, y) => println!("At ({}, {})", x, y),
    }
}
```

### 4. 错误处理

**Result<T, E>**:
```rust
use std::fs::File;
use std::io::ErrorKind;

fn main() {
    // 打开文件
    let file_result = File::open("hello.txt");
    
    let file = match file_result {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create("hello.txt") {
                Ok(fc) => fc,
                Err(e) => panic!("Problem creating file: {:?}", e),
            },
            other_error => panic!("Problem opening file: {:?}", other_error),
        },
    };
    
    // unwrap和expect
    let file = File::open("hello.txt").unwrap();
    let file = File::open("hello.txt").expect("Failed to open hello.txt");
    
    // ?运算符
    let file = File::open("hello.txt")?;
}

// 自定义错误类型
#[derive(Debug)]
enum MyError {
    Io(std::io::Error),
    Parse(std::num::ParseIntError),
}

fn read_number_from_file(filename: &str) -> Result<i32, MyError> {
    let content = std::fs::read_to_string(filename).map_err(MyError::Io)?;
    let number = content.trim().parse::<i32>().map_err(MyError::Parse)?;
    Ok(number)
}
```

### 5. 泛型和Trait

**泛型**:
```rust
// 泛型函数
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    
    largest
}

// 泛型结构体
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

// 多个泛型参数
struct Point2<T, U> {
    x: T,
    y: U,
}
```

**Trait**:
```rust
// 定义Trait
pub trait Summary {
    fn summarize(&self) -> String;
    
    // 默认实现
    fn author(&self) -> String {
        String::from("Unknown")
    }
}

// 实现Trait
pub struct Article {
    pub title: String,
    pub content: String,
    pub author: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
}

// Trait作为参数
fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}

// Trait bound
fn notify2<T: Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}

// 多个Trait
fn notify3(item: &(impl Summary + Display)) {}

// where子句
fn some_function<T, U>(t: &T, u: &U) -> i32
where
    T: Display + Clone,
    U: Clone + Debug,
{
    0
}
```

### 6. 并发

**线程**:
```rust
use std::thread;
use std::time::Duration;

fn main() {
    // 创建线程
    let handle = thread::spawn(|| {
        for i in 1..10 {
            println!("Spawned thread: {}", i);
            thread::sleep(Duration::from_millis(1));
        }
    });
    
    for i in 1..5 {
        println!("Main thread: {}", i);
        thread::sleep(Duration::from_millis(1));
    }
    
    // 等待线程完成
    handle.join().unwrap();
    
    // 使用move闭包
    let v = vec![1, 2, 3];
    let handle = thread::spawn(move || {
        println!("Vector: {:?}", v);
    });
    handle.join().unwrap();
}
```

**消息传递**:
```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        let val = String::from("hi");
        tx.send(val).unwrap();
    });
    
    let received = rx.recv().unwrap();
    println!("Got: {}", received);
    
    // 多生产者
    let (tx1, rx1) = mpsc::channel();
    let tx2 = tx1.clone();
    
    thread::spawn(move || {
        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];
        
        for val in vals {
            tx1.send(val).unwrap();
        }
    });
    
    thread::spawn(move || {
        let vals = vec![
            String::from("more"),
            String::from("messages"),
        ];
        
        for val in vals {
            tx2.send(val).unwrap();
        }
    });
    
    for received in rx1 {
        println!("Got: {}", received);
    }
}
```

**共享状态**:
```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Result: {}", *counter.lock().unwrap());
}
```

## 最佳实践

### ✅ DO

1. **使用clippy检查代码**
```bash
cargo clippy
```

2. **使用Result而不是panic**
```rust
// ✅ 好
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("division by zero"))
    } else {
        Ok(a / b)
    }
}

// ❌ 差
fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        panic!("division by zero");
    }
    a / b
}
```

3. **使用文档注释**
```rust
/// Adds two numbers.
/// 
/// # Examples
/// ```
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### ❌ DON'T

1. **不要过度使用unwrap**
```rust
// ❌ 差
let file = File::open("file.txt").unwrap();

// ✅ 好
let file = File::open("file.txt")
    .expect("Failed to open file.txt");
```

2. **不要忽略错误**
```rust
// ❌ 差
let _ = dangerous_operation();

// ✅ 好
if let Err(e) = dangerous_operation() {
    eprintln!("Error: {}", e);
}
```

## 学习路径

### 初级 (2-3周)
1. 所有权和借用
2. 基本数据类型
3. 函数和控制流

### 中级 (2-3周)
1. 结构体和枚举
2. 模式匹配
3. 错误处理

### 高级 (2-4周)
1. 泛型和Trait
2. 并发编程
3. 智能指针

### 专家级 (持续)
1. 异步编程(async/await)
2. unsafe Rust
3. 嵌入式开发

## 参考资料

### 官方
- [Rust官方书籍](https://doc.rust-lang.org/book/)
- [Rust标准库文档](https://doc.rust-lang.org/std/)

### 社区
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings练习](https://github.com/rust-lang/rustlings)

---

**知识ID**: `rust-complete`  
**领域**: development  
**类型**: standards  
**难度**: advanced  
**质量分**: 94  
**维护者**: dev-team@superdev.com  
**最后更新**: 2026-03-28
