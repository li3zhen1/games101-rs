use std::f32::consts::PI;

use glam::{vec3, Mat3};

// 给定一个点 P =(2,1), 将该点绕原点先逆时针旋转 45◦，再平移 (1,2), 计算出 变换后点的坐标(要求用齐次坐标进行计算)。

fn main() {
    let p = vec3(2.0, 1.0, 1.0);
    let m = Mat3::from_angle(PI * 0.25);
    let result = m * p + vec3(1.0, 2.0, 0.0);

    println!("{result}");
}
