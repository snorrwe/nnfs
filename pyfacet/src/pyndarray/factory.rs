use facet_core::ndarray::{shape::Shape, NdArray};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyList};

use super::{NdArrayB, NdArrayD};

pub fn array_f32(py: Python, shape: Vec<u32>, inp: &PyList) -> PyResult<PyObject> {
    let shape = Shape::from(shape);
    let mut values: Vec<f32> = Vec::with_capacity(shape.span());
    flatten(inp, &mut values)?;
    let arr = NdArray::<f32>::new_with_values(shape, values.into()).map_err(|err| {
        PyValueError::new_err::<String>(format!("Failed to create nd-array of f32: {}", err))
    })?;
    let res = NdArrayD { inner: arr };
    let res = Py::new(py, res)?;

    // cast result as Any
    let res = unsafe { Py::from_owned_ptr(py, res.into_ptr()) };
    Ok(res)
}

pub fn array_bool(py: Python, shape: Vec<u32>, inp: &PyList) -> PyResult<PyObject> {
    let shape = Shape::from(shape);
    let mut values: Vec<bool> = Vec::with_capacity(shape.span());
    flatten(inp, &mut values)?;
    let arr = NdArray::<bool>::new_with_values(shape, values.into()).map_err(|err| {
        PyValueError::new_err::<String>(format!("Failed to create nd-array of bool: {}", err))
    })?;
    let res = NdArrayB { inner: arr };
    let res = Py::new(py, res)?;

    // cast result as Any
    let res = unsafe { Py::from_owned_ptr(py, res.into_ptr()) };
    Ok(res)
}

fn flatten<'a, T: Clone + FromPyObject<'a>>(inp: &'a PyList, out: &mut Vec<T>) -> PyResult<()> {
    for val in inp.iter() {
        if let Ok(l) = val.downcast() {
            flatten(l, out)?;
        } else {
            out.push(val.extract()?);
        }
    }
    Ok(())
}
