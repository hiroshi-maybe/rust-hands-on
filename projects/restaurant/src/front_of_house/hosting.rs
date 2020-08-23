pub fn add_to_waitlist() {
    println!("Added to a wait list");
}
fn seat_at_table() {
    // Non-public function in a parent module is visible
    super::f()
}
