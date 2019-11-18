fn main() {
    /* 1 */let aaa =
    {
        /* 1a */let bbb = 1;
        /* 1b */bbb/* 1c */
    };
    /* 2 */println!("{}", aaa);/* 3 */
}
/*
 * Valid selections:
 * - 1:2, 1:3, 2:3
 * - 1a:1b, 1a:1c, 1b:1c
 * 
 * bytepos:
 * 1: 16, 2: 107, 3: 134
 * 1a: 47, 1b: 76, 1c: 87
 */