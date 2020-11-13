#![macro_use]

pub fn size_of_slice<T: Sized>(slice: &[T]) -> usize {
    std::mem::size_of::<T>() * slice.len()
}

pub fn make_spirv(data: &[u8]) -> std::borrow::Cow<[u32]> {
    const MAGIC_NUMBER: u32 = 0x723_0203;

    assert_eq!(
        data.len() % std::mem::size_of::<u32>(),
        0,
        "data size is not a multiple of 4"
    );

    let words = if data.as_ptr().align_offset(std::mem::align_of::<u32>()) == 0 {
        let (pre, words, post) = unsafe { data.align_to::<u32>() };
        debug_assert!(pre.is_empty());
        debug_assert!(post.is_empty());
        std::borrow::Cow::from(words)
    } else {
        let mut words = vec![0u32; data.len() / std::mem::size_of::<u32>()];
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), words.as_mut_ptr() as *mut u8, data.len());
        }
        std::borrow::Cow::from(words)
    };
    assert_eq!(
        words[0], MAGIC_NUMBER,
        "wrong magic word {:x}. Make sure you are using a binary SPIRV file.",
        words[0]
    );
    words
}

#[macro_export]
macro_rules! any {
    ($x:expr, $($y:expr),+ $(,)?) => {
        {
            false $(|| $x == $y)+
        }
    };
}

#[macro_export]
macro_rules! include_str_from_outdir {
    ($t: literal) => {
        include_str!(concat!(env!("OUT_DIR"), $t))
    };
}

#[macro_export]
macro_rules! include_bytes_from_outdir {
    ($t: literal) => {
        include_bytes!(concat!(env!("OUT_DIR"), $t))
    };
}

#[macro_export]
macro_rules! include_spirv_from_outdir {
    ($t: literal) => {
        crate::utils::make_spirv(include_bytes_from_outdir!($t))
    };
}

#[macro_export]
macro_rules! tuple_as {
    ($e:expr, ( $T0:ty, $T1:ty, $T2:ty, $T3:ty, $T4:ty, $T5:ty ) ) => {
        (
            $e.0 as $T0,
            $e.1 as $T1,
            $e.2 as $T2,
            $e.3 as $T3,
            $e.4 as $T4,
            $e.5 as $T5,
        )
    };
    ($e:expr, ( $T0:ty, $T1:ty, $T2:ty, $T3:ty, $T4:ty ) ) => {
        (
            $e.0 as $T0,
            $e.1 as $T1,
            $e.2 as $T2,
            $e.3 as $T3,
            $e.4 as $T4,
        )
    };
    ($e:expr, ( $T0:ty, $T1:ty, $T2:ty, $T3:ty ) ) => {
        ($e.0 as $T0, $e.1 as $T1, $e.2 as $T2, $e.3 as $T3)
    };
    ($e:expr, ( $T0:ty, $T1:ty, $T2:ty ) ) => {
        ($e.0 as $T0, $e.1 as $T1, $e.2 as $T2)
    };
    ($e:expr, ( $T0:ty, $T1:ty ) ) => {
        ($e.0 as $T0, $e.1 as $T1)
    };
    ($e:expr, ( $T0:ty, ) ) => {
        ($e.0 as $T0,)
    };
}
