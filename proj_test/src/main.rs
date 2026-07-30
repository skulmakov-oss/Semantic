fn main() {
    let source = std::fs::read_to_string("../examples/experimental/quad_logic_calculator/src/quad_calc.proj.sm").unwrap();
    println!("Source: {}", source.len());
    let bundle = prom_ui::shell_bridge::AppBundle::compile(
        std::fs::read_to_string("../examples/experimental/quad_logic_calculator/src/calculator.sm").unwrap().as_str(),
        source.as_str()
    );
    println!("{:#?}", bundle);
}
