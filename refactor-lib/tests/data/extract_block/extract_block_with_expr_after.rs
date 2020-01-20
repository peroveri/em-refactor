fn main() {
    let j = {
        let i = 0;
        {let i = 1;
        i + 1}
    };
    print!{"{}", j};
}