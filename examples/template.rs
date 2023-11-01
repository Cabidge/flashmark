use indoc::indoc;

fn main() {
    let input = indoc! {r#"
        ---
        let s = "foo";
        let map = #{ a: 1, b: 2, c: 3};
        ---
        @for i in 0..10
            @if i % 2 == 0
                @i is even
            @else
                @i is odd
            @end
        @end

        ## The characters of "@s":
            @for ch in s
                - @ch
            @end

        @for row in [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
            - row:
                @for x in row
                    @if x % 2 == 0
                        - @x is even
                    @else
                        - @x is odd
                    @end
                @end
        @end

        @for k in map.keys()
            - @k => @(map[k])
        @end

        @for x in [1, 2, 3]
            @for y in [1, 2, 3]
                @x * @y = @((x * y))
            @end
        @end
    "#};

    let output = flashmark::template::render(input);

    println!("{}", output);
}
