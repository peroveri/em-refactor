fn main() {
    let i = 0;
    let j = &i;
    &j;
}

#[cfg(never)]
fn shouldnt_be_found() {
    let x = 0;
}