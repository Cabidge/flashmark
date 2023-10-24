use indoc::indoc;

fn main() {
    let engine = flashmark::template::new_engine();
    let mut scope = rhai::Scope::new();

    scope.push("s", "foo");

    let map: rhai::Map = [("a", 1), ("b", 2), ("c", 3)]
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect();

    scope.push("map", map);

    let input = indoc! {r#"
        @for i in 0..10
            @if i % 2 == 0
                @(i) is even
            @else
                @(i) is odd
            @end
        @end

        ## The characters of "@(s)":
            @for ch in s
                - @(ch)
            @end

        @for row in [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
            - row:
                @for x in row
                    @if x % 2 == 0
                        - @(x) is even
                    @else
                        - @(x) is odd
                    @end
                @end
        @end

        @for k in map.keys()
            - @(k) => @(map[k])
        @end
    "#};

    let output = flashmark::template2::render(&engine, &mut scope, input);

    println!("{}", output);
}
