use super::*;

#[test]
fn size_of() {
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn size_of<T>() -> usize;
        }

        const GOAL: usize = size_of::<i32>();
        "#,
        4,
    );
}

#[test]
fn transmute() {
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn transmute<T, U>(e: T) -> U;
        }

        const GOAL: i32 = transmute((1i16, 1i16));
        "#,
        0x00010001,
    );
}

#[test]
fn const_eval_select() {
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn const_eval_select<ARG, F, G, RET>(arg: ARG, called_in_const: F, called_at_rt: G) -> RET
            where
                G: FnOnce<ARG, Output = RET>,
                F: FnOnce<ARG, Output = RET>;
        }

        const fn in_const(x: i32, y: i32) -> i32 {
            x + y
        }

        fn in_rt(x: i32, y: i32) -> i32 {
            x + y
        }

        const GOAL: i32 = const_eval_select((2, 3), in_const, in_rt);
        "#,
        5,
    );
}

#[test]
fn wrapping_add() {
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn wrapping_add<T>(a: T, b: T) -> T;
        }

        const GOAL: u8 = wrapping_add(10, 250);
        "#,
        4,
    );
}

#[test]
fn saturating_add() {
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn saturating_add<T>(a: T, b: T) -> T;
        }

        const GOAL: u8 = saturating_add(10, 250);
        "#,
        255,
    );
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn saturating_add<T>(a: T, b: T) -> T;
        }

        const GOAL: i8 = saturating_add(5, 8);
        "#,
        13,
    );
}

#[test]
fn allocator() {
    check_number(
        r#"
        extern "Rust" {
            #[rustc_allocator]
            fn __rust_alloc(size: usize, align: usize) -> *mut u8;
            #[rustc_deallocator]
            fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize);
            #[rustc_reallocator]
            fn __rust_realloc(ptr: *mut u8, old_size: usize, align: usize, new_size: usize) -> *mut u8;
            #[rustc_allocator_zeroed]
            fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8;
        }

        const GOAL: u8 = unsafe {
            let ptr = __rust_alloc(4, 1);
            let ptr2 = ((ptr as usize) + 1) as *mut u8;
            *ptr = 23;
            *ptr2 = 32;
            let ptr = __rust_realloc(ptr, 4, 1, 8);
            let ptr2 = ((ptr as usize) + 1) as *mut u8;
            *ptr + *ptr2
        };
        "#,
        55,
    );
}

#[test]
fn overflowing_add() {
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn add_with_overflow<T>(x: T, y: T) -> (T, bool);
        }

        const GOAL: u8 = add_with_overflow(1, 2).0;
        "#,
        3,
    );
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn add_with_overflow<T>(x: T, y: T) -> (T, bool);
        }

        const GOAL: u8 = add_with_overflow(1, 2).1 as u8;
        "#,
        0,
    );
}

#[test]
fn needs_drop() {
    check_number(
        r#"
        //- minicore: copy, sized
        extern "rust-intrinsic" {
            pub fn needs_drop<T: ?Sized>() -> bool;
        }
        struct X;
        const GOAL: bool = !needs_drop::<i32>() && needs_drop::<X>();
        "#,
        1,
    );
}

#[test]
fn likely() {
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn likely(b: bool) -> bool;
            pub fn unlikely(b: bool) -> bool;
        }

        const GOAL: bool = likely(true) && unlikely(true) && !likely(false) && !unlikely(false);
        "#,
        1,
    );
}

