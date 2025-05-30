use anyhow::{Result, anyhow};
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Mul};

/// 矩阵
#[derive(PartialEq)]
pub struct Matrix<T> {
    data: Vec<T>,
    row: usize,
    col: usize,
}

impl<T: fmt::Debug> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}
impl<T> fmt::Display for Matrix<T>
where
    T: fmt::Display,
{
    // display a 2x3 as {1 2 3, 4 5 6},3x2 as {1 2, 3 4, 5 6}
    /// 实现格式化输出的 trait 方法
    /// 该方法用于将矩阵以字符串的形式输出
    ///
    /// # 参数
    ///
    /// * `f`: 一个借用的 `Formatter` 实例，用于输出格式化文本
    ///
    /// # 返回
    ///
    /// 返回 `fmt::Result`，表示格式化操作成功或失败
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // 写入左花括号，开始格式化输出矩阵
        write!(f, "{{")?;

        // 遍历矩阵的行
        for i in 0..self.row {
            // 遍历矩阵的列
            for j in 0..self.col {
                // 格式化输出当前元素
                write!(f, "{}", self.data[i * self.col + j])?;
                // 如果当前元素不是当前行的最后一个元素，写入一个空格
                if j != self.col - 1 {
                    write!(f, " ")?;
                }
            }
            // 如果当前行不是矩阵的最后一行，写入一个逗号和一个空格
            if i != self.row - 1 {
                write!(f, ", ")?;
            }
        }
        // 写入右花括号，完成矩阵的格式化输出
        write!(f, "}}")?;

        // 返回 Ok 表示格式化操作成功
        Ok(())
    }
}

impl<T> fmt::Debug for Matrix<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)
    }
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: fmt::Debug + Default + Copy + Add<Output = T> + AddAssign + Mul<Output = T>,
{
    if a.col != b.row {
        return Err(anyhow!("Matrix multiply error: a.col != b.row"));
    }

    let mut data = vec![T::default(); a.row * b.col]; // T 是一个泛型，这里数组没办法直接初始化为 0

    for i in 0..a.row {
        for j in 0..b.col {
            for k in 0..a.col {
                data[i * b.col + j] += a.data[i * a.col + k] * b.data[k * b.col + j];
            }
        }
    }
    Ok(Matrix {
        data,
        row: a.row,
        col: b.col,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply() -> Result<()> {
        let a = Matrix::new(vec![1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new(vec![7, 8, 9, 10, 11, 12], 3, 2);
        let c = multiply(&a, &b)?;
        assert_eq!(c, Matrix::new(vec![58, 64, 139, 154], 2, 2));
        Ok(())
    }

    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new(vec![1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new(vec![7, 8, 9, 10, 11, 12], 3, 2);
        let c = multiply(&a, &b)?;
        assert_eq!(c.data, vec![58, 64, 139, 154]);
        assert_eq!(format!("{}", c), "{58 64, 139 154}");
        Ok(())
    }
}
