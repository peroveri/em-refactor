fn main() {
    for i in 10..12 {
        {
            if i == 11 {
                break;
            }
            print!("{}", i);
        }
    }
}
// Introduce closure at line 3 to 8
// The conditional break at line 5 should be preserved.