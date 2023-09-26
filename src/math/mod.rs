pub mod parse;

use parse::expressions::{Expr, ExprVariant, Fraction, GroupExpr, UnitExpr};

pub fn render(input: &str) -> String {
    let mut output = String::new();
    for expr in parse::Parser::new(input) {
        render_expr(expr, &mut output);
    }

    output
}

fn render_expr(expr: Expr, output: &mut String) {
    match expr {
        Expr::Unit(unit) => render_unit(*unit, output),
        Expr::Fraction(fraction) => render_fraction(*fraction, output),
    }
}

fn render_unit(unit: UnitExpr, output: &mut String) {
    let UnitExpr {
        variant,
        super_script,
        sub_script,
    } = unit;

    render_variant(variant, output);

    todo!()
}

fn render_variant(variant: ExprVariant, output: &mut String) {
    match variant {
        ExprVariant::Grouping(group) => render_group(group, output),
        _ => todo!(),
    }
}

fn render_group(group: GroupExpr, output: &mut String) {
    todo!()
}

fn render_fraction(fraction: Fraction, output: &mut String) {
    todo!()
}
