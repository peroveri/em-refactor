fn main() {
    let (i, j) = (0, 1);
    let (i, j) = 
{let (i, j) = (2, 3);(i, j)};
    print!{"{}{}", i, j};
}