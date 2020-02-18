fn main() {
    for i in 0..2 {
        {
            if i == 1 {
                break;
            }
            print!("{}", i);
        }
    }
}
// Introduce closure at line 3 to 8
// The conditional break at line 5 should be preserved.