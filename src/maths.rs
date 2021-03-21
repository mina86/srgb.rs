#[cfg(all(target_feature = "fma", not(test)))]
pub(crate) fn mul_add(a: f32, b: f32, c: f32) -> f32 { a.mul_add(b, c) }

#[cfg(any(not(target_feature = "fma"), test))]
pub(crate) fn mul_add(a: f32, b: f32, c: f32) -> f32 { a * b + c }


pub(crate) fn dot_product(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    #[cfg(target_feature = "sse4.1")]
    return intel::dot_product_sse4_1(a, b);
    #[cfg(all(target_feature = "sse", not(target_feature = "sse4.1")))]
    return intel::dot_product_sse(a, b);
    #[cfg(not(target_feature = "sse"))]
    return dot_product_base(a, b);
}


#[allow(dead_code)]
fn dot_product_base(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    mul_add(a[2], b[2], mul_add(a[1], b[1], a[0] * b[0]))
}


#[test]
fn test_dot_product() {
    let a: [f32; 3] = [1.0, 2.0, 3.0];
    let b: [f32; 3] = [2.0, 20.0, 200.0];
    let want: f32 = 642.0;

    assert_eq!(want, dot_product(&a, &b));
    assert_eq!(want, dot_product_base(&a, &b));
}


#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod intel {
    #[cfg(target_arch = "x86")]
    use ::core::arch::x86 as arch;
    #[cfg(target_arch = "x86_64")]
    use ::core::arch::x86_64 as arch;


    #[cfg(any(target_feature = "sse", test))]
    #[allow(dead_code)]
    fn m128_from_array(arr: &[f32; 3]) -> arch::__m128 {
        #[repr(C, align(16))]
        struct Arr([f32; 4]);
        let arr = Arr([arr[0], arr[1], arr[2], 0.0]);
        unsafe { core::mem::transmute(arr) }
    }

    #[cfg(any(target_feature = "sse4.1", test))]
    #[allow(dead_code)]
    pub(super) fn dot_product_sse4_1(a: &[f32; 3], b: &[f32; 3]) -> f32 {
        let a = m128_from_array(a);
        let b = m128_from_array(b);
        unsafe { arch::_mm_cvtss_f32(arch::_mm_dp_ps(a, b, 0b1111_0001)) }
    }

    #[cfg(any(target_feature = "sse", test))]
    #[allow(dead_code)]
    pub(super) fn dot_product_sse(a: &[f32; 3], b: &[f32; 3]) -> f32 {
        let a = m128_from_array(a);
        let b = m128_from_array(b);
        unsafe {
            let v = arch::_mm_mul_ps(a, b);
            // https://stackoverflow.com/questions/6996764/fastest-way-to-do-horizontal-sse-vector-sum-or-other-reduction/35270026#35270026
            let shuf = arch::_mm_shuffle_ps(v, v, 0b10_11_00_01);
            let sums = arch::_mm_add_ps(v, shuf);
            let shuf = arch::_mm_movehl_ps(shuf, sums);
            let sums = arch::_mm_add_ss(sums, shuf);
            arch::_mm_cvtss_f32(sums)
        }
    }

    #[test]
    fn test_dot_product() {
        let a: [f32; 3] = [1.0, 2.0, 3.0];
        let b: [f32; 3] = [2.0, 20.0, 200.0];
        let want: f32 = 642.0;

        if std::is_x86_feature_detected!("sse4.1") {
            assert_eq!(want, dot_product_sse(&a, &b));
        }
        if std::is_x86_feature_detected!("sse4.1") {
            assert_eq!(want, dot_product_sse4_1(&a, &b));
        }
    }
}
