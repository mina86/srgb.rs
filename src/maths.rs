#[cfg(all(feature = "no-fma", feature = "prefer-fma"))]
compile_error!("no-fma and prefer-fma features cannot both be enabled");

#[inline(always)]
pub(crate) fn mul_add(a: f32, b: f32, c: f32) -> f32 {
    if cfg!(feature = "no-fma") {
        a * b + c
    } else {
        a.mul_add(b, c)
    }
}


#[inline]
#[allow(dead_code)]
fn dot_product_fallback(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[2].mul_add(b[2], a[1].mul_add(b[1], a[0] * b[0]))
}


#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg(any(target_feature = "sse", test))]
mod sse {
    #[cfg(target_arch = "x86")]
    use ::core::arch::x86 as arch;
    #[cfg(target_arch = "x86_64")]
    use ::core::arch::x86_64 as arch;

    #[allow(dead_code)]
    #[target_feature(enable = "sse")]
    unsafe fn m128_from_array(arr: &[f32; 3]) -> arch::__m128 {
        #[repr(C, align(16))]
        struct Arr([f32; 4]);
        let arr = Arr([arr[0], arr[1], arr[2], 0.0]);
        core::mem::transmute(arr)
    }

    #[target_feature(enable = "sse4.1")]
    #[allow(dead_code)]
    pub(super) unsafe fn dot_product_sse4_1(a: &[f32; 3], b: &[f32; 3]) -> f32 {
        let a = m128_from_array(a);
        let b = m128_from_array(b);
        arch::_mm_cvtss_f32(arch::_mm_dp_ps(a, b, 0b1111_0001))
    }

    #[target_feature(enable = "sse")]
    #[allow(dead_code)]
    pub(super) unsafe fn dot_product_sse(a: &[f32; 3], b: &[f32; 3]) -> f32 {
        let a = m128_from_array(a);
        let b = m128_from_array(b);
        let v = arch::_mm_mul_ps(a, b);
        // https://stackoverflow.com/questions/6996764/fastest-way-to-do-horizontal-sse-vector-sum-or-other-reduction/35270026#35270026
        let shuf = arch::_mm_shuffle_ps(v, v, 0b10_11_00_01);
        let sums = arch::_mm_add_ps(v, shuf);
        let shuf = arch::_mm_movehl_ps(shuf, sums);
        let sums = arch::_mm_add_ss(sums, shuf);
        arch::_mm_cvtss_f32(sums)
    }

    #[allow(unreachable_code)]
    pub(super) fn has_sse4_1() -> bool {
        #[cfg(target_feature = "sse4.1")]
        return true;
        is_x86_feature_detected!("sse4.1")
    }

    #[allow(unreachable_code)]
    pub(super) fn has_sse() -> bool {
        #[cfg(target_feature = "sse")]
        return true;
        is_x86_feature_detected!("sse")
    }
}


macro_rules! matrix_product_body {
    ($dot:path, $matrix:ident, $column:ident) => {
        [
            $dot(&$matrix[0], &$column),
            $dot(&$matrix[1], &$column),
            $dot(&$matrix[2], &$column),
        ]
    };
}

#[inline(always)]
pub(crate) fn matrix_product(
    matrix: &[[f32; 3]; 3],
    column: [f32; 3],
) -> [f32; 3] {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if !cfg!(feature = "prefer-fma") && sse::has_sse() {
        return if sse::has_sse4_1() {
            // SAFETY: We check whether CPU supports SSE 4.1.
            unsafe {
                matrix_product_body!(sse::dot_product_sse4_1, matrix, column)
            }
        } else {
            // SAFETY: We check whether CPU supports SSE.
            unsafe {
                matrix_product_body!(sse::dot_product_sse, matrix, column)
            }
        };
    }
    matrix_product_body!(dot_product_fallback, matrix, column)
}



#[cfg(test)]
mod test {
    #[test]
    pub fn test_matrix_product() {
        let matrix = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        assert_eq!(
            [321.0, 654.0, 987.0],
            super::matrix_product(&matrix, [1.0, 10.0, 100.0])
        );
    }

    const A: [f32; 3] = [1.0, 2.0, 3.0];
    const B: [f32; 3] = [2.0, 20.0, 200.0];
    const WANT: f32 = 642.0;

    #[test]
    pub fn test_dot_product() {
        assert_eq!(WANT, super::dot_product_fallback(&A, &B));
    }

    fn unsupported(requirement: &str) {
        panic!(
            "{} required to run this test.  This failure does not mean the \
             implementation is incorrect; just that we’re unable to test it.",
            requirement
        );
    }

    #[test]
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    fn testdot_product_sse() { unsupported("x86 or x86_64 CPU"); }

    #[test]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn testdot_product_sse() {
        if is_x86_feature_detected!("sse") {
            unsafe {
                assert_eq!(WANT, super::sse::dot_product_sse(&A, &B));
            }
        }
        if is_x86_feature_detected!("sse4.1") {
            unsafe {
                assert_eq!(WANT, super::sse::dot_product_sse4_1(&A, &B));
            }
        } else {
            unsupported("SSE 4.1 support");
        }
    }
}
