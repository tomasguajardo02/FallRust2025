// Define a trait for showing student information
trait ShowInfo {
    fn show_info(&self);
}

// Shared struct fields for all students
#[derive(Debug)]
struct StudentBase {
    name: String,
    major: String,
    gpa: f32,
}

// Undergraduate student struct
#[derive(Debug)]
struct Undergrad {
    base: StudentBase,
    year: u32,
}

// Graduate student struct (has a thesis)
#[derive(Debug)]
struct Grad {
    base: StudentBase,
    thesis: String,
}

// Implement ShowInfo for Undergrad
impl ShowInfo for Undergrad {
    fn show_info(&self) {
        println!(
            "Undergrad: {}, Major: {}, GPA: {}, Year: {}",
            self.base.name, self.base.major, self.base.gpa, self.year
        );
    }
}

// Implement ShowInfo for Grad
impl ShowInfo for Grad {
    fn show_info(&self) {
        println!(
            "Grad: {}, Major: {}, GPA: {}, Thesis: {}",
            self.base.name, self.base.major, self.base.gpa, self.thesis
        );
    }
}

// Enrollment struct using generics and trait bounds
struct Enrollment<T: ShowInfo> {
    students: Vec<T>,
}

impl<T: ShowInfo> Enrollment<T> {
    fn new() -> Self {
        Enrollment { students: Vec::new() }
    }

    fn add_student(&mut self, student: T) {
        self.students.push(student);
    }

    fn show_all(&self) {
        for s in &self.students {
            s.show_info();
        }
    }
}

fn main() {
    // Create some undergrads
    let u1 = Undergrad {
        base: StudentBase {
            name: String::from("Alice"),
            major: String::from("Computer Science"),
            gpa: 3.6,
        },
        year: 2,
    };

    let u2 = Undergrad {
        base: StudentBase {
            name: String::from("Bob"),
            major: String::from("Math"),
            gpa: 3.9,
        },
        year: 3,
    };

    // Create some grads
    let g1 = Grad {
        base: StudentBase {
            name: String::from("Carol"),
            major: String::from("Data Science"),
            gpa: 3.8,
        },
        thesis: String::from("Machine Learning in Healthcare"),
    };

    // Create two enrollments
    let mut undergrad_enrollment = Enrollment::new();
    undergrad_enrollment.add_student(u1);
    undergrad_enrollment.add_student(u2);

    let mut grad_enrollment = Enrollment::new();
    grad_enrollment.add_student(g1);

    println!("--- Undergrad Enrollment ---");
    undergrad_enrollment.show_all();

    println!("\n--- Grad Enrollment ---");
    grad_enrollment.show_all();
}
