use embroidery_tools::thread::{Color, Thread};

fn main() {
    let bytes = std::fs::read("without.pes").unwrap();
    let mut emb = embroidery_tools::format::pes::read_bytes(&bytes, "without.pes").unwrap();
    emb.threads[0] = Thread {
        color: Color {
            r: 34,
            g: 29,
            b: 228,
        },
        ..Default::default()
    };

    embroidery_tools::format::pes::write_path(&emb, "test.pes").unwrap();
}
