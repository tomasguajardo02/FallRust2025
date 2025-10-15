enum Fruit {
    Apple(String),
    Banana(String),
    Tomato(String),
}

struct Inventory {
    fruit: Vec<Fruit>,
}

impl Inventory {
    fn available_fruits(&self) {
        for f in &self.fruit {
            Self::tell_me_joke(f);
        }
    }

    fn tell_me_joke(fruit: &Fruit) {
        match fruit {
            Fruit::Apple(j) => println!("Apple joke: {}", j),
            Fruit::Banana(j) => println!("Banana joke: {}", j),
            Fruit::Tomato(j) => println!("Tomato joke: {}", j),
        }
    }
}

fn main() {
    let a = "Apples never fall far from the tree.".to_string();
    let b = "Bananas go bad because they can’t handle the peelings.".to_string();
    let t = "Tomatoes can’t hide their emotions—they always ketchup.".to_string();

    let fruits = vec![
        Fruit::Banana(b),
        Fruit::Apple(a),
        Fruit::Tomato(t),
    ];

    let grocery_store = Inventory { fruit: fruits };
    grocery_store.available_fruits();
}
