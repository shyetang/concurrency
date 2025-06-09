use anyhow::{Result, anyhow};
use std::fmt::Formatter;
use std::ops::{Add, AddAssign, Mul};
use std::sync::mpsc;
use std::{fmt, thread};

use crate::vector::{Vector, dot_product};

const NUM_THREADS: usize = 4; // 线程数

/// 矩阵结构体
///
/// # 泛型参数
/// * `T`: 矩阵元素类型，需满足Debug trait
///
/// # 字段
/// * `data`: 存储矩阵元素的向量
/// * `row`: 矩阵行数
/// * `col`: 矩阵列数
#[derive(PartialEq)]
pub struct Matrix<T> {
    data: Vec<T>,
    row: usize,
    col: usize,
}

impl<T: fmt::Debug> Matrix<T> {
    /// 创建矩阵实例
    ///
    /// # 参数
    /// * `data`: 元素数据集合
    /// * `row`: 行数
    /// * `col`: 列数
    ///
    /// # 返回值
    /// 返回Matrix<T>实例
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
    /// Debug格式化输出实现
    /// 以标准格式输出矩阵维度和内容
    /// 示例：Matrix(row=2, col=3, {1 2 3, 4 5 6})
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)
    }
}

/// 并发矩阵乘法运算
///
/// # 类型参数
/// * `T`: 元素类型，需满足多个trait约束
///
/// # 参数
/// * `a`: 左操作数矩阵
/// * `b`: 右操作数矩阵
///
/// # 返回值
/// 返回Result<Matrix<T>>，包含乘积结果或错误信息
///
/// # 并发策略
/// 使用固定大小线程池（NUM_THREADS）进行并行计算
pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
where
    T: fmt::Debug + Default + Copy + Add<Output = T> + AddAssign + Mul<Output = T> + Send + 'static,
{
    // 检查矩阵维度是否匹配
    if a.col != b.row {
        return Err(anyhow!("Matrix multiply error: a.col != b.row"));
    }

    // 创建线程池和通信通道
    let senders = (0..NUM_THREADS)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                // 线程工作循环：接收消息并计算点积
                for msg in rx {
                    let value = dot_product(msg.input.row, msg.input.col)?;
                    // 通过一次性通道返回计算结果
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        value,
                    }) {
                        eprintln!("Send error: {:?}", e);
                    }
                }
                Ok::<_, anyhow::Error>(())
            });
            tx
        })
        .collect::<Vec<_>>();

    // 初始化结果矩阵数据
    let matrix_len = a.row * b.col;
    let mut data = vec![T::default(); matrix_len];
    let mut receivers = Vec::with_capacity(matrix_len);

    // 分发计算任务
    for i in 0..a.row {
        for j in 0..b.col {
            // 提取当前行和列的数据
            let row = Vector::new(&a.data[i * a.col..(i + 1) * a.col]);
            let col_data = b.data[j..]
                .iter()
                .step_by(b.col)
                .copied()
                .collect::<Vec<_>>();
            let col = Vector::new(col_data);

            // 创建任务索引和通信通道
            let idx = i * b.col + j;
            let input = MsgInput::new(idx, row, col);
            let (tx, rx) = oneshot::channel();
            let msg = Msg::new(input, tx);

            // 轮询分配任务到线程池
            if let Err(e) = senders[idx % NUM_THREADS].send(msg) {
                eprintln!("Send error: {:?}", e)
            }
            receivers.push(rx)
        }
    }

    // 收集计算结果
    for rx in receivers {
        let msg = rx.recv()?;
        data[msg.idx] = msg.value;
    }

    // 返回最终计算结果
    Ok(Matrix {
        data,
        row: a.row,
        col: b.col,
    })
}

/// 消息输入结构体
/// 用于封装单个点积计算任务的参数
///
/// # 字段
/// * `idx`: 结果矩阵中的位置索引
/// * `row`: 当前行向量
/// * `col`: 当前列向量
pub struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

/// 消息输出结构体
/// 用于封装单个点积计算结果
///
/// # 字段
/// * `idx`: 结果矩阵中的位置索引
/// * `value`: 计算结果值
pub struct MsgOutput<T> {
    idx: usize,
    value: T,
}

impl<T> MsgInput<T> {
    /// 创建消息输入实例
    ///
    /// # 参数
    /// * `idx`: 结果矩阵中的位置索引
    /// * `row`: 当前行向量
    /// * `col`: 当前列向量
    ///
    /// # 返回值
    /// 返回MsgInput<T>实例
    pub fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

pub struct Msg<T> {
    input: MsgInput<T>,
    sender: oneshot::Sender<MsgOutput<T>>, // 一次性channel
}
impl<T> Msg<T> {
    /// 创建消息实例
    ///
    /// # 参数
    /// * `input`: 计算任务参数
    /// * `sender`: 一次性发送通道
    ///
    /// # 返回值
    /// 返回Msg<T>实例
    pub fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

impl<T> Mul for Matrix<T>
where
    T: fmt::Debug + Default + Copy + Add<Output = T> + AddAssign + Mul<Output = T> + Send + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        multiply(&self, &rhs).unwrap_or_else(|e| panic!("Matrix multiply error: {}", e))
    }
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

    #[test]
    fn test_a_can_not_multiply_b() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b);
        assert!(c.is_err());
    }

    #[test]
    #[should_panic]
    fn test_a_can_not_multiply_b_panic() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let _c = a * b;
    }
}
