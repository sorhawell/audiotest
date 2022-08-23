use extendr_api::{FromRobj, Robj, Rinternals, AsTypedSlice};
use ndarray::{Array2, ArrayView2, Axis, ShapeBuilder};
use num_complex::Complex;

/// Convert to 1 channel taking the average across channels.
pub fn to_mono_ndarray(arr: &ArrayView2<f64>) -> Array2<f64> { // into_shape doesn't create a copy but mean_axis does
    arr.mean_axis(Axis(0))
        .expect("cannot mean_axis") 
        .into_shape((1, arr.ncols()))
        .expect("cannot reshape")
}

pub struct ArrayView2Wrapper<'a>(pub ArrayView2<'a, Complex<f64>>);

impl<'a> FromRobj<'a> for ArrayView2Wrapper<'a> {
    fn from_robj(robj: &'a Robj) -> std::result::Result<Self, &'static str> {
        if robj.is_matrix() {
            let nrows = robj.nrows();
            let ncols = robj.ncols();
            if let Some(v) = robj.as_typed_slice() {
                let shape = (nrows, ncols).into_shape().f();
                if let Ok(res) = ArrayView2::from_shape(shape, v) {
                    return Ok(ArrayView2Wrapper(res));
                }
            }
        }
        return Err("cannot convert Robj to ArrayView2Wrapper");
    }
}
