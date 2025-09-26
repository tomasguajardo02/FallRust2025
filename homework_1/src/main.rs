const FREEZING_POINT_F: f64 = 32.0;

fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - FREEZING_POINT_F) * 5.0 / 9.0
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    (c * 9.0 / 5.0) + FREEZING_POINT_F
}

fn is_even(n: i32) -> bool {
    n % 2 == 0
}

fn check_guess(guess: i32, secret: i32) -> i32 {
    if guess == secret {
        0
    } else if guess > secret {
        1
    } else {
        -1
    }
}

fn main() {
    // ---------------- Assignment 1 ----------------
    println!("--- Assignment 1: Temperature Converter ---");
    let mut temp_f: f64 = 32.0;

    let temp_c = fahrenheit_to_celsius(temp_f);
    println!("{:.1}°F = {:.1}°C", temp_f, temp_c);

    for _ in 0..5 {
        temp_f += 1.0;
        let temp_c = fahrenheit_to_celsius(temp_f);
        println!("{:.1}°F = {:.1}°C", temp_f, temp_c);
    }

    // Extra line so celsius_to_fahrenheit is used → no warning
    let c: f64 = 0.0;
    let f = celsius_to_fahrenheit(c);
    println!("{:.1}°C = {:.1}°F", c, f);

    // ---------------- Assignment 2 ----------------
    println!("\n--- Assignment 2: Number Analyzer ---");
    let numbers: [i32; 10] = [3, 5, 8, 12, 15, 20, 25, 30, 33, 40];

    for n in numbers {
        if n % 3 == 0 && n % 5 == 0 {
            println!("{} -> FizzBuzz", n);
        } else if n % 3 == 0 {
            println!("{} -> Fizz", n);
        } else if n % 5 == 0 {
            println!("{} -> Buzz", n);
        } else if is_even(n) {
            println!("{} -> Even", n);
        } else {
            println!("{} -> Odd", n);
        }
    }

    let mut sum = 0;
    let mut i = 0;
    while i < numbers.len() {
        sum += numbers[i];
        i += 1;
    }
    println!("Sum of all numbers = {}", sum);

    let mut largest = numbers[0];
    let mut j = 0;
    loop {
        if j >= numbers.len() {
            break;
        }
        if numbers[j] > largest {
            largest = numbers[j];
        }
        j += 1;
    }
    println!("Largest number = {}", largest);

    // ---------------- Assignment 3 ----------------
    println!("\n--- Assignment 3: Guessing Game ---");
    let secret: i32 = 7;
    let mut guess: i32;
    let mut attempts = 0;

    loop {
        guess = attempts + 5; // guesses: 5, 6, 7...
        attempts += 1;

        let result = check_guess(guess, secret);

        if result == 0 {
            println!("Guess {} is correct!", guess);
            break;
        } else if result == 1 {
            println!("Guess {} is too high.", guess);
        } else {
            println!("Guess {} is too low.", guess);
        }
    }
    println!("It took {} guesses.", attempts);
}
