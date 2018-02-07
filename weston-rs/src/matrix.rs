use std::{mem, ops};
use libweston_sys::{
    weston_vector, weston_matrix,
    weston_matrix_init, weston_matrix_multiply, weston_matrix_scale,
    weston_matrix_translate, weston_matrix_rotate_xy,
    weston_matrix_transform, weston_matrix_invert,
    weston_matrix_transform_type_WESTON_MATRIX_TRANSFORM_TRANSLATE,
    weston_matrix_transform_type_WESTON_MATRIX_TRANSFORM_SCALE,
    weston_matrix_transform_type_WESTON_MATRIX_TRANSFORM_ROTATE,
    weston_matrix_transform_type_WESTON_MATRIX_TRANSFORM_OTHER,
};
use num_traits::{ToPrimitive, FromPrimitive};

/// Four f32 values
pub type Vector = weston_vector;

/// Purpose of a matrix
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Primitive)]
pub enum TransformType {
    Translate = weston_matrix_transform_type_WESTON_MATRIX_TRANSFORM_TRANSLATE,
    Scale = weston_matrix_transform_type_WESTON_MATRIX_TRANSFORM_SCALE,
    Rotate = weston_matrix_transform_type_WESTON_MATRIX_TRANSFORM_ROTATE,
    Other = weston_matrix_transform_type_WESTON_MATRIX_TRANSFORM_OTHER,
}
 

/// Matrices are stored in column-major order, that is the array indices are:
///
/// 0  4  8 12  
/// 1  5  9 13  
/// 2  6 10 14  
/// 3  7 11 15  
pub struct Matrix {
    pub weston: weston_matrix
}

impl ops::Index<usize> for Matrix {
    type Output = f32;

    #[inline]
    fn index(&self, i: usize) -> &f32 {
        &self.weston.d[i]
    }
}

impl ops::IndexMut<usize> for Matrix {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        &mut self.weston.d[i]
    }
}

impl Default for Matrix {
    /// Returns a new identity matrix, with the type set as Translate
    fn default() -> Matrix {
        let mut weston: weston_matrix = unsafe { mem::zeroed() };
        unsafe { weston_matrix_init(&mut weston); }
        Matrix { weston }
    }
}

impl Matrix {
    pub fn new(type_: TransformType, d: [f32; 16]) -> Matrix {
        Matrix {
            weston: weston_matrix {
                d,
                type_: type_.to_u32().unwrap_or(weston_matrix_transform_type_WESTON_MATRIX_TRANSFORM_OTHER),
            }
        }
    }

    #[inline]
    pub fn transform_type(&self) -> TransformType {
        TransformType::from_u32(self.weston.type_).unwrap_or(TransformType::Other)
    }

    /// m ← n * m, that is, m is multiplied on the LEFT
    #[inline]
    pub fn multiply(&mut self, other: &Matrix) -> &mut Matrix {
        unsafe { weston_matrix_multiply(&mut self.weston, &other.weston); }
        self
    }

    #[inline]
    pub fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut Matrix {
        unsafe { weston_matrix_scale(&mut self.weston, x, y, z); }
        self
    }

    #[inline]
    pub fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut Matrix {
        unsafe { weston_matrix_translate(&mut self.weston, x, y, z); }
        self
    }

    #[inline]
    pub fn rotate_xy(&mut self, cos: f32, sin: f32) -> &mut Matrix {
        unsafe { weston_matrix_rotate_xy(&mut self.weston, cos, sin); }
        self
    }

    /// v ← m * v
    #[inline]
    pub fn transform<'a>(&mut self, v: &'a mut Vector) -> &'a mut Vector {
        unsafe { weston_matrix_transform(&mut self.weston, v); }
        v
    }

    #[inline]
    pub fn invert(&self) -> Matrix {
        let mut weston: weston_matrix = unsafe { mem::zeroed() };
        unsafe { weston_matrix_invert(&mut weston, &self.weston); }
        Matrix { weston }
    }
}
