use crate::btor::Bitwuzla;
use crate::sort::Sort;
use crate::Bool;
use crate::RoundingMode;
use crate::BV;
use bitwuzla_sys::*;
use std::borrow::Borrow;
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::os::raw::c_char;

/// Enumerates the errors that may occur when using these bindings.
///
/// # Note
///
/// These do not cover internal bitwuzla errors.
#[derive(Clone, Debug)]
pub enum FPError {
    InvalidIdentifier,
}
// The attr:meta stuff is so that doc comments work correctly.
// See https://stackoverflow.com/questions/41361897/documenting-a-function-created-with-a-macro-in-rust
macro_rules! unop {
    ( $(#[$attr:meta])* => $f:ident, $kind:ident ) => {
        $(#[$attr])*
        pub fn $f(&self,rounding_mode:RoundingMode) -> Self {
            let tm = self.btor.borrow().tm;
            let rm = rounding_mode.to_node(self.btor.clone());

            Self::_new(
                self.btor.clone(),
                unsafe { bitwuzla_mk_term2(tm, $kind, rm.node, self.node) },
            )
        }
    };
}
// The attr:meta stuff is so that doc comments work correctly.
// See https://stackoverflow.com/questions/41361897/documenting-a-function-created-with-a-macro-in-rust
macro_rules! unop_non_rounding {
    ( $(#[$attr:meta])* => $f:ident, $kind:ident ) => {
        $(#[$attr])*
        pub fn $f(&self) -> Self {
            let tm = self.btor.borrow().tm;

            Self::_new(
                self.btor.clone(),
                unsafe { bitwuzla_mk_term1(tm, $kind, self.node) },
            )
        }
    };
}

// The attr:meta stuff is so that doc comments work correctly.
// See https://stackoverflow.com/questions/41361897/documenting-a-function-created-with-a-macro-in-rust
macro_rules! unop_cmp {
    ( $(#[$attr:meta])* => $f:ident, $kind:ident ) => {
        $(#[$attr])*
        pub fn $f(&self) -> Bool<R> {
            let tm = self.btor.borrow().tm;
            Bool {
                btor: self.btor.clone(),
                node: unsafe { bitwuzla_mk_term1(tm, $kind, self.node) },
            }
        }
    };
}

// The attr:meta stuff is so that doc comments work correctly.
// See https://stackoverflow.com/questions/41361897/documenting-a-function-created-with-a-macro-in-rust
macro_rules! binop {
    ( $(#[$attr:meta])* => $f:ident, $kind:ident ) => {
        $(#[$attr])*
        pub fn $f(&self, other: &Self) -> Self {
            let tm = self.btor.borrow().tm;
            Self::_new(
                self.btor.clone(),
                unsafe { bitwuzla_mk_term2(tm, $kind, self.node, other.node) },
            )
        }
    };
}

// The attr:meta stuff is so that doc comments work correctly.
// See https://stackoverflow.com/questions/41361897/documenting-a-function-created-with-a-macro-in-rust
macro_rules! binop_cmp {
    ( $(#[$attr:meta])* => $f:ident, $kind:ident ) => {
        $(#[$attr])*
        pub fn $f(&self, other: &Self) -> Bool<R> {
            let tm = self.btor.borrow().tm;
            Bool {
                btor: self.btor.clone(),
                node:  unsafe { bitwuzla_mk_term2(tm, $kind, self.node, other.node) },
            }
        }
    };
}

// The attr:meta stuff is so that doc comments work correctly.
// See https://stackoverflow.com/questions/41361897/documenting-a-function-created-with-a-macro-in-rust
macro_rules! ternop {
    ( $(#[$attr:meta])* => $f:ident, $kind:ident ) => {
        $(#[$attr])*
        pub fn $f(&self, other: &Self, rounding_mode: RoundingMode) -> Self {
            let tm = self.btor.borrow().tm;
            let rm = rounding_mode.to_node(self.btor.clone());
            Self::_new(
                 self.btor.clone(),
                unsafe { bitwuzla_mk_term3(tm, $kind, rm.node, self.node, other.node) },
            )
        }
    };
}

/// Enumerates the supported FP formats.
pub enum Formats {
    F16,
    F32,
    F64,
    F128,
}

impl Formats {
    fn fraction(&self) -> u64 {
        match self {
            Self::F16 => 10 + 1,
            Self::F32 => 23 + 1,
            Self::F64 => 53 + 1,
            Self::F128 => 113 + 1,
        }
    }

    fn exponent(&self) -> u64 {
        match self {
            Self::F128 => 128 - self.fraction(),
            Self::F64 => 64 - self.fraction(),
            Self::F32 => 32 - self.fraction(),
            Self::F16 => 16 - self.fraction(),
        }
    }
}

/// A floating-point object: that is, a single symbolic value, consisting of a
/// symbolic exponent, significand component, and sign component.
///
/// This is generic in the `Bitwuzla` reference type.
/// For instance, you could use `FP<Rc<Bitwuzla>>` for single-threaded applications,
/// or `FP<Arc<Bitwuzla>>` for multi-threaded applications.
#[derive(PartialEq, Eq)]
pub struct FP<R: Borrow<Bitwuzla> + Clone> {
    pub(crate) btor: R,
    pub(crate) is_nan: Bool<R>,
    pub(crate) node: BitwuzlaTerm,
}

impl<R: Borrow<Bitwuzla> + Clone> FP<R> {
    /// Create a new unconstrained `FP` variable of the given `exp_width` and `sig_width`.
    ///
    /// The `symbol`, if present, will be used to identify the `FP` when printing
    /// a model or dumping to file. It must be unique if it is present.
    ///
    /// # Example
    ///
    /// ```
    /// # use bitwuzla::{Bitwuzla, FP, SolverResult};
    /// let btor = Bitwuzla::new();
    ///
    /// // An 8-bit unconstrained `BV` with the symbol "foo"
    /// let fp = FP::new(&btor, 8, 23, Some("foo"));
    /// assert_eq!(format!("{:?}", fp), "foo");
    ///
    /// // Assert that it must be greater than `3`
    /// // fp.gt(&BV::from_u32(&btor, 3, 8)).assert();
    ///
    /// // Now any solution must give it a value greater than `3`
    /// assert_eq!(btor.sat(), SolverResult::Sat);
    /// // let solution = fp.get_a_solution().as_u64().unwrap();
    /// // assert!(solution > 3);
    /// ```
    pub fn new(btor: R, ty: Formats, symbol: Option<&str>) -> Result<Self, FPError> {
        let tm = btor.borrow().tm;
        let sort = Sort::fp(btor.clone(), ty.exponent(), ty.fraction());
        let node = match symbol {
            None => unsafe { bitwuzla_mk_const(tm, sort.as_raw(), std::ptr::null()) },
            Some(symbol) => {
                let cstring = CString::new(symbol).map_err(|_| FPError::InvalidIdentifier)?;
                let symbol = cstring.as_ptr() as *const c_char;
                unsafe { bitwuzla_mk_const(tm, sort.as_raw(), symbol) }
            },
        };
        Ok(Self {
            is_nan: Bool::new(btor.clone(), None),
            btor,
            node,
        })
    }

    pub(crate) fn _new(btor: R, node: BitwuzlaTerm) -> Self {
        Self {
            btor: btor.clone(),
            is_nan: Bool::new(btor, None),
            node: unsafe { bitwuzla_term_copy(node) },
        }
    }

    /// Create a new constant `FP` representing the given floating point value.
    /// The new `FP` represents an IEEE 754 binary32 value.
    pub fn from_f32(btor: R, val: f32) -> Self {
        BV::from_u32(btor, val.to_bits(), 32).to_fp(8, 23 + 1)
    }

    /// Create a new constant `FP` representing the given floating point value.
    /// The new `FP` represents an IEEE 754 binary64 value.
    pub fn from_f64(btor: R, val: f64) -> Self {
        BV::from_u64(btor, val.to_bits(), 64).to_fp(11, 52 + 1)
    }

    /// Create a new constant `FP` representing the given floating point value.
    /// The new `FP` represents an IEEE 754 binary64 value.
    pub fn new_from_f64(btor: R, val: f64, ty: Formats) -> Self {
        BV::from_u64(btor, val.to_bits(), 64).to_fp(ty.exponent(), ty.fraction())
    }

    pub fn btor(&self) -> &R {
        &self.btor
    }

    /// Get the value of the `BV` as a string of '0's and '1's. This method is
    /// only effective for `BV`s which are constant, as indicated by
    /// [`BV::is_const()`](struct.BV.html#method.is_const).
    ///
    /// This method does not require the current state to be satisfiable. To get
    /// the value of nonconstant `BV` objects given the current constraints, see
    /// [`get_a_solution()`](struct.BV.html#method.get_a_solution), which does
    /// require that the current state be satisfiable.
    ///
    /// Returns `None` if the `BV` is not constant.
    ///
    /// # Example
    ///
    /// ```
    /// # use bitwuzla::{Btor, FP};
    /// let btor = Btor::new();
    ///
    /// // This `BV` is constant, so we get a `Some`
    /// let leet = FP::from_f32(&btor, 1337.0);
    /// assert_eq!(leet.as_str().as_deref(), Some("(fp #b0 #b10001001 #b01001110010000000000000)"));
    /// assert_eq!(leet.as_f64(), Some(1337.0));
    ///
    /// // This `BV` is not constant, so we get `None`
    /// let unconstrained = FP::new(&btor, 8, 24, Some("foo"));
    /// assert_eq!(unconstrained.as_str(), None);
    /// ```
    pub fn as_str(&self) -> Option<String> {
        if self.is_const() {
            let string = unsafe { CStr::from_ptr(bitwuzla_term_to_string(self.node)) };
            Some(string.to_string_lossy().into_owned())
        } else {
            None
        }
    }

    /// # Example
    ///
    /// ```
    /// # use bitwuzla::{Btor, FP};
    /// let btor = Btor::new();
    ///
    /// // as_f64 should round-trip for every edgecase:
    /// for val in [-0., 0., f64::MIN, f64::MAX, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
    ///     let leet = FP::from_f64(&btor, val);
    ///     assert_eq!(leet.as_f64(), Some(val));
    /// }
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        if self.is_const() {
            // TODO: assert that this is a binary64 fp?
            let s = self.as_str()?;
            dbg!(&s);
            // TODO: this is fp32
            assert!(s.starts_with("(fp #b"));
            assert_eq!(
                s.len(),
                "(fp #b0 #b10000001 #b01000000000000000000000)".len()
            );
            // "(fp #b0 #b10000001 #b01000000000000000000000)"
            //  012345^789^^^^^^^^  20
            //                   18
            // let sign = &s[6 .. 7] == "1";
            // Some(u64::from_str_radix(&s, 2).unwrap())
            todo!()
        } else {
            None
        }
    }

    /// # Example
    ///
    /// ```
    /// # use bitwuzla::{Btor, FP};
    /// let btor = Btor::new();
    ///
    /// // as_f64 should round-trip for every edgecase:
    /// for val in [-0., 0., f64::MIN, f64::MAX, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
    ///     let leet = FP::from_f64(&btor, val);
    ///     assert_eq!(leet.as_f64(), Some(val));
    /// }
    /// ```
    pub fn as_f128(&self) -> Option<f64> {
        if self.is_const() {
            // TODO: assert that this is a binary64 fp?
            let s = self.as_str()?;

            dbg!(&s);
            // TODO: this is fp32
            assert!(s.starts_with("(fp #b"));
            assert_eq!(
                s.len(),
                "(fp #b0 #b10000001 #b01000000000000000000000)".len()
            );
            // "(fp #b0 #b10000001 #b01000000000000000000000)"
            //  012345^789^^^^^^^^  20
            //                   18
            // let sign = &s[6 .. 7] == "1";
            // Some(u64::from_str_radix(&s, 2).unwrap())
            todo!()
        } else {
            None
        }
    }

    /// Does the `FP` have a constant value?
    ///
    /// Note: bitwuzla `is_const` is something entirely different
    ///
    /// # Examples
    ///
    /// ```
    /// # use bitwuzla::{Btor, FP, RoundingMode};
    /// let btor = Btor::new();
    ///
    /// // This `FP` is constant
    /// let pi = FP::from_f32(&btor, 3.1415);
    /// assert!(pi.is_const());
    ///
    /// // This `BV` is not constant
    /// let unconstrained = FP::new_binary32(&btor, Some("foo"));
    /// assert!(!unconstrained.is_const());
    ///
    /// // pi + [unconstrained] is also not constant
    /// let sum = pi.add(&unconstrained, RoundingMode::RTN);
    /// assert!(!sum.is_const());
    ///
    /// // But pi + pi is constant
    /// let tau = pi.add(&pi, RoundingMode::RTN);
    /// assert!(tau.is_const());
    /// ```
    pub fn is_const(&self) -> bool {
        unsafe { bitwuzla_term_is_value(self.node) }
    }

    unop_non_rounding!(
        /// Floating-point absolute value.
        => abs, BITWUZLA_KIND_FP_ABS
    );

    ternop!(
        /// Floating-point addition. `self` and `other` must have the same layout.
        => add, BITWUZLA_KIND_FP_ADD
    );

    ternop!(
        /// Floating-point division. `self` and `other` must have the same layout.
        => div, BITWUZLA_KIND_FP_DIV
    );

    binop_cmp!(
        /// Floating-point equality. `self` and `other` must have the same bitwidth.
        /// Resulting `BV` will have bitwidth 1.
        => _eq, BITWUZLA_KIND_FP_EQUAL
    );

    binop_cmp!(
        /// Floating-point greater than or equal. `self` and `other` must have the same bitwidth.
        /// Resulting `BV` will have bitwidth 1.
        => geq, BITWUZLA_KIND_FP_GEQ
    );

    binop_cmp!(
        /// Floating-point greater than. `self` and `other` must have the same bitwidth.
        /// Resulting `BV` will have bitwidth 1.
        => gt, BITWUZLA_KIND_FP_GT
    );

    /// Floating-point is Nan tester.
    /// Resulting `BV` will have bitwidth 1.
    pub fn is_nan(&self) -> Bool<R> {
        self.is_nan.clone()
    }
    // unop_cmp!(
    //     /// Floating-point is Nan tester.
    //     /// Resulting `BV` will have bitwidth 1.
    //     => is_nan, BITWUZLA_KIND_FP_IS_NAN
    // );

    unop_cmp!(
        /// Floating-point is negative tester.
        /// Resulting `BV` will have bitwidth 1.
        => is_neg, BITWUZLA_KIND_FP_IS_NEG
    );

    unop_cmp!(
        /// Floating-point is subnormal tester.
        /// Resulting `BV` will have bitwidth 1.
        => is_subnormal, BITWUZLA_KIND_FP_IS_SUBNORMAL
    );

    unop_cmp!(
        /// Floating-point is normal tester.
        /// Resulting `BV` will have bitwidth 1.
        => is_normal, BITWUZLA_KIND_FP_IS_NORMAL
    );

    unop_cmp!(
        /// Floating-point is normal tester.
        /// Resulting `BV` will have bitwidth 1.
        => is_infinite, BITWUZLA_KIND_FP_IS_INF
    );

    unop_cmp!(
        /// Floating-point is zero tester.
        /// Resulting `BV` will have bitwidth 1.
        => is_zero, BITWUZLA_KIND_FP_IS_ZERO
    );

    binop_cmp!(
        /// Floating-point less than. `self` and `other` must have the same bitwidth.
        /// Resulting `BV` will have bitwidth 1.
        => lt, BITWUZLA_KIND_FP_LT
    );

    binop_cmp!(
        /// Floating-point greater than or equal. `self` and `other` must have the same bitwidth.
        /// Resulting `BV` will have bitwidth 1.
        => leq, BITWUZLA_KIND_FP_LEQ
    );

    binop!(
        /// Floating-point max. `self` and `other` must have the same layout.
        => max, BITWUZLA_KIND_FP_MAX
    );

    binop!(
        /// Floating-point min. `self` and `other` must have the same layout.
        => min, BITWUZLA_KIND_FP_MIN
    );

    ternop!(
        /// Floating-point multiplcation. `self` and `other` must have the same layout.
        => mul, BITWUZLA_KIND_FP_MUL
    );

    unop!(
        /// Floating-point negation.
        => neg, BITWUZLA_KIND_FP_NEG
    );

    binop!(
        /// Floating-point remainder. `self` and `other` must have the same layout.
        => rem, BITWUZLA_KIND_FP_REM
    );

    /// Floating-point round to integral.
    pub fn round_to_integral(&self, rounding_mode: RoundingMode) -> Self {
        let tm = self.btor.borrow().tm;
        let rm = rounding_mode.to_node(self.btor.clone());
        Self::_new(self.btor.clone(), unsafe {
            bitwuzla_mk_term2(tm, BITWUZLA_KIND_FP_RTI, rm.node, self.node)
        })
    }

    unop!(
        /// Floating-point round to square root. (sic)
        => sqrt, BITWUZLA_KIND_FP_SQRT
    );

    ternop!(
        /// Floating-point round to subtraction. (sic)
        => sub, BITWUZLA_KIND_FP_SUB
    );

    //pub fn to_ieee754_bv(&self) -> BV<R> {
    //    let tm = self.btor.borrow().tm;
    //    // TODO: assert width?
    //    BV {
    //        btor: self.btor.clone(),
    //        node: unsafe { bitwuzla_mk_term1(tm, BITWUZLA_KIND_FP_TO_FP_TO_BV, self.node) },
    //    }
    //}
    //
    pub fn from_ieee754_bv(bv: &BV<R>, ty: &Formats) -> Self {
        let tm = bv.borrow().btor.borrow().tm;
        let (e, s) = (ty.exponent(), ty.fraction());
        // TODO: assert width?
        FP::_new(bv.btor.clone(), unsafe {
            bitwuzla_mk_term1_indexed2(tm, BITWUZLA_KIND_FP_TO_FP_FROM_BV, bv.node, e, s)
        })
    }

    pub fn to_sbv(&self, rounding_mode: RoundingMode, width: u64) -> BV<R> {
        let tm = self.btor.borrow().tm;
        let rm = rounding_mode.to_node(self.btor().clone());
        // TODO: assert width?
        BV::_new(self.btor.clone(), unsafe {
            bitwuzla_mk_term2_indexed1(tm, BITWUZLA_KIND_FP_TO_SBV, rm.node, self.node, width)
        })
    }

    pub fn to_ubv(&self, rounding_mode: RoundingMode, width: u64) -> BV<R> {
        let tm = self.btor.borrow().tm;
        let rm = rounding_mode.to_node(self.btor().clone());
        BV::_new(self.btor.clone(), unsafe {
            bitwuzla_mk_term2_indexed1(tm, BITWUZLA_KIND_FP_TO_UBV, rm.node, self.node, width)
        })
    }

    pub fn from_ubv(bv: BV<R>, rounding_mode: RoundingMode, ty: &Formats) -> Self {
        let tm = bv.btor.borrow().tm;
        let rm = rounding_mode.to_node(bv.btor.clone());
        FP::_new(bv.btor.clone(), unsafe {
            bitwuzla_mk_term2_indexed2(
                tm,
                BITWUZLA_KIND_FP_TO_FP_FROM_UBV,
                rm.node,
                bv.node,
                ty.exponent(),
                ty.fraction(),
            )
        })
    }

    pub fn from_sbv(bv: BV<R>, rounding_mode: RoundingMode, ty: &Formats) -> Self {
        let tm = bv.btor.borrow().tm;
        let rm = rounding_mode.to_node(bv.btor.clone());
        FP::_new(bv.btor.clone(), unsafe {
            bitwuzla_mk_term2_indexed2(
                tm,
                BITWUZLA_KIND_FP_TO_FP_FROM_SBV,
                rm.node,
                bv.node,
                ty.exponent(),
                ty.fraction(),
            )
        })
    }

    pub fn to_fp32(&self) -> FP<R> {
        self.to_fp(8, 23 + 1)
    }

    pub fn to_fp64(&self) -> FP<R> {
        self.to_fp(11, 52 + 1)
    }

    pub fn unconstrained(&self, ty: &Formats, name: Option<&str>) -> Result<FP<R>, FPError> {
        let tm = self.btor.borrow().tm;
        let sort = Sort::fp(self.btor.clone(), ty.exponent(), ty.fraction());

        Ok(FP::_new(
            self.btor.clone(),
            match name {
                None => unsafe { bitwuzla_mk_const(tm, sort.as_raw(), core::ptr::null()) },
                Some(name) => {
                    let name = CString::new(name).map_err(|_| FPError::InvalidIdentifier)?;
                    let cname = name.as_ptr();
                    unsafe { bitwuzla_mk_const(tm, sort.as_raw(), cname) }
                },
            },
        ))
    }

    pub fn to_fp(&self, exp_width: u64, sig_width: u64) -> FP<R> {
        let tm = self.btor.borrow().tm;
        FP::_new(self.btor.clone(), unsafe {
            bitwuzla_mk_term1_indexed2(
                tm,
                BITWUZLA_KIND_FP_TO_FP_FROM_FP,
                self.node,
                exp_width,
                sig_width,
            )
        })
    }
}

impl<R: Borrow<Bitwuzla> + Clone> Clone for FP<R> {
    fn clone(&self) -> Self {
        Self {
            node: unsafe { bitwuzla_term_copy(self.node) },
            btor: self.btor.clone(),
            is_nan: self.is_nan.clone(),
        }
    }
}

impl<R: Borrow<Bitwuzla> + Clone> Drop for FP<R> {
    fn drop(&mut self) {
        unsafe {
            // bitwuzla_term_release(self.node);
        }
    }
}

impl<R: Borrow<Bitwuzla> + Clone> fmt::Debug for FP<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = unsafe { CStr::from_ptr(bitwuzla_term_to_string(self.node)) };
        write!(f, "{}", string.to_string_lossy())
    }
}
