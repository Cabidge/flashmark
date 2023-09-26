pub mod parse;

use parse::{
    expressions::{Expr, ExprVariant, Fraction, GroupExpr, UnitExpr},
    tokenize::token::{Function, GroupingKind},
};

pub fn render(input: &str) -> String {
    let mut output = String::new();

    output.push_str("<math>");
    for expr in parse::Parser::new(input) {
        render_expr(expr, &mut output);
    }
    output.push_str("</math>");

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
        ExprVariant::Identifier(ident) => render_simple_tag("mi", &ident, output),
        ExprVariant::Operator(op) => render_operator(&op, output),
        ExprVariant::Num(num) => render_simple_tag("mn", &num, output),
        // TODO: escape text
        ExprVariant::Text(text) => render_simple_tag("mtext", &text, output),
        ExprVariant::Unary(function, expr) => render_unary(function, *expr, output),
        ExprVariant::Grouping(group) => render_group(group, output),
    }
}

fn render_group(group: GroupExpr, output: &mut String) {
    output.push_str("<mrow>");

    match group.left {
        GroupingKind::Paren => render_operator("(", output),
        GroupingKind::Bracket => render_operator("[", output),
        GroupingKind::Brace => render_operator("{", output),
    };

    for expr in group.body {
        render_expr(expr, output);
    }

    match group.right {
        GroupingKind::Paren => render_operator(")", output),
        GroupingKind::Bracket => render_operator("]", output),
        GroupingKind::Brace => render_operator("}", output),
    };

    output.push_str("</mrow>");
}

fn render_fraction(fraction: Fraction, output: &mut String) {
    output.push_str("<mfrac>");
    render_expr(fraction.numerator, output);
    render_expr(fraction.denominator, output);
    output.push_str("</mfrac>");
}

fn render_unary(function: Function, expr: Expr, output: &mut String) {
    todo!()
}

fn render_simple_tag(tag: &str, inner: &str, output: &mut String) {
    output.push_str(&format!("<{}>{}</{}>", tag, inner, tag));
}

fn render_operator(op: &str, output: &mut String) {
    render_simple_tag("mo", op, output);
}
