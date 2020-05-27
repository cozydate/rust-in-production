// use std::future::Future;

#[derive(Debug)]
pub struct Value {
    val: bool,
}

impl Value {
    pub fn new() -> Self {
        Self { val: false }
    }

    pub fn change(self: &mut Self) {
        self.val ^= true;
    }

    pub fn get(self: &Self) -> bool {
        self.val
    }
}

#[derive(Debug)]
pub struct Struct {
    a: Value,
    b: Value,
}

impl Struct {
    pub fn new() -> Self {
        Struct { a: Value::new(), b: Value::new() }
    }

    pub fn simple(self: &mut Struct) {
        println!("change()");
        self.a.change();
        self.b.change();
    }

    pub fn mutable_refs(self: &mut Struct) {
        println!("mutable_refs()");
        let a = &mut self.a;
        let b = &mut self.b;
        a.change();
        b.change();
    }

    pub fn both_in_closure(self: &mut Struct) {
        println!("both_in_closure()");
        let a = &mut self.a;
        let b = &mut self.b;
        let mut closure = move || {
            a.change();
            b.change();
        };
        closure();
    }

    pub fn one_in_closure(self: &mut Struct) {
        println!("one_in_closure()");
        // // error[E0500]: closure requires unique access to `self` but it is already borrowed
        // let a = &mut self.a;
        // //      ----------- borrow occurs here
        // let mut closure = || {
        //     //            ^^ closure construction occurs here
        //     a.change(); // first borrow later captured here by closure
        //     self.b.change(); // second borrow occurs due to use of `self` in closure
        // };
        // closure();
    }

    pub fn stream(self: &mut Struct) {
        println!("stream()");
        let v = Vec::<bool>::new();

        // let a = &mut self.a;
        // //                 ----------- borrow occurs here
        // let mut closure = || {
        //     //                  ^^ closure construction occurs here
        //     a.change(); // first borrow later captured here by closure
        //     self.b.change(); // second borrow occurs due to use of `self` in closure
        // };
        // closure();
    }

    // pub fn increment_both_in_closure(self: &mut Struct) {
    //     println!("add1()");
    //     let mut closure = || {
    //         self.x.increment();
    //         self.y.increment();
    //     };
    //     closure();
    // }
    //
    // pub fn increment_one_in_closure(self: &mut Struct) {
    //     println!("add1()");
    //     let mut closure = || {
    //         self.x.increment();
    //         self.y.increment();
    //     };
    //     closure();
    // }
}

pub fn main() {
    let mut s = Struct::new();
    println!("{:?}", s);
    s.simple();
    println!("{:?}", s);
    s.mutable_refs();
    println!("{:?}", s);
    s.both_in_closure();
    println!("{:?}", s);
    s.one_in_closure();
    println!("{:?}", s);
    s.stream();
    println!("{:?}", s);
}