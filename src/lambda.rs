use crate::btor::Bitwuzla;
use crate::{BitwuzlaOptions, Btor, BV};
use bitwuzla_sys::*;
use std::borrow::Borrow;
use std::cell::Cell;
use std::rc::Rc;

/// A bitvector object: that is, a single symbolic value, consisting of some
/// number of symbolic bits.
///
/// This is generic in the `Bitwuzla` reference type.
/// For instance, you could use `BV<Rc<Bitwuzla>>` for single-threaded applications,
/// or `BV<Arc<Bitwuzla>>` for multi-threaded applications.
#[derive(Debug)]
pub struct Lambda<R: Borrow<Bitwuzla> + Clone, const ARITY: usize> {
    pub(crate) btor: R,
    pub(crate) f: BitwuzlaTerm,
    pub(crate) c: Cell<Option<u64>>,
}

impl<R: Borrow<Bitwuzla> + Clone, const ARITY: usize> Clone for Lambda<R, ARITY> {
    fn clone(&self) -> Self {
        Self {
            f: unsafe { bitwuzla_term_copy(self.f) },
            btor: self.btor.clone(),
            c: self.c.clone(),
        }
    }
}

impl<R: Borrow<Bitwuzla> + Clone, const ARITY: usize> Drop for Lambda<R, ARITY> {
    fn drop(&mut self) {
        unsafe {
            bitwuzla_term_release(self.f);
        }
    }
}

impl<R: Borrow<Bitwuzla> + Clone, const ARITY: usize> Lambda<R, ARITY> {
    pub fn new<F: FnOnce(&[BV<R>; ARITY]) -> BV<R>>(btor: R, width: u64, f: F) -> Self {
        let tm = btor.borrow().tm;
        let args = std::array::from_fn::<BV<R>, ARITY, _>(|_| BV::param(btor.clone(), width));
        let values = f(&args);
        let mut args = args.iter().map(|el| el.node).collect::<Vec<_>>();
        args.push(values.node);
        let f = unsafe {
            bitwuzla_mk_term(
                tm,
                BITWUZLA_KIND_LAMBDA,
                1 + ARITY as u32,
                args.as_mut_ptr(),
            )
        };
        Self {
            btor,
            f,
            c: Cell::new(None),
        }
    }

    pub fn apply(&self, inp_args: &[BV<R>; ARITY]) -> BV<R> {
        let tm = self.btor.borrow().tm;

        let mut args = Vec::with_capacity(ARITY + 1);
        args.push(self.f);
        args.extend(inp_args.iter().map(|el| el.node));
        let res = unsafe {
            bitwuzla_mk_term(tm, BITWUZLA_KIND_APPLY, 1 + ARITY as u32, args.as_mut_ptr())
        };
        BV {
            btor: self.btor.clone(),
            node: res,
            c: self.c.clone(),
        }
    }
}

#[test]
fn lambda_two_args_returns_bv_and_assume_eq() {
    unsafe {
        // Setup solver + term manager
        let bzla = BitwuzlaOptions::new().with_model_gen().build();
        let btor = Rc::new(bzla);

        // Helper: construct named BV const (symbolic)
        let x = BV::new(btor.clone(), 32, Some("X"));
        let y = BV::new(btor.clone(), 32, Some("Y"));

        // Define lambda f(x, y) = (x + y) & 0xFFFF_FFFE
        let f = Lambda::<_, 2>::new(btor.clone(), 32, |[x, y]: &[BV<_>; 2]| {
            let sum = x.add(&y);
            let mask = BV::from_u32(btor.clone(), 0xFFFF_FFFE, 32);
            let ret = sum.and(&mask);
            let ret = ret.resize_unsigned(x.get_width());

            ret
        });

        // Apply: r = f(X, Y)
        let r = f.apply(&[x.clone(), y.clone()]);

        // Build equality r == 0x0000_0100
        let tgt = BV::from_u32(btor.clone(), 0x0000_0100, 32);
        let eq = r._eq(&tgt);

        // check_sat_assuming expects Bool terms
        let mut assumptions = [eq.node];
        let res = bitwuzla_check_sat_assuming(btor.btor, 1, assumptions.as_mut_ptr());
        assert!(
            res == BITWUZLA_SAT || res == BITWUZLA_UNKNOWN,
            "expected sat/unknown, got {res:?}"
        );

        // if res == BITWUZLA_SAT {
        //     let x_val = bitwuzla_get_bv_value(btor.borrow().raw(), x.node);
        //     let y_val = bitwuzla_get_bv_value(btor.borrow().raw(), y.node);
        //     let r_val = bitwuzla_get_bv_value(btor.borrow().raw(), r.node);
        //     // Basic sanity: r should be "00000000000000000000000100000000" (binary) or hex format
        //     // Depends on your get_bv_value formatting; we just ensure pointers not null here.
        //     assert!(!x_val.is_null());
        //     assert!(!y_val.is_null());
        //     assert!(!r_val.is_null());
        // }

        // Cleanup handled by Drop for Bitwuzla/BV/Lambda if your wrappers implement it.
        // If not, add:
        // bitwuzla_options_delete(opts);
        // bitwuzla_term_manager_delete(tm);
    }
}
