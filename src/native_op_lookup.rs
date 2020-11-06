use super::node::Node;
use super::pysexp::PySExp;
use super::types::{EvalErr, Reduction};

use super::f_table::{make_f_lookup, FLookup};

use pyo3::prelude::*;
use pyo3::types::PyTuple;

#[pyclass]
#[derive(Clone)]
pub struct NativeOpLookup {
    py_callback: PyObject,
    f_lookup: FLookup,
}

#[pymethods]
impl NativeOpLookup {
    #[new]
    fn new(native_opcode_list: &[u8], unknown_op_callback: &PyAny) -> Self {
        let native_lookup = make_f_lookup();
        let mut f_lookup: FLookup = [None; 256];
        for i in native_opcode_list.iter() {
            let idx = *i as usize;
            f_lookup[idx] = native_lookup[idx];
        }
        NativeOpLookup {
            py_callback: unknown_op_callback.into(),
            f_lookup,
        }
    }
}

impl NativeOpLookup {
    pub fn operator_handler(&self, op: &[u8], argument_list: &Node) -> Result<Reduction, EvalErr> {
        if op.len() == 1 {
            if let Some(f) = self.f_lookup[op[0] as usize] {
                return f(argument_list);
            }
        }

        Python::with_gil(|py| {
            let pysexp: PySExp = argument_list.clone().into();
            let r1 = self.py_callback.call1(py, (op, pysexp));
            match r1 {
                Err(_) => argument_list.err("fooooooooo"),
                Ok(o) => {
                    let pair: &PyTuple = o.extract(py).unwrap();
                    let i0: u32 = pair.get_item(0).extract()?;
                    let i1: PyRef<PySExp> = pair.get_item(1).extract()?;
                    let n = i1.node.clone();
                    let r: Reduction = Reduction(n, i0);
                    Ok(r)
                }
            }
        })
    }
}
