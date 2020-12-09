use crate::{
    ndarray::NdArray,
    ndarray::{matrix::matmul_impl, shape::Shape},
    DuResult,
};
use std::f64::consts::E;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[cfg(feature = "rayon")]
pub fn relu(inp: &NdArray<f64>) -> NdArray<f64> {
    let mut out = inp.clone();
    out.par_iter_cols_mut().for_each(|col| {
        for v in col {
            *v = v.max(0.0);
        }
    });
    out
}

#[cfg(not(feature = "rayon"))]
pub fn relu(inp: &NdArray<f64>) -> NdArray<f64> {
    inp.map(|v| v.max(0.0))
}

/// ReLU derivative
pub fn drelu_dz(inputs: &NdArray<f64>, dvalues: &NdArray<f64>) -> NdArray<f64> {
    // #[cfg(feature = "rayon")]
    let mut res = dvalues.clone();
    for (dx, dz) in res.iter_cols_mut().zip(inputs.iter_cols()) {
        debug_assert_eq!(dx.len(), dz.len());
        for i in 0..dx.len() {
            if dz[i] <= 0.0 {
                dx[i] = 0.0;
            }
        }
    }
    res
}

/// Inp is interpreted as a either a collection of vectors, applying softmax to each column or as a
/// single vector.
///
/// Scalars will always return 1
pub fn softmax(inp: &NdArray<f64>) -> DuResult<NdArray<f64>> {
    // softmax of a scalar value is always 1.
    if matches!(inp.shape(), Shape::Scalar(_)) {
        return Ok(NdArray::new_with_values(0, [1.0][..].into())?);
    }
    // else treat the input as a collection of vectors
    let mut it = inp.as_slice().iter().cloned();
    let first = it.next().expect("no value");
    let max: f64 = it.fold(first, |max, value| value.max(max));

    let expvalues = inp
        .sub(&NdArray::from(max))
        .expect("Failed to sub max from the input")
        .map(|v: &f64| {
            let res = E.powf(*v);
            if res.is_nan() || res == 0.0 {
                // very large V's will produce an output of 0 which will be bad down the line
                1e-12
            } else {
                res
            }
        });

    let mut norm_base: NdArray<f64> = expvalues
        .iter_cols()
        .map(|col| col.iter().cloned().sum())
        .collect();

    norm_base.reshape([norm_base.shape().span() as u32, 1]);

    let mut res = expvalues;
    for (norm, col) in norm_base.iter_cols().zip(res.iter_cols_mut()) {
        debug_assert_eq!(norm.len(), 1);
        col.iter_mut().for_each(|v| *v /= norm[0]);
    }
    Ok(res)
}

/// Softmax backwards pass, calculating gradient
pub fn dsoftmax(output: &NdArray<f64>, dvalues: &NdArray<f64>) -> DuResult<NdArray<f64>> {
    let mut res = NdArray::new(dvalues.shape().clone());

    let collen = output.shape().last().unwrap();

    let mut jacobian_matrix = NdArray::new([collen, collen]);
    let mut dotcache = NdArray::new([collen, collen]);

    for (i, (output, dvalues)) in output.iter_cols().zip(dvalues.iter_cols()).enumerate() {
        diagflat(output, &mut jacobian_matrix);
        matmul_impl(
            [collen, 1],
            output,
            [1, collen],
            output,
            dotcache.as_mut_slice(),
        )?;

        jacobian_matrix = jacobian_matrix.sub(&dotcache)?;

        matmul_impl(
            [collen, collen],
            jacobian_matrix.as_slice(),
            [collen, 1],
            dvalues,
            res.get_column_mut(&[i as u32]).unwrap(),
        )?;
    }

    Ok(res)
}

fn diagflat(output: &[f64], mat: &mut NdArray<f64>) {
    for i in 0..output.len() {
        for j in 0..output.len() {
            let i = i as u32;
            let j = j as u32;
            // branchless setter, if the item is not in the diagonal set it to 0
            *mat.get_mut(&[i, j]).unwrap() = output[i as usize] * (i == j) as u32 as f64;
        }
    }
}

pub fn sigmoid(input: &NdArray<f64>, output: &mut NdArray<f64>) -> DuResult<()> {
    output.reshape(input.shape().clone());
    for (x, y) in input
        .as_slice()
        .iter()
        .zip(output.as_mut_slice().iter_mut())
    {
        *y = 1.0 / (1. + E.powf(-x))
    }
    Ok(())
}

/// Derivative of sigmoid function
///
/// dvalues * (1-output) * output
pub fn dsigmoid(output: &NdArray<f64>, dvalues: &NdArray<f64>) -> DuResult<NdArray<f64>> {
    let v = dvalues.mul(&(NdArray::new_scalar(1.).sub(output)?))?;
    Ok(v.mul(output)?)
}