#[test]
fn atomic() {
    check_number(
        r#"
        //- minicore: copy
        extern "rust-intrinsic" {
            pub fn atomic_load_seqcst<T: Copy>(src: *const T) -> T;
            pub fn atomic_xchg_acquire<T: Copy>(dst: *mut T, src: T) -> T;
            pub fn atomic_cxchg_release_seqcst<T: Copy>(dst: *mut T, old: T, src: T) -> (T, bool);
            pub fn atomic_cxchgweak_acquire_acquire<T: Copy>(dst: *mut T, old: T, src: T) -> (T, bool);
            pub fn atomic_store_release<T: Copy>(dst: *mut T, val: T);
            pub fn atomic_xadd_acqrel<T: Copy>(dst: *mut T, src: T) -> T;
            pub fn atomic_xsub_seqcst<T: Copy>(dst: *mut T, src: T) -> T;
            pub fn atomic_and_acquire<T: Copy>(dst: *mut T, src: T) -> T;
            pub fn atomic_nand_seqcst<T: Copy>(dst: *mut T, src: T) -> T;
            pub fn atomic_or_release<T: Copy>(dst: *mut T, src: T) -> T;
            pub fn atomic_xor_seqcst<T: Copy>(dst: *mut T, src: T) -> T;
        }

        fn should_not_reach() {
            _ // fails the test if executed
        }

        const GOAL: i32 = {
            let mut x = 5;
            atomic_store_release(&mut x, 10);
            let mut y = atomic_xchg_acquire(&mut x, 100);
            atomic_xadd_acqrel(&mut y, 20);
            if (30, true) != atomic_cxchg_release_seqcst(&mut y, 30, 40) {
                should_not_reach();
            }
            if (40, false) != atomic_cxchg_release_seqcst(&mut y, 30, 50) {
                should_not_reach();
            }
            if (40, true) != atomic_cxchgweak_acquire_acquire(&mut y, 40, 30) {
                should_not_reach();
            }
            let mut z = atomic_xsub_seqcst(&mut x, -200);
            atomic_xor_seqcst(&mut x, 1024);
            atomic_load_seqcst(&x) + z * 3 + atomic_load_seqcst(&y) * 2
        };
        "#,
        660 + 1024,
    );
}

#[test]
fn offset() {
    check_number(
        r#"
        //- minicore: coerce_unsized, index, slice
        extern "rust-intrinsic" {
            pub fn offset<T>(dst: *const T, offset: isize) -> *const T;
        }

        const GOAL: u8 = unsafe {
            let ar: &[(u8, u8, u8)] = &[
                (10, 11, 12),
                (20, 21, 22),
                (30, 31, 32),
                (40, 41, 42),
                (50, 51, 52),
            ];
            let ar: *const [(u8, u8, u8)] = ar;
            let ar = ar as *const (u8, u8, u8);
            let element = *offset(ar, 2);
            element.1
        };
        "#,
        31,
    );
}

#[test]
fn arith_offset() {
    check_number(
        r#"
        //- minicore: coerce_unsized, index, slice
        extern "rust-intrinsic" {
            pub fn arith_offset<T>(dst: *const T, offset: isize) -> *const T;
        }

        const GOAL: u8 = unsafe {
            let ar: &[(u8, u8, u8)] = &[
                (10, 11, 12),
                (20, 21, 22),
                (30, 31, 32),
                (40, 41, 42),
                (50, 51, 52),
            ];
            let ar: *const [(u8, u8, u8)] = ar;
            let ar = ar as *const (u8, u8, u8);
            let element = *arith_offset(arith_offset(ar, 102), -100);
            element.1
        };
        "#,
        31,
    );
}

#[test]
fn copy_nonoverlapping() {
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize);
        }

        const GOAL: u8 = unsafe {
            let mut x = 2;
            let y = 5;
            copy_nonoverlapping(&y, &mut x, 1);
            x
        };
        "#,
        5,
    );
}

#[test]
fn copy() {
    check_number(
        r#"
        //- minicore: coerce_unsized, index, slice
        extern "rust-intrinsic" {
            pub fn copy<T>(src: *const T, dst: *mut T, count: usize);
        }

        const GOAL: i32 = unsafe {
            let mut x = [1i32, 2, 3, 4, 5];
            let y = (&mut x as *mut _) as *mut i32;
            let z = (y as usize + 4) as *const i32;
            copy(z, y, 4);
            x[0] + x[1] + x[2] + x[3] + x[4]
        };
        "#,
        19,
    );
}

#[test]
fn ctpop() {
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn ctpop<T: Copy>(x: T) -> T;
        }

        const GOAL: i64 = ctpop(-29);
        "#,
        61,
    );
}

#[test]
fn cttz() {
    check_number(
        r#"
        extern "rust-intrinsic" {
            pub fn cttz<T: Copy>(x: T) -> T;
        }

        const GOAL: i64 = cttz(-24);
        "#,
        3,
    );
}
