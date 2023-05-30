use too_many_lists::first::List;

fn main() {
    let mut list: List = List::new();
    list.push(1);
    list.push(2);
    println!("{:?}", list);

    let last = list.pop();
    println!("{:?} popped from {:?}", last, list);
}
