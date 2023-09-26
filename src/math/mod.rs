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

    let tag = match (sub_script.is_some(), super_script.is_some()) {
        (true, true) => Some("msubsup"),
        (true, false) => Some("msub"),
        (false, true) => Some("msup"),
        (false, false) => None,
    };

    if let Some(tag) = tag {
        output.push_str(&format!("<{}>", tag));
    }

    render_variant(variant, output);

    if let Some(sub_script) = sub_script {
        render_expr(sub_script, output);
    }

    if let Some(super_script) = super_script {
        render_expr(super_script, output);
    }

    if let Some(tag) = tag {
        output.push_str(&format!("</{}>", tag));
    }
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
