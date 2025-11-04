use crate::btor::Bitwuzla;
use crate::BV;
use bitwuzla_sys::*;
use std::borrow::Borrow;
use std::cell::Cell;

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
