struct Student {
    name: String,
    major: String,
}

impl Student {
    fn new(name: &str, major: &str) -> Self {
        Student {
            name: name.to_string(),
            major: major.to_string(),
        }
    }

    fn set_major(&mut self, new_major: &str) {
        self.major = new_major.to_string();
    }

    fn get_major(&self) -> &str {
        &self.major
    }
}

struct ParkingSystem {
    big: i32,
    medium: i32,
    small: i32,
}

impl ParkingSystem {
    fn new(big: i32, medium: i32, small: i32) -> Self {
        ParkingSystem { big, medium, small }
    }

    fn add_car(&mut self, car_type: i32) -> bool {
        match car_type {
            1 if self.big > 0 => {
                self.big -= 1;
                true
            }
            2 if self.medium > 0 => {
                self.medium -= 1;
                true
            }
            3 if self.small > 0 => {
                self.small -= 1;
                true
            }
            _ => false,
        }
    }
}

fn main() {
    // ===== Assignment 1: Student Struct =====
    let mut student1 = Student::new("Tomas Guajardo", "Computer Engineering");
    println!("Student: {}, Major: {}", student1.name, student1.get_major());

    student1.set_major("Computer engineering");
    println!("Updated Major: {}", student1.get_major());

    // ===== Assignment 2: Parking System =====
    let mut ps = ParkingSystem::new(1, 1, 0); // 1 big, 1 medium, 0 small

    println!("Add big car: {}", ps.add_car(1));   // true
    println!("Add medium car: {}", ps.add_car(2)); // true
    println!("Add small car: {}", ps.add_car(3));  // false
    println!("Add big car again: {}", ps.add_car(1)); // false
}
