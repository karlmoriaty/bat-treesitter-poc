use giallo::{HighlightOptions, Registry, RenderOptions, TerminalRenderer, ThemeVariant};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).unwrap_or_else(|| {
        eprintln!("usage: giallo-bench <file>");
        std::process::exit(2);
    });

    let src = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("read error: {e}");
        std::process::exit(1);
    });

    let t0 = Instant::now();
    let mut registry = Registry::builtin().expect("builtin registry");
    registry.link_grammars();
    let build_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let options = HighlightOptions::new("rust", ThemeVariant::Single("catppuccin-mocha"));

    let t1 = Instant::now();
    let highlighted = registry.highlight(&src, &options).expect("highlight");
    let highlight_ms = t1.elapsed().as_secs_f64() * 1000.0;

    let t2 = Instant::now();
    let out = TerminalRenderer::default().render(&highlighted, &RenderOptions::default());
    let render_ms = t2.elapsed().as_secs_f64() * 1000.0;

    let esc = out.bytes().filter(|b| *b == 0x1b).count();

    println!(
        "mode=giallo formatter_build_ms={build_ms:.3} highlight_ms={highlight_ms:.3} render_ms={render_ms:.3}"
    );
    println!(
        "input_bytes={} output_bytes={} esc_count={}",
        src.len(),
        out.len(),
        esc
    );
}