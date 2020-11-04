// Ch 17-2 Using Trait Objects That Allow for Values of Different Types

/**
 *  A trait is object safe if all the methods defined in the trait have the following properties:
 *  - The return type isn’t Self.
 *  - There are no generic type parameters.
 */

pub trait Draw {
    fn draw(&self);
}

pub struct Screen {
    pub components: Vec<Box<dyn Draw>>,
}

impl Screen {
    pub fn run(&self) {
        for component in self.components.iter() {
            component.draw();
        }
    }
}

#[derive(Debug)]
pub struct Button {
    pub width: u32,
    pub height: u32,
    pub label: String,
}

impl Draw for Button {
    fn draw(&self) {
        println!("Button {:?} was drawn", self);
    }
}

#[derive(Debug)]
pub struct SelectBox {
    pub width: u32,
    pub height: u32,
    pub options: Vec<String>,
}

impl Draw for SelectBox {
    fn draw(&self) {
        println!("SelectBox {:?} was drawn", self);
    }
}

// Ch 17-1 Characteristics of Object-Oriented Languages

pub struct AveragedCollection {
    list: Vec<i32>,
    average: f64,
}

impl AveragedCollection {
    pub fn add(&mut self, val: i32) {
        self.list.push(val);
        self.update_average();
    }

    pub fn remove(&mut self) -> Option<i32> {
        let res = self.list.pop();
        match res {
            Some(val) => {
                self.update_average();
                Some(val)
            }
            None => None,
        }
    }

    pub fn average(&self) -> f64 {
        self.average
    }

    fn update_average(&mut self) {
        let tot: i32 = self.list.iter().sum();
        self.average = tot as f64 / self.list.len() as f64;
    }
}