use std::io::Write;

fn main() {
    foo("2 in { 1, 2, 3 }");

    // render with user input forever
    loop {
        print!("\nInput: ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        println!("Output:");
        foo(&input);
    }
}

fn foo(input: &str) {
    use flashmark::math::render;

    let output = render(input).replace("><", ">\n<");

    let mut level = 0;
    for line in output.lines() {
        if line.starts_with("</") {
            level -= 1;
        }

        println!("{:indent$}{line}", "", indent = level * 4);

        if !line.starts_with("</") && !line.chars().any(|ch| ch == '/') {
            level += 1;
        }
    }
}
