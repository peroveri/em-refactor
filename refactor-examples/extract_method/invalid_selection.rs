fn foo() {
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
 * 1: 15, 2: 106, 3: 133
 * 1a: 46, 1b: 75, 1c: 86
 */