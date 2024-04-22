use std::{sync::atomic::AtomicU32, sync::atomic::Ordering};

#[derive(Clone, Debug)]
pub struct Matrix(Vec<Vec<i32>>);

#[allow(dead_code)]
impl Matrix {
    pub fn random(n: usize) -> Self {
        let min_val = 0;
        let max_val = 9;
        let diff = max_val - min_val;

        let mut outer = Vec::with_capacity(n);
        for _ in 0..n {
            let mut row = Vec::with_capacity(n);
            for _ in 0..n {
                row.push(min_val + ((rand::random::<i32>() % diff) + diff) % diff);
            }
            outer.push(row);
        }
        Matrix(outer)
    }

    pub fn get_dim(&self) -> usize {
        self.0.len()
    }

    const fn serialized_dim_size() -> usize {
        std::mem::size_of::<u32>()
    }

    const fn serialized_elem_size() -> usize {
        std::mem::size_of::<i32>()
    }

    pub const fn serialized_size(matrix_dim: u32) -> usize {
        Self::serialized_dim_size() + 
            (matrix_dim * matrix_dim) as usize * Self::serialized_elem_size()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let matrix_dim = self.0.len() as u32;

        let mut serialized = Vec::with_capacity(Self::serialized_size(matrix_dim));

        let matrix_dim_bytes = matrix_dim.to_be_bytes();
        serialized.extend_from_slice(&matrix_dim_bytes);
        for row in &self.0 {
            for val in row {
                serialized.extend_from_slice(&val.to_be_bytes());
            }
        }

        serialized
    }

    fn read_be_u32(input: &mut &[u8]) -> u32 {
        let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
        *input = rest;
        u32::from_be_bytes(int_bytes.try_into().unwrap())
    }

    fn read_be_i32(input: &mut &[u8]) -> i32 {
        let (int_bytes, rest) = input.split_at(std::mem::size_of::<i32>());
        *input = rest;
        i32::from_be_bytes(int_bytes.try_into().unwrap())
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        let matrix_dim_size = Self::serialized_dim_size();
        println!("[Matrix. bytes_len: {}]", bytes.len());
        if bytes.len() < matrix_dim_size {
            return None;
        }

        let mut rest = bytes;
        let matrix_dim = Self::read_be_u32(&mut rest);
        println!("[Matrix. serialized_size: {}]", Self::serialized_size(matrix_dim));
        if bytes.len() != Self::serialized_size(matrix_dim) {
            return None;
        }

        let mut rows = Vec::with_capacity(matrix_dim as usize);
        for _ in 0..matrix_dim {
            let mut row = Vec::with_capacity(matrix_dim as usize);
            for _ in 0..matrix_dim {
                let val = Self::read_be_i32(&mut rest);
                row.push(val);
            }
            rows.push(row)
        }

        Some(Matrix(rows))
    }

    pub fn split(&mut self, parts: usize) -> Vec<MatrixSlice> {
        let matrix_dim = self.0.len();
        let amount_each = matrix_dim / parts;

        let mut result = Vec::new();
        let mut temp: &mut [Vec<i32>] = self.0.as_mut_slice();
        for i in 0..parts {
            let pair = temp.split_at_mut(if i == parts - 1 {
                amount_each + matrix_dim % parts
            } else {
                amount_each
            });
            result.push(MatrixSlice(i * amount_each, pair.0));
            temp = pair.1;
        }

        result
    }
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        let matrix1 = &self.0;
        let matrix2 = &other.0;
        // Check if matrices have the same dimensions
        if matrix1.len() != matrix2.len() || matrix1.iter().any(|row| row.len() != matrix2[0].len())
        {
            return false;
        }

        for (row1, row2) in matrix1.iter().zip(matrix2) {
            for (elem1, elem2) in row1.iter().zip(row2) {
                if elem1 != elem2 {
                    return false;
                }
            }
        }

        true
    }
}

#[allow(dead_code)]
pub struct MatrixSlice<'a>(usize, &'a mut [Vec<i32>]);
unsafe impl<'a> Send for MatrixSlice<'a> {}

#[allow(dead_code)]
pub fn count_assign_row_sums(matrix: MatrixSlice, ready: &AtomicU32) {
    let start_index = matrix.0;
    for (i, row) in matrix.1.iter_mut().enumerate() {
        row[start_index + i] = row.iter().sum();
    }
    ready.fetch_add(1, Ordering::Relaxed);
}

