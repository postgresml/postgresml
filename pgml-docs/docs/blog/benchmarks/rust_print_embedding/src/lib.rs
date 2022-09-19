// fn main() {
//     let mut embeddings = [[0 as f32; 128]; 10_000];
//     for i in 0..10_000 {
//         for j in 0..128 {
//             embeddings[i][j] = rand::random()
//         }
//     };
//     // println!("{:?}", embeddings);
// }

pg_module_magic!();

#[pg_extern(immutable, parallel_safe, strict]
    fn dot_product_rust(vector: Vec<f32>, other: Vec<f32>) -> f32 {
        vector
            .as_slice()
            .iter()
            .zip(other.as_slice().iter())
            .map(|(a, b)| (a - b).powf(2.0))
            .sum::<f32>()
            .sqrt()
    }
    
#[pg_extern(immutable, parallel_safe, strict)]
fn dot_product_blas(vector: Vec<f32>, other: Vec<f32>) -> f32 {
    unsafe {
        blas::sdot(
            vector.len().try_into().unwrap(),
            vector.as_slice(),
            1,
            other.as_slice(),
            1,
        )
    }
}
