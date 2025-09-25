// Problem 1
fn concat_strings(s1: &String, s2: &String) -> String {
    let mut result = String::new();
    result.push_str(s1);
    result.push_str(s2);
    result
}

// Problem 2
fn clone_and_modify(s: &String) -> String {
    let mut cloned = s.clone(); 
    cloned.push_str("World!");  
    cloned
}

// Problem 3
fn sum(total: &mut i32, low: i32, high: i32) {
    *total = 0; 
    for i in low..=high {
        *total += i;
    }
}

fn main() {
    // --- Problem 1 ---
    let s1 = String::from("Hello, ");
    let s2 = String::from("World!");
    let result = concat_strings(&s1, &s2);
    println!("Problem 1 Result: {}", result); // "Hello, World!"

    // --- Problem 2 ---
    let s = String::from("Hello, ");
    let modified = clone_and_modify(&s);
    println!("Problem 2 Original: {}", s);       // "Hello, "
    println!("Problem 2 Modified: {}", modified); // "Hello, World!"

    // --- Problem 3 ---
    let mut total = 0;
    sum(&mut total, 0, 100);
    println!("Problem 3 Sum from 0 to 100: {}", total); // 5050
}
