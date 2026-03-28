mod protocol;
mod renderer;

use std::io::{self, Read};

fn main() {
    let mut input_json = String::new();
    io::stdin().read_to_string(&mut input_json).expect("Failed to read stdin");

    let input: protocol::A2NInput = serde_json::from_str(&input_json).unwrap_or_else(|e| {
        eprintln!("Error parsing input: {}", e);
        std::process::exit(1);
    });

    let renderer = renderer::egui_impl::EguiRenderer::new();
    let output = renderer::Renderer::run(renderer, input);

    println!("{}", serde_json::to_string(&output).unwrap());
}
