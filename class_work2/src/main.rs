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

fn main() {
    // ===== Assignment 1: Student Struct =====
    let mut student1 = Student::new("Tomas Guajardo", "Computer Engineering");
    println!("Student: {}, Major: {}", student1.name, student1.get_major());

    student1.set_major("Computer Engineering");
    println!("Updated Major: {}", student1.get_major());
}
