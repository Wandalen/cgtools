use embroidery_tools::thread::{Color, Thread};

fn main() {
    let bytes = std::fs::read(r"D:\cgtools\module\embroidery_tools\without.pes").unwrap();
    let mut emb = embroidery_tools::format::pes::read_bytes(&bytes, "without.pes").unwrap();
    // emb.threads[0] = Thread {
    //     color: Color {
    //         r: 34,
    //         g: 29,
    //         b: 228,
    //     },
    //     ..Default::default()
    // };
    // println!("{:?}", &emb.stitches[..10]);
    embroidery_tools::format::pes::write_path(&emb, "path.pes").unwrap();
    // println!("{:?}", emb.threads);
}
